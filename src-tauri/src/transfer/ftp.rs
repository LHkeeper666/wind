use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::sync::Mutex as TokioMutex;

use log::{info, warn, error, debug};

use crate::ftp::FtpManager;

use super::TransferTask;
use super::helpers::{check_cancelled, extract_ftp_conn};

pub async fn execute_ftp_copy(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn = task.conn_name.clone().or_else(|| extract_ftp_conn(&task.source))
        .ok_or("Missing FTP connection")?;
    let source = task.source.strip_prefix(&format!("ftp://{conn}")).ok_or("Invalid FTP source")?;
    let destination = task.destination.strip_prefix(&format!("ftp://{conn}")).ok_or("Invalid FTP destination")?;
    let temp = std::env::temp_dir().join(format!("wind_ftp_copy_{}_{}", std::process::id(), task.id));
    let result = async {
        let session = crate::ftp::independent_from_manager(ftp_manager, &conn).await?;
        let mut ftp = session.lock().await;
        let mut stream = tokio::time::timeout(std::time::Duration::from_secs(30), ftp.client.retr_as_stream(source)).await
            .map_err(|_| "FTP RETR timed out".to_string())?.map_err(|e| format!("FTP RETR failed: {e}"))?;
        let mut file = tokio::fs::File::create(&temp).await.map_err(|e| e.to_string())?;
        let mut buffer = vec![0u8; 65536];
        let mut done = 0u64;
        loop {
            if cancel_flag.load(Ordering::Relaxed) { return Err("Cancelled".into()); }
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let n = tokio::time::timeout(std::time::Duration::from_secs(30), stream.read(&mut buffer)).await
                .map_err(|_| "FTP read timed out".to_string())?.map_err(|e| e.to_string())?;
            if n == 0 { break; }
            file.write_all(&buffer[..n]).await.map_err(|e| e.to_string())?;
            done += n as u64;
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id, "op_type": task.op_type, "status": "running",
                "bytes_done": done, "total_bytes": task.total_bytes, "speed_bps": 0,
                "source": task.source, "destination": task.destination,
            }));
        }
        use tokio::io::AsyncWriteExt;
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);
        tokio::time::timeout(std::time::Duration::from_secs(30), ftp.client.finalize_retr_stream(stream)).await
            .map_err(|_| "FTP RETR finalize timed out".to_string())?.map_err(|e| format!("FTP RETR finalize failed: {e}"))?;
        if cancel_flag.load(Ordering::Relaxed) { return Err("Cancelled".into()); }
        let file = tokio::fs::File::open(&temp).await.map_err(|e| e.to_string())?;
        let mut reader = ProgressAsyncReader {
            inner: file, done: 0, total: done, task_id: task.id, batch_id: task.batch_id,
            op_type: task.op_type, source: task.source.clone(), dest: task.destination.clone(),
            last_emit: Instant::now(), speed_sample_at: Instant::now(),
            speed_sample_bytes: 0, current_speed_bps: 0,
            cancel_flag: cancel_flag.clone(), app: app.clone(),
        };
        let upload = tokio::time::timeout(std::time::Duration::from_secs(300), ftp.client.put_file(destination, &mut reader)).await
            .map_err(|_| "FTP upload timed out".to_string())?.map_err(|e| format!("FTP upload failed: {e}"));
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = ftp.client.rm(destination).await;
            return Err("Cancelled".into());
        }
        upload?;
        let _ = ftp.client.quit().await;
        Ok(done)
    }.await;
    let _ = tokio::fs::remove_file(&temp).await;
    result
}

pub async fn execute_ftp_download(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.source))
        .ok_or_else(|| "No FTP connection for download".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.source[6..].find('/') {
        &task.source[6 + pos..]
    } else {
        "/"
    };
    info!("[transfer] FTP download: conn={conn_name} remote={remote_path} → local={}", task.destination);

    let setup_started = Instant::now();
    let session = crate::ftp::independent_from_manager(ftp_manager, conn_name).await?;
    info!("[transfer] FTP download task {} session ready in {} ms", task.id, setup_started.elapsed().as_millis());

    let mut ftp = session.lock().await;

    // Try to get file size for progress display
    let file_size = match ftp.client.size(remote_path).await {
        Ok(s) => { debug!("[transfer] FTP SIZE: {s} bytes"); s }
        Err(e) => { warn!("[transfer] FTP SIZE failed (ignored): {e}"); 0 }
    };

    let retr_started = Instant::now();
    let mut stream = ftp.client.retr_as_stream(remote_path)
        .await
        .map_err(|e| {
            let msg = format!("FTP download failed: {e}");
            error!("[transfer] {msg}");
            msg
        })?;
    info!("[transfer] FTP download task {} RETR ready in {} ms", task.id, retr_started.elapsed().as_millis());

    let local = Path::new(&task.destination);
    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(local)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut buf = vec![0u8; 65536];
    let mut done: u64 = 0;
    let total = if file_size > 0 { file_size as u64 } else { task.total_bytes };
    // Immediately update frontend with the correct file size
    if total > 0 {
        if let Err(e) = app.emit("transfer-progress", serde_json::json!({
            "id": task.id, "batch_id": task.batch_id,
            "op_type": task.op_type, "status": "running",
            "bytes_done": 0, "total_bytes": total, "speed_bps": 0,
            "source": task.source, "destination": task.destination,
        })) {
            log::debug!("[emit] transfer-progress failed: {}", e);
        }
    }
    let mut last_emit = Instant::now();
    let start = Instant::now();
    let mut speed_sample_at = start;
    let mut speed_sample_bytes = 0u64;
    let mut current_speed_bps = 0u64;
    let mut last_diagnostic = start;
    let mut diagnostic_bytes = 0u64;
    let mut read_wait = std::time::Duration::ZERO;
    let mut write_wait = std::time::Duration::ZERO;
    let mut reads = 0u64;

    loop {
        if check_cancelled(cancel_flag) {
            drop(file);
            let _ = tokio::fs::remove_file(local).await;
            return Err("Cancelled".into());
        }

        use tokio::io::AsyncReadExt;
        let read_started = Instant::now();
        let n = tokio::time::timeout(std::time::Duration::from_secs(30), stream.read(&mut buf))
            .await.map_err(|_| "Download read timed out".to_string())?
            .map_err(|e| format!("Download read error: {}", e))?;
        read_wait += read_started.elapsed();
        if n == 0 { break; }

        let write_started = Instant::now();
        file.write_all(&buf[..n])
            .await
            .map_err(|e| format!("Download write error: {}", e))?;
        write_wait += write_started.elapsed();

        done += n as u64;
        reads += 1;
        let now = Instant::now();
        let sample_elapsed = now.duration_since(speed_sample_at).as_secs_f64();
        if sample_elapsed >= 1.0 {
            current_speed_bps = ((done - speed_sample_bytes) as f64 / sample_elapsed) as u64;
            speed_sample_at = now;
            speed_sample_bytes = done;
        }
        if now.duration_since(last_diagnostic).as_secs() >= 5 {
            let interval = now.duration_since(last_diagnostic).as_secs_f64();
            info!(
                "[transfer] FTP download task {}: bytes={}/{}, interval_mib_s={:.2}, avg_mib_s={:.2}, read_wait_ms={}, write_wait_ms={}, reads={}",
                task.id, done, total,
                (done - diagnostic_bytes) as f64 / interval / 1_048_576.0,
                done as f64 / start.elapsed().as_secs_f64() / 1_048_576.0,
                read_wait.as_millis(), write_wait.as_millis(), reads,
            );
            last_diagnostic = now;
            diagnostic_bytes = done;
            read_wait = std::time::Duration::ZERO;
            write_wait = std::time::Duration::ZERO;
            reads = 0;
        }
        if now.duration_since(last_emit).as_millis() >= 100 {
            if let Err(e) = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": done, "total_bytes": total,
                "speed_bps": current_speed_bps,
                "source": task.source, "destination": task.destination,
            })) {
                log::debug!("[emit] transfer-progress failed: {}", e);
            }
            last_emit = now;
        }
    }

    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    ftp.client.finalize_retr_stream(stream).await
        .map_err(|e| format!("Download finalize error: {e}"))?;
    info!("[transfer] FTP download task {} completed: bytes={}, elapsed_ms={}", task.id, done, start.elapsed().as_millis());
    let _ = ftp.client.quit().await;
    Ok(done)
}

pub async fn execute_ftp_upload(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.destination))
        .ok_or_else(|| "No FTP connection for upload".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.destination[6..].find('/') {
        &task.destination[6 + pos..]
    } else {
        "/"
    };
    info!("[transfer] FTP upload: local={} → conn={conn_name} remote={remote_path}", task.source);

    let session = crate::ftp::independent_from_manager(ftp_manager, conn_name).await?;

    let mut ftp = session.lock().await;
    let local = Path::new(&task.source);
    let file = tokio::fs::File::open(local)
        .await
        .map_err(|e| format!("Failed to open local file: {}", e))?;
    let total = file.metadata()
        .await
        .map(|m| m.len())
        .unwrap_or(task.total_bytes);

    let mut reader = ProgressAsyncReader {
        inner: file,
        done: 0,
        total,
        task_id: task.id,
        batch_id: task.batch_id,
        op_type: task.op_type,
        source: task.source.clone(),
        dest: task.destination.clone(),
        last_emit: Instant::now(),
        speed_sample_at: Instant::now(),
        speed_sample_bytes: 0,
        current_speed_bps: 0,
        cancel_flag: cancel_flag.clone(),
        app: app.clone(),
    };

    let put_result = ftp.client.put_file(remote_path, &mut reader).await;

    let result = match put_result {
        Ok(_) => Ok(reader.done),
        Err(e) => {
            if reader.cancel_flag.load(Ordering::Relaxed) {
                // Try to delete partial file from server
                let _ = ftp.client.rm(remote_path).await;
                info!("[transfer] FTP upload cancelled, cleaned up remote file: {remote_path}");
                Err("Cancelled".into())
            } else {
                Err(format!("FTP upload failed: {}", e))
            }
        }
    };
    let _ = ftp.client.quit().await;
    result
}

pub async fn execute_ftp_delete(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.source))
        .ok_or_else(|| "No FTP connection for delete".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.source[6..].find('/') {
        &task.source[6 + pos..]
    } else {
        "/"
    };
    info!("[transfer] FTP delete: conn={conn_name} remote={remote_path}");

    let session = crate::ftp::independent_from_manager(ftp_manager, conn_name).await?;

    let mut ftp = session.lock().await;

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }

    // Try rm (file), fall back to rmdir (directory)
    let result = match ftp.client.rm(remote_path).await {
        Ok(_) => {
            if let Err(e) = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": 1, "total_bytes": 1,
                "speed_bps": 0,
                "source": task.source, "destination": task.destination,
            })) {
                log::debug!("[emit] transfer-progress failed: {}", e);
            }
            Ok(1)
        }
        Err(rm_err) => {
            warn!("[transfer] FTP delete rm failed, trying rmdir: {rm_err}");
            ftp.client.rmdir(remote_path)
                .await
                .map_err(|e| format!("FTP delete failed (both rm and rmdir): {rm_err} / {e}"))?;
            if let Err(e) = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": 1, "total_bytes": 1,
                "speed_bps": 0,
                "source": task.source, "destination": task.destination,
            })) {
                log::debug!("[emit] transfer-progress failed: {}", e);
            }
            Ok(1)
        }
    };
    let _ = ftp.client.quit().await;
    result
}

struct ProgressAsyncReader {
    inner: tokio::fs::File,
    done: u64,
    total: u64,
    task_id: u64,
    batch_id: u64,
    op_type: super::TransferType,
    source: String,
    dest: String,
    last_emit: Instant,
    speed_sample_at: Instant,
    speed_sample_bytes: u64,
    current_speed_bps: u64,
    cancel_flag: Arc<AtomicBool>,
    app: AppHandle,
}

impl AsyncRead for ProgressAsyncReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        if this.cancel_flag.load(Ordering::Relaxed) {
            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Cancelled")));
        }
        let filled_before = buf.filled().len();
        let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        let result = AsyncRead::poll_read(inner, cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            let n = (buf.filled().len() - filled_before) as u64;
            if n > 0 {
                this.done += n;
                let now = Instant::now();
                let sample_elapsed = now.duration_since(this.speed_sample_at).as_secs_f64();
                if sample_elapsed >= 1.0 {
                    this.current_speed_bps = ((this.done - this.speed_sample_bytes) as f64 / sample_elapsed) as u64;
                    this.speed_sample_at = now;
                    this.speed_sample_bytes = this.done;
                }
                if now.duration_since(this.last_emit).as_millis() >= 100 {
                    if let Err(e) = this.app.emit("transfer-progress", serde_json::json!({
                        "id": this.task_id, "batch_id": this.batch_id,
                        "op_type": this.op_type, "status": "running",
                        "bytes_done": this.done, "total_bytes": this.total,
                        "speed_bps": this.current_speed_bps,
                        "source": this.source, "destination": this.dest,
                    })) {
                        log::debug!("[emit] transfer-progress failed: {}", e);
                    }
                    this.last_emit = now;
                }
            }
        }
        result
    }
}

/// Helper: MKD accepting 250/257/200 (Android FTP servers return 250 for MKD).
async fn mkd_compat(
    client: &mut suppaftp::tokio::AsyncRustlsFtpStream,
    path: &str,
) -> Result<(), suppaftp::FtpError> {
    client
        .custom_command(
            format!("MKD {}", path),
            &[
                suppaftp::Status::RequestedFileActionOk,
                suppaftp::Status::PathCreated,
                suppaftp::Status::CommandOk,
            ],
        )
        .await
        .map(|_| ())
}

/// Ensure a remote directory path exists on the FTP server, creating
/// intermediate directories as needed.
pub async fn ensure_remote_dir(
    client: &mut suppaftp::tokio::AsyncRustlsFtpStream,
    remote_path: &str,
) -> Result<(), String> {
    let path = remote_path.trim_end_matches('/');
    // Try creating the full path directly
    match mkd_compat(client, path).await {
        Ok(_) => return Ok(()),
        Err(e) => {
            // CWD to check if directory already exists (550 can mean either)
            if client.cwd(path).await.is_ok() {
                return Ok(());
            }
            warn!("[transfer] MKD '{}' direct failed, building by segments: {}", path, e);
        }
    }

    // Build directory path segment by segment
    // Android FTP servers often reject absolute MKD on protected paths but allow
    // relative MKD after CWD into the parent directory
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut built = String::new();
    for seg in segments {
        built.push('/');
        built.push_str(seg);
        // Try absolute MKD first
        match mkd_compat(client, &built).await {
            Ok(_) => {
                let _ = client.cwd(&built).await;
                continue;
            }
            Err(_) => {
                // MKD failed — could be "already exists" (550) or real error.
                // Use CWD to probe: if CWD succeeds, directory exists.
                if client.cwd(&built).await.is_ok() {
                    continue;
                }
                // Try relative MKD after CWD to parent
                let parent = if built.len() > seg.len() + 1 {
                    &built[..built.len() - seg.len() - 1]
                } else {
                    "/"
                };
                let _ = client.cwd(parent).await;
                match mkd_compat(client, seg).await {
                    Ok(_) => {
                        let _ = client.cwd(&built).await;
                        continue;
                    }
                    Err(e2) => {
                        // Check if it was created (some servers return error but still create)
                        if client.cwd(&built).await.is_ok() {
                            continue;
                        }
                        return Err(format!("MKD '{}' failed: {}", built, e2));
                    }
                }
            }
        }
    }
    // Restore to root
    let _ = client.cwd("/").await;
    Ok(())
}
