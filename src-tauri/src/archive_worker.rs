//! Extraction worker: `wind.exe --archive-extract-worker` runs archive
//! extraction in its own process so a process-fatal failure (e.g. the
//! `0xC0000409` fail-fast abort) takes down the worker, not the UI.
//!
//! Contract with the parent (see `transfer/extract.rs`):
//! - stdin: one line of task JSON, then optional `{"cmd":"cancel"}` lines.
//!   The password travels here, never in argv.
//! - stdout: one JSON event per line — `entry` / `progress` / `done` / `error`.
//! - stderr: plain diagnostic lines, forwarded by the parent (which adds the
//!   `[extract-worker]` marker and the task id).
//!
//! No `log` logger is installed: the parent owns the log file, and the child's
//! stderr is a pipe to the parent.

use std::collections::HashSet;
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::archive;

const MEMORY_LOG_INTERVAL: Duration = Duration::from_secs(5);
const PROGRESS_THROTTLE: Duration = Duration::from_millis(100);

#[derive(Deserialize)]
struct WorkerTask {
    archive_path: String,
    dest_dir: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    internal_paths: Option<Vec<String>>,
    #[serde(default)]
    skip_rel_paths: Vec<String>,
}

/// Entry point. Never returns to the Tauri builder path.
pub fn run() {
    install_panic_hook();

    // Read the task line without holding the stdin lock (StdinLock is not Send,
    // so it cannot follow us into the watcher thread).
    let mut first = String::new();
    if std::io::stdin().read_line(&mut first).is_err() || first.trim().is_empty() {
        eprintln!("no task received on stdin");
        std::process::exit(2);
    }
    let task: WorkerTask = match serde_json::from_str(first.trim()) {
        Ok(task) => task,
        Err(e) => {
            eprintln!("invalid task JSON: {e}");
            std::process::exit(2);
        }
    };

    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let cancelled = cancelled.clone();
        let _ = std::thread::Builder::new()
            .name("worker-cancel-watch".into())
            .spawn(move || {
                // Cancel arrives as another line on stdin while extraction runs.
                let stdin = std::io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(line) if line.contains("cancel") => {
                            cancelled.store(true, Ordering::Relaxed);
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            });
    }

    std::process::exit(run_task(&task, &cancelled));
}

fn run_task(task: &WorkerTask, cancelled: &Arc<AtomicBool>) -> i32 {
    let bytes_done = Arc::new(std::sync::Mutex::new(0u64));
    let last_emit = Arc::new(std::sync::Mutex::new(Instant::now()));

    let progress: archive::ExtractProgress = {
        let bytes_done = bytes_done.clone();
        let last_emit = last_emit.clone();
        let cancelled = cancelled.clone();
        Arc::new(move |chunk_bytes: u64| {
            if cancelled.load(Ordering::Relaxed) {
                return false;
            }
            let current = {
                let mut done = bytes_done.lock().unwrap();
                *done += chunk_bytes;
                *done
            };
            let mut last = last_emit.lock().unwrap();
            if last.elapsed() >= PROGRESS_THROTTLE {
                *last = Instant::now();
                drop(last);
                emit_event(serde_json::json!({
                    "event": "progress",
                    "bytes_done": current,
                }));
            }
            true
        })
    };

    // Entry boundaries bypass the throttle so the UI can switch to an
    // indeterminate state immediately (unrar reports no bytes mid-entry).
    let on_entry: archive::EntryStartProgress = {
        let bytes_done = bytes_done.clone();
        let last_emit = last_emit.clone();
        Arc::new(move |name: &str, size: u64| {
            let current = *bytes_done.lock().unwrap();
            *last_emit.lock().unwrap() = Instant::now();
            emit_event(serde_json::json!({
                "event": "entry",
                "name": name,
                "size": size,
                "bytes_done": current,
            }));
        })
    };

    let mem_stop = Arc::new(AtomicBool::new(false));
    let mem_logger = spawn_memory_logger(mem_stop.clone());

    let result = if let Some(ref paths) = task.internal_paths {
        archive::extract_files(
            &task.archive_path,
            paths,
            &task.dest_dir,
            task.password.clone(),
            &progress,
            Some(&on_entry),
        )
        .map(|_| 0u64)
    } else {
        let skip_paths: Option<HashSet<String>> = if task.skip_rel_paths.is_empty() {
            None
        } else {
            Some(task.skip_rel_paths.iter().cloned().collect())
        };
        archive::extract_all(
            &task.archive_path,
            &task.dest_dir,
            task.password.clone(),
            skip_paths.as_ref(),
            &progress,
            Some(&on_entry),
        )
    };

    mem_stop.store(true, Ordering::Relaxed);
    if let Some(handle) = mem_logger {
        let _ = handle.join();
    }

    match result {
        Ok(bytes) => {
            emit_event(serde_json::json!({ "event": "done", "bytes": bytes }));
            0
        }
        Err(message) => {
            if cancelled.load(Ordering::Relaxed) {
                eprintln!("extraction cancelled");
            } else {
                eprintln!("extraction failed: {message}");
                emit_event(serde_json::json!({ "event": "error", "message": message }));
            }
            1
        }
    }
}

fn emit_event(value: serde_json::Value) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{value}");
    let _ = out.flush();
}

fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown location>".to_string());
        eprintln!("panic: {message} at {location}");
        previous(info);
    }));
}

/// Logs this process's memory footprint every few seconds to stderr (forwarded
/// by the parent). Makes a memory-exhaustion stall distinguishable from a merely
/// slow extraction — and, since it now lives in the worker, it survives to
/// report the run-up to a main-process-fatal condition.
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
                    eprintln!(
                        "[archive] extract memory: working_set={} private={}",
                        counters.WorkingSetSize, counters.PagefileUsage
                    );
                }
            }
        })
        .ok()
}
