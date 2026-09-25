use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

use super::{TransferTask, TransferType};
use super::helpers::{check_cancelled, dir_size};

pub async fn execute_local_copy(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let src = Path::new(&task.source);
    let dst = Path::new(&task.destination);

    if !src.exists() {
        return Err(format!("Source not found: {}", task.source));
    }

    // Ensure parent dir
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tokio::task::spawn_blocking({
        let cancel = cancel_flag.clone();
        let app = app.clone();
        let task_clone = task.clone();
        let src = src.to_path_buf();
        let dst = dst.to_path_buf();
        move || {
            let mut task_corrected = task_clone;
            // Runtime fallback: if enqueue didn't compute the directory size,
            // compute it here (on the blocking thread) and emit the correct total.
            if src.is_dir() && task_corrected.total_bytes == 0 {
                let actual_total = dir_size(&src);
                if actual_total > 0 {
                    task_corrected.total_bytes = actual_total;
                    let _ = app.emit("transfer-progress", serde_json::json!({
                        "id": task_corrected.id,
                        "batch_id": task_corrected.batch_id,
                        "op_type": task_corrected.op_type,
                        "status": "running",
                        "bytes_done": 0,
                        "total_bytes": actual_total,
                        "speed_bps": 0,
                        "source": task_corrected.source,
                        "destination": task_corrected.destination,
                    }));
                }
            }
            local_copy_blocking(&src, &dst, &cancel, &app, &task_corrected)
        }
    }).await
    .map_err(|e| format!("Join error: {}", e))?
}

fn local_copy_blocking(
    src: &Path,
    dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    task: &TransferTask,
) -> Result<u64, String> {
    let mut total: u64 = 0;
    let mut last_emit = Instant::now();
    let dst_existed = dst.exists();
    let mut created: Vec<PathBuf> = Vec::new();
    let skip: HashSet<String> = task.skip_rel_paths.iter().cloned().collect();

    let copy_result = if src.is_dir() {
        copy_dir_with_progress(src, dst, cancel_flag, app, task, &mut total, &mut last_emit, &mut created, &skip, src)
    } else {
        copy_file_with_progress(src, dst, cancel_flag, app, task, &mut total, &mut last_emit)
    };

    if let Err(e) = copy_result {
        if e == "Cancelled" && src.is_dir() {
            cleanup_cancelled_dir(dst, dst_existed, &created);
        }
        return Err(e);
    }

    // If this is a move, delete source after copy
    if task.op_type == TransferType::Move {
        if src.is_dir() {
            if skip.is_empty() {
                let _ = fs::remove_dir_all(src);
            } else {
                delete_with_progress(src, cancel_flag, app, task, &mut total, &mut last_emit)?;
            }
        } else {
            let _ = fs::remove_file(src);
        }
    }

    Ok(total)
}

fn cleanup_cancelled_dir(dst: &Path, dst_existed: bool, created: &[PathBuf]) {
    if !dst_existed {
        let _ = fs::remove_dir_all(dst);
    } else {
        for f in created {
            let _ = fs::remove_file(f);
        }
    }
}

fn copy_file_with_progress(
    src: &Path, dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle, task: &TransferTask,
    total: &mut u64, last_emit: &mut Instant,
) -> Result<(), String> {
    let mut reader = fs::File::open(src)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let mut writer = fs::File::create(dst)
        .map_err(|e| format!("Failed to create: {}", e))?;
    let mut buf = [0u8; 65536];
    let source_str = task.source.clone();
    let dest_str = task.destination.clone();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            drop(writer);
            let _ = fs::remove_file(dst);
            return Err("Cancelled".into());
        }
        let n = reader.read(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        std::io::Write::write_all(&mut writer, &buf[..n])
            .map_err(|e| format!("Write error: {}", e))?;
        *total += n as u64;

        let now = Instant::now();
        if now.duration_since(*last_emit).as_millis() >= 100 {
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": *total, "total_bytes": task.total_bytes,
                "speed_bps": 0,
                "source": source_str, "destination": dest_str,
            }));
            *last_emit = now;
        }
    }

    writer.flush().map_err(|e| format!("Flush error: {}", e))?;
    Ok(())
}

fn copy_dir_with_progress(
    src: &Path, dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle, task: &TransferTask,
    total: &mut u64, last_emit: &mut Instant,
    created: &mut Vec<PathBuf>,
    skip: &HashSet<String>,
    root: &Path,
) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Read dir error: {}", e))? {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let rel = path.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if skip.contains(&rel) {
            continue;
        }

        if path.is_dir() {
            copy_dir_with_progress(&path, &dst_path, cancel_flag, app, task, total, last_emit, created, skip, root)?;
        } else {
            copy_file_with_progress(&path, &dst_path, cancel_flag, app, task, total, last_emit)?;
            created.push(dst_path);
        }
    }
    Ok(())
}

pub async fn execute_local_delete(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let path = Path::new(&task.source);
    if !path.exists() {
        return Ok(0);
    }

    tokio::task::spawn_blocking({
        let cancel = cancel_flag.clone();
        let app = app.clone();
        let task_clone = task.clone();
        let path = path.to_path_buf();
        move || {
            let mut total: u64 = 0;
            let mut last_emit = Instant::now();
            delete_with_progress(&path, &cancel, &app, &task_clone, &mut total, &mut last_emit)
                .map(|_| total)
        }
    }).await
    .map_err(|e| format!("Join error: {}", e))?
}

fn delete_with_progress(
    path: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    task: &TransferTask,
    total: &mut u64,
    last_emit: &mut Instant,
) -> Result<(), String> {
    delete_with_progress_filtered(path, cancel_flag, app, task, total, last_emit, path, &task.skip_rel_paths.iter().map(|p| p.replace('\\', "/")).collect())
}

fn delete_with_progress_filtered(
    path: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    task: &TransferTask,
    total: &mut u64,
    last_emit: &mut Instant,
    root: &Path,
    skip: &HashSet<String>,
) -> Result<(), String> {
    if check_cancelled(cancel_flag) {
        return Err("Cancelled".into());
    }

    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("Read dir error: {}", e))? {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            let relative = entry.path().strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if skip.iter().any(|excluded| relative == *excluded || relative.starts_with(&format!("{excluded}/"))) {
                continue;
            }
            delete_with_progress_filtered(&entry.path(), cancel_flag, app, task, total, last_emit, root, skip)?;
        }
        let is_empty = fs::read_dir(path)
            .map_err(|e| format!("Read dir error: {}", e))?
            .next()
            .is_none();
        if is_empty {
            fs::remove_dir(path).map_err(|e| format!("Remove dir error: {}", e))?;
        }
    } else {
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        fs::remove_file(path).map_err(|e| format!("Remove file error: {}", e))?;
        *total += size;

        let now = Instant::now();
        if now.duration_since(*last_emit).as_millis() >= 100 {
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": *total, "total_bytes": task.total_bytes,
                "speed_bps": 0,
                "source": task.source.clone(), "destination": task.destination.clone(),
            }));
            *last_emit = now;
        }
    }
    Ok(())
}