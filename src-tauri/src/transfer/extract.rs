use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tauri::{AppHandle, Emitter};

use crate::archive;

use super::TransferTask;

/// Execute an extraction task with progress emission and cancellation support.
pub async fn execute_extract(
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

    let app_clone = app.clone();
    let cancel = cancel_flag.clone();
    let task_id = task.id;
    let batch_id = task.batch_id;
    let op_type = task.op_type;
    let source = task.source.clone();
    let dest = task.destination.clone();
    let total = task.total_bytes;

    let last_emit = Arc::new(std::sync::Mutex::new(Instant::now()));
    let bytes_done = Arc::new(std::sync::Mutex::new(0u64));

    let progress: archive::ExtractProgress = Arc::new(move |chunk_bytes: u64| {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }

        let mut done = bytes_done.lock().unwrap();
        *done += chunk_bytes;
        let current_done = *done;
        drop(done);

        let mut last = last_emit.lock().unwrap();
        if last.elapsed().as_millis() >= 100 {
            *last = Instant::now();
            drop(last);
            if let Err(e) = app_clone.emit("transfer-progress", serde_json::json!({
                "id": task_id,
                "batch_id": batch_id,
                "op_type": op_type,
                "status": "running",
                "bytes_done": current_done,
                "total_bytes": total,
                "speed_bps": 0u64,
                "source": source,
                "destination": dest,
            })) {
                log::debug!("[emit] transfer-progress failed: {}", e);
            }
        }

        true
    });

    let internal_paths = task.internal_paths.clone();

    // Run extraction in blocking task
    let result = tokio::task::spawn_blocking(move || {
        if let Some(ref paths) = internal_paths {
            archive::extract_files(
                &archive_path,
                paths,
                &dest_dir,
                password,
                &progress,
            )?;
            Ok(0u64)
        } else {
            archive::extract_all(
                &archive_path,
                &dest_dir,
                password,
                skip_paths.as_ref(),
                &progress,
            )
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result
}