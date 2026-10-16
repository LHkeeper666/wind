use std::collections::{HashSet, VecDeque};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::archive;
use crate::tool_cache;

use super::TransferTask;

/// One-line rollback switch: flip to `false` to run extraction in-process again
/// (see design D11). The worker path is the default so an abort can't take the
/// UI down with it.
const EXTRACT_USE_WORKER: bool = true;

/// How long to wait for a worker to honour a negotiated cancel before killing it.
const CANCEL_GRACE: Duration = Duration::from_secs(5);
/// Number of trailing stderr lines kept for attributing an abnormal exit.
const STDERR_TAIL_LINES: usize = 20;

/// Execute an extraction task with progress emission and cancellation support.
pub async fn execute_extract(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    if EXTRACT_USE_WORKER {
        execute_extract_via_worker(task, cancel_flag, app).await
    } else {
        execute_extract_in_process(task, cancel_flag, app).await
    }
}

/// Spawn `wind.exe --archive-extract-worker` for this task and translate its
/// stdio back into the existing UI events. Extraction crashes here, not in the
/// UI process.
async fn execute_extract_via_worker(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot locate worker executable: {e}"))?;

    // argv carries only the mode flag — the password goes over stdin.
    let mut command = tool_cache::background_command(&exe);
    command
        .arg("--archive-extract-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = tokio::process::Command::from(command).spawn().map_err(|e| {
        // No silent fallback to in-process extraction (design D10).
        log::error!("[extract-worker] task {}: failed to spawn worker: {}", task.id, e);
        format!("failed to start extract worker: {e}")
    })?;

    let mut stdin = child.stdin.take().ok_or_else(|| "worker stdin unavailable".to_string())?;
    let stdout = child.stdout.take().ok_or_else(|| "worker stdout unavailable".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "worker stderr unavailable".to_string())?;

    let task_id = task.id;

    // Both pipes must be drained continuously; a full pipe buffer would stall the
    // worker. Start the readers before waiting on the process.
    let stderr_tail = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let stderr_handle = {
        let tail = stderr_tail.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                {
                    let mut tail = tail.lock().unwrap();
                    if tail.len() == STDERR_TAIL_LINES {
                        tail.pop_front();
                    }
                    tail.push_back(line.clone());
                }
                log::warn!("[extract-worker] task {}: {}", task_id, line);
            }
        })
    };

    let payload = serde_json::json!({
        "archive_path": task.source,
        "dest_dir": task.destination,
        "password": task.password,
        "internal_paths": task.internal_paths,
        "skip_rel_paths": task.skip_rel_paths,
    });
    let mut payload_line = payload.to_string();
    payload_line.push('\n');
    if let Err(e) = stdin.write_all(payload_line.as_bytes()).await {
        log::error!("[extract-worker] task {}: failed to send task: {}", task_id, e);
        let _ = child.kill().await;
        return Err(format!("failed to send task to extract worker: {e}"));
    }
    let _ = stdin.flush().await;

    let stdout_handle = {
        let app = app.clone();
        let id = task.id;
        let batch_id = task.batch_id;
        let op_type = task.op_type;
        let total = task.total_bytes;
        let source = task.source.clone();
        let dest = task.destination.clone();
        tokio::spawn(async move {
            let mut bytes_done = 0u64;
            let mut error: Option<String> = None;
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                match value.get("event").and_then(|e| e.as_str()) {
                    Some("entry") => {
                        if let Some(done) = value.get("bytes_done").and_then(|v| v.as_u64()) {
                            bytes_done = done;
                        }
                        let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let size = value.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        emit_progress(&app, id, batch_id, op_type, total, &source, &dest, bytes_done, Some((name, size)));
                    }
                    Some("progress") => {
                        if let Some(done) = value.get("bytes_done").and_then(|v| v.as_u64()) {
                            bytes_done = done;
                        }
                        emit_progress(&app, id, batch_id, op_type, total, &source, &dest, bytes_done, None);
                    }
                    Some("done") => {
                        if let Some(bytes) = value.get("bytes").and_then(|v| v.as_u64()) {
                            bytes_done = bytes;
                        }
                    }
                    Some("error") => {
                        error = value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
            (bytes_done, error)
        })
    };

    let mut cancel_sent = false;
    let mut cancel_deadline: Option<Instant> = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                log::error!("[extract-worker] task {}: wait failed: {}", task_id, e);
                let _ = child.kill().await;
                break child
                    .wait()
                    .await
                    .map_err(|e| format!("worker wait failed: {e}"))?;
            }
        }

        if !cancel_sent && cancel_flag.load(Ordering::Relaxed) {
            cancel_sent = true;
            if let Err(e) = stdin.write_all(b"{\"cmd\":\"cancel\"}\n").await {
                log::warn!("[extract-worker] task {}: failed to send cancel: {}", task_id, e);
            }
            let _ = stdin.flush().await;
            cancel_deadline = Some(Instant::now() + CANCEL_GRACE);
        }

        if let Some(deadline) = cancel_deadline {
            if Instant::now() >= deadline {
                log::warn!("[extract-worker] task {}: no exit {}s after cancel, terminating", task_id, CANCEL_GRACE.as_secs());
                let _ = child.kill().await;
                break child
                    .wait()
                    .await
                    .map_err(|e| format!("worker wait failed: {e}"))?;
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    };

    drop(stdin);
    let (bytes_done, worker_error) = stdout_handle.await.unwrap_or((0, None));
    let _ = stderr_handle.await;
    let tail: Vec<String> = stderr_tail.lock().unwrap().iter().cloned().collect();
    let tail_text = tail.join(" | ");

    if cancel_flag.load(Ordering::Relaxed) {
        log::info!("[extract-worker] task {} cancelled", task_id);
        return Err("cancelled".to_string());
    }

    if status.success() {
        log::info!("[extract-worker] task {} done: bytes={}", task_id, bytes_done);
        return Ok(bytes_done);
    }

    match status.code() {
        Some(code) if code >= 0 => {
            let message = worker_error.unwrap_or_else(|| format!("extract worker exited with code {code}"));
            log::warn!("[extract-worker] task {} failed (exit {}): {}", task_id, code, message);
            Err(message)
        }
        Some(code) => {
            log::error!(
                "[extract-worker] task {} terminated abnormally: 0x{:08X}; last stderr: {}",
                task_id,
                code as u32,
                tail_text
            );
            Err(format!("extract worker terminated abnormally (0x{:08X})", code as u32))
        }
        None => {
            log::error!(
                "[extract-worker] task {} terminated by signal; last stderr: {}",
                task_id,
                tail_text
            );
            Err("extract worker terminated abnormally".to_string())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_progress(
    app: &AppHandle,
    id: u64,
    batch_id: u64,
    op_type: super::TransferType,
    total: u64,
    source: &str,
    dest: &str,
    bytes_done: u64,
    entry: Option<(&str, u64)>,
) {
    let mut payload = serde_json::json!({
        "id": id,
        "batch_id": batch_id,
        "op_type": op_type,
        "status": "running",
        "bytes_done": bytes_done,
        "total_bytes": total,
        "speed_bps": 0u64,
        "source": source,
        "destination": dest,
    });
    if let Some((name, size)) = entry {
        if let Some(object) = payload.as_object_mut() {
            object.insert("entry_name".to_string(), serde_json::json!(name));
            object.insert("entry_size".to_string(), serde_json::json!(size));
        }
    }
    if let Err(e) = app.emit("transfer-progress", payload) {
        log::debug!("[emit] transfer-progress failed: {}", e);
    }
}

/// In-process fallback (rollback switch). Retained so a broken worker path can
/// be reverted without reverting the rest of the change.
async fn execute_extract_in_process(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let archive_path = task.source.clone();
    let dest_dir = task.destination.clone();
    let password = task.password.clone();
    let skip_paths: Option<HashSet<String>> = if task.skip_rel_paths.is_empty() {
        None
    } else {
        Some(task.skip_rel_paths.iter().cloned().collect())
    };

    let cancel = cancel_flag.clone();
    let task_id = task.id;
    let batch_id = task.batch_id;
    let op_type = task.op_type;
    let source = task.source.clone();
    let dest = task.destination.clone();
    let total = task.total_bytes;

    let last_emit = Arc::new(Mutex::new(Instant::now()));
    let bytes_done = Arc::new(Mutex::new(0u64));

    let progress: archive::ExtractProgress = {
        let bytes_done = bytes_done.clone();
        let last_emit = last_emit.clone();
        let app_clone = app.clone();
        let source = source.clone();
        let dest = dest.clone();
        Arc::new(move |chunk_bytes: u64| {
            if cancel.load(Ordering::Relaxed) {
                return false;
            }

            let current_done = {
                let mut done = bytes_done.lock().unwrap();
                *done += chunk_bytes;
                *done
            };

            let mut last = last_emit.lock().unwrap();
            if last.elapsed().as_millis() >= 100 {
                *last = Instant::now();
                drop(last);
                emit_progress(&app_clone, task_id, batch_id, op_type, total, &source, &dest, current_done, None);
            }

            true
        })
    };

    let on_entry: archive::EntryStartProgress = {
        let bytes_done = bytes_done.clone();
        let last_emit = last_emit.clone();
        let app_clone = app.clone();
        let source = source.clone();
        let dest = dest.clone();
        Arc::new(move |name: &str, size: u64| {
            let current_done = *bytes_done.lock().unwrap();
            *last_emit.lock().unwrap() = Instant::now();
            emit_progress(&app_clone, task_id, batch_id, op_type, total, &source, &dest, current_done, Some((name, size)));
        })
    };

    let internal_paths = task.internal_paths.clone();

    log::info!(
        "[archive] extract task {} start (in-process): archive={} dest={}",
        task.id,
        task.source,
        task.destination
    );

    let joined = tokio::task::spawn_blocking(move || {
        if let Some(ref paths) = internal_paths {
            archive::extract_files(
                &archive_path,
                paths,
                &dest_dir,
                password,
                &progress,
                Some(&on_entry),
            )?;
            Ok(0u64)
        } else {
            archive::extract_all(
                &archive_path,
                &dest_dir,
                password,
                skip_paths.as_ref(),
                &progress,
                Some(&on_entry),
            )
        }
    })
    .await;

    let result = joined.map_err(|e| format!("Task join error: {}", e))?;

    match &result {
        Ok(bytes) => log::info!("[archive] extract task {} done: bytes={}", task.id, bytes),
        Err(e) => log::warn!("[archive] extract task {} failed: {}", task.id, e),
    }

    result
}
