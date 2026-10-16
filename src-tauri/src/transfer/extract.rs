use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::archive;

use super::TransferTask;

const MEMORY_LOG_INTERVAL: Duration = Duration::from_secs(5);

/// Spawns a background thread that logs the process memory footprint every few
/// seconds until `stop` is set. Makes a memory-exhaustion stall distinguishable
/// from a merely slow extraction in the log.
fn spawn_memory_logger(stop: Arc<AtomicBool>) -> Option<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("archive-mem-log".into())
        .spawn(move || {
            use windows::Win32::System::ProcessStatus::{
                GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
            };
            use windows::Win32::System::Threading::GetCurrentProcess;

            let process = unsafe { GetCurrentProcess() };
            let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

            while !stop.load(Ordering::Relaxed) {
                // Sleep in short slices so the stop signal is observed promptly.
                let slices = MEMORY_LOG_INTERVAL.as_millis() / 250;
                for _ in 0..slices {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(250));
                }

                let mut counters = PROCESS_MEMORY_COUNTERS {
                    cb,
                    ..Default::default()
                };
                if unsafe { GetProcessMemoryInfo(process, &mut counters, cb) }.is_ok() {
                    log::info!(
                        "[archive] extract memory: working_set={} private={}",
                        counters.WorkingSetSize,
                        counters.PagefileUsage
                    );
                }
            }
        })
        .ok()
}

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

    let cancel = cancel_flag.clone();
    let task_id = task.id;
    let batch_id = task.batch_id;
    let op_type = task.op_type;
    let source = task.source.clone();
    let dest = task.destination.clone();
    let total = task.total_bytes;

    let last_emit = Arc::new(std::sync::Mutex::new(Instant::now()));
    let bytes_done = Arc::new(std::sync::Mutex::new(0u64));

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
        })
    };

    // Entry boundaries are the only place the UI learns which file is currently
    // being written: unrar's extract path reports no bytes mid-entry. Emit
    // immediately (bypassing the throttle) so the UI can show an indeterminate
    // state for entries too large to report a live percentage for.
    let on_entry: archive::EntryStartProgress = {
        let bytes_done = bytes_done.clone();
        let last_emit = last_emit.clone();
        let app_clone = app.clone();
        let source = source.clone();
        let dest = dest.clone();
        Arc::new(move |name: &str, size: u64| {
            let current_done = *bytes_done.lock().unwrap();
            *last_emit.lock().unwrap() = Instant::now();
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
                "entry_name": name,
                "entry_size": size,
            })) {
                log::debug!("[emit] transfer-progress failed: {}", e);
            }
        })
    };

    let internal_paths = task.internal_paths.clone();

    log::info!(
        "[archive] extract task {} start: archive={} dest={}",
        task.id, task.source, task.destination
    );

    let mem_stop = Arc::new(AtomicBool::new(false));
    let mem_logger = spawn_memory_logger(mem_stop.clone());

    // Run extraction in blocking task
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

    mem_stop.store(true, Ordering::Relaxed);
    if let Some(handle) = mem_logger {
        let _ = handle.join();
    }

    let result = joined.map_err(|e| format!("Task join error: {}", e))?;

    match &result {
        Ok(bytes) => log::info!("[archive] extract task {} done: bytes={}", task.id, bytes),
        Err(e) => log::warn!("[archive] extract task {} failed: {}", task.id, e),
    }

    result
}