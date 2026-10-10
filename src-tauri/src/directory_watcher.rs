use log::{debug, error, info};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;
use tauri::Emitter;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadDirectoryChangesW, FILE_ACTION_ADDED, FILE_ACTION_MODIFIED,
    FILE_ACTION_REMOVED, FILE_ACTION_RENAMED_NEW_NAME, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME,
    FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_INFORMATION, FILE_SHARE_DELETE,
    FILE_SHARE_MODE, OPEN_EXISTING,
};
use windows::Win32::System::IO::{CancelIo, OVERLAPPED};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::PCWSTR;

const WATCH_BUFFER_SIZE: u32 = 16384;
const POLL_TIMEOUT_MS: u32 = 50;

pub struct DirectoryWatcher {
    stop_flag: Option<Arc<AtomicBool>>,
    thread_handle: Option<JoinHandle<()>>,
    done_rx: Option<mpsc::Receiver<()>>,
}

impl DirectoryWatcher {
    pub fn new() -> Self {
        Self {
            stop_flag: None,
            thread_handle: None,
            done_rx: None,
        }
    }

    pub fn start(&mut self, root: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
        info!("[directory_watcher] Starting watcher for {}", root);
        // Ensure previous watcher is fully stopped before starting new one
        self.stop();

        let root_path = Path::new(root);
        if !root_path.is_dir() {
            return Err(format!("Not a directory: {}", root));
        }

        let root_string = root.to_string();

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();

        // Channel for the watch thread to signal it has exited
        let (done_tx, done_rx) = mpsc::sync_channel::<()>(1);

        let handle = std::thread::spawn(move || {
            watch_loop(&root_string, &app_handle, &stop_clone);
            let _ = done_tx.send(());
        });

        self.stop_flag = Some(stop_flag);
        self.thread_handle = Some(handle);
        self.done_rx = Some(done_rx);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            debug!("[directory_watcher] stop(): setting stop_flag");
            flag.store(true, Ordering::SeqCst);
        } else {
            debug!("[directory_watcher] stop(): no stop_flag to set");
        }
        // Wait for the watch thread to exit with a timeout
        if let Some(done_rx) = self.done_rx.take() {
            debug!("[directory_watcher] stop(): waiting for watch thread (5s timeout)...");
            if done_rx.recv_timeout(Duration::from_secs(5)).is_err() {
                info!("[directory_watcher] Watch thread did not exit within 5s, waiting on join...");
                if let Some(handle) = self.thread_handle.take() {
                    match handle.join() {
                        Ok(_) => { info!("[directory_watcher] stop(): join completed after timeout"); }
                        Err(e) => { log::error!("[directory_watcher] thread panicked: {:?}", e); }
                    }
                }
            } else {
                debug!("[directory_watcher] stop(): watch thread exited cleanly");
            }
        } else if let Some(handle) = self.thread_handle.take() {
            debug!("[directory_watcher] stop(): no done_rx, joining directly...");
            match handle.join() {
                Ok(_) => {}
                Err(e) => { log::error!("[directory_watcher] thread panicked: {:?}", e); }
            }
        }
    }
}

impl Drop for DirectoryWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

fn watch_loop(root: &str, app_handle: &tauri::AppHandle, stop_flag: &Arc<AtomicBool>) {
    debug!("[directory_watcher] watch_loop starting for root={}", root);

    let dir_handle = match open_dir_handle(root) {
        Ok(h) => h,
        Err(e) => {
            error!("[directory_watcher] Failed to open handle for {}: {}", root, e);
            return;
        }
    };
    debug!("[directory_watcher] dir_handle={:?}", dir_handle);

    let io_event = match create_event() {
        Ok(h) => h,
        Err(e) => {
            error!("[directory_watcher] Failed to create I/O event: {}", e);
            unsafe { let _ = CloseHandle(dir_handle); }
            return;
        }
    };
    debug!("[directory_watcher] io_event={:?}", io_event);

    let mut buffer = vec![0u8; WATCH_BUFFER_SIZE as usize];
    let mut overlap = OVERLAPPED::default();
    overlap.hEvent = io_event;

    let notify_filter = FILE_NOTIFY_CHANGE_FILE_NAME
        | FILE_NOTIFY_CHANGE_DIR_NAME
        | FILE_NOTIFY_CHANGE_LAST_WRITE;

    let mut iteration: u64 = 0;

    loop {
        iteration += 1;

        // Check stop flag before submitting I/O
        if stop_flag.load(Ordering::SeqCst) {
            debug!("[directory_watcher] loop iter={} stop_flag detected before I/O submit", iteration);
            break;
        }

        let ok = unsafe {
            ReadDirectoryChangesW(
                dir_handle,
                buffer.as_mut_ptr() as *mut _,
                WATCH_BUFFER_SIZE,
                true,
                notify_filter,
                None,
                Some(&mut overlap),
                None,
            )
        };

        if let Err(e) = ok {
            error!("[directory_watcher] ReadDirectoryChangesW failed: {}", e);
            break;
        }

        // Wait for I/O completion or short timeout — check stop_flag after each wake
        let wait_result = unsafe { WaitForSingleObject(io_event, POLL_TIMEOUT_MS) };

        if stop_flag.load(Ordering::SeqCst) {
            debug!("[directory_watcher] loop iter={} stop_flag detected after wait", iteration);
            unsafe { let _ = CancelIo(dir_handle); }
            break;
        }

        if wait_result == WAIT_OBJECT_0 {
            // I/O event signaled — reset it
            let status = overlap.Internal as i32;
            let bytes_returned = overlap.InternalHigh as u32;

            if status != 0 {
                info!("[directory_watcher] Overlapped I/O error (status={}), stopping", status);
                break;
            }

            if bytes_returned == 0 {
                continue;
            }

            let paths = parse_notify_buffer(&buffer, bytes_returned, root);
            let filtered: Vec<String> = paths
                .into_iter()
                .filter(|p| !is_filtered_path(p))
                .collect();

            if !filtered.is_empty() {
                if let Err(e) = app_handle.emit("directory-changed", filtered) {
                    log::debug!("[emit] directory-changed failed: {}", e);
                }
            }
        }
        // Timeout: loop back to check stop_flag and resubmit
    }

    // Cleanup — only close handles owned by this thread
    unsafe {
        let _ = CloseHandle(io_event);
        let _ = CloseHandle(dir_handle);
    }

    info!("[directory_watcher] Watch thread exiting for {} (ran {} iterations)", root, iteration);
}

fn open_dir_handle(path: &str) -> Result<HANDLE, String> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_MODE(FILE_SHARE_DELETE.0 | 2 | 4),
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }
    .map_err(|e| format!("CreateFileW failed: {}", e))?;

    Ok(handle)
}

fn create_event() -> Result<HANDLE, String> {
    let handle = unsafe { CreateEventW(None, true, false, None) }
        .map_err(|e| format!("CreateEventW failed: {}", e))?;
    Ok(handle)
}

fn parse_notify_buffer(buffer: &[u8], bytes_returned: u32, root: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut offset = 0usize;
    let limit = bytes_returned as usize;

    loop {
        if offset + std::mem::size_of::<FILE_NOTIFY_INFORMATION>() > limit {
            break;
        }

        let info_ptr = unsafe { buffer.as_ptr().add(offset) as *const FILE_NOTIFY_INFORMATION };
        let info = unsafe { &*info_ptr };

        if matches!(
            info.Action,
            FILE_ACTION_ADDED | FILE_ACTION_REMOVED | FILE_ACTION_MODIFIED | FILE_ACTION_RENAMED_NEW_NAME
        ) {
            let name_bytes = unsafe {
                std::slice::from_raw_parts(
                    info.FileName.as_ptr(),
                    (info.FileNameLength / 2) as usize,
                )
            };
            let relative = OsString::from_wide(name_bytes)
                .to_string_lossy()
                .to_string();

            let full_path = format!("{}\\{}", root.trim_end_matches('\\'), relative);
            paths.push(full_path);
        }

        if info.NextEntryOffset == 0 {
            break;
        }
        offset += info.NextEntryOffset as usize;
    }

    paths
}

fn is_filtered_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.split('\\').any(|part| matches!(part, ".git" | "target" | "node_modules"))
}