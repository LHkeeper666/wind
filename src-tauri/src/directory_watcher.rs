use log::{error, info};
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
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};
use windows::core::PCWSTR;

const WATCH_BUFFER_SIZE: u32 = 16384;
const WAIT_TIMEOUT_MS: u32 = 1000;

/// SAFETY: HANDLE is an opaque integer value. We only use it with Win32 APIs
/// that are safe to call from any thread (SetEvent, CloseHandle).
/// The handle is created and destroyed within controlled lifetimes.
#[derive(Clone, Copy)]
struct SendHandle(HANDLE);
unsafe impl Send for SendHandle {}

pub struct DirectoryWatcher {
    stop_flag: Option<Arc<AtomicBool>>,
    event_handle: Option<SendHandle>,
    thread_handle: Option<JoinHandle<()>>,
    done_rx: Option<mpsc::Receiver<()>>,
}

impl DirectoryWatcher {
    pub fn new() -> Self {
        Self {
            stop_flag: None,
            event_handle: None,
            thread_handle: None,
            done_rx: None,
        }
    }

    pub fn start(&mut self, root: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
        // Ensure previous watcher is fully stopped before starting new one
        self.stop();

        let root_path = Path::new(root);
        if !root_path.is_dir() {
            return Err(format!("Not a directory: {}", root));
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();
        let root_string = root.to_string();

        // Create event handle here so stop() can signal it
        let event_handle = SendHandle(create_event().map_err(|e| format!("Failed to create event: {}", e))?);
        let event_for_thread = event_handle;

        // Channel for the watch thread to signal it has exited
        let (done_tx, done_rx) = mpsc::sync_channel::<()>(1);

        let handle = std::thread::spawn(move || {
            watch_loop(&root_string, &stop_clone, &app_handle, event_for_thread);
            let _ = done_tx.send(());
        });

        self.stop_flag = Some(stop_flag);
        self.event_handle = Some(event_handle);
        self.thread_handle = Some(handle);
        self.done_rx = Some(done_rx);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, Ordering::SeqCst);
            // Wake the watch thread if it's blocked in WaitForSingleObject
            if let Some(SendHandle(event)) = self.event_handle.take() {
                unsafe { let _ = SetEvent(event); }
            }
        }
        // Wait for the watch thread to exit with a timeout
        if let Some(done_rx) = self.done_rx.take() {
            if done_rx.recv_timeout(Duration::from_secs(5)).is_err() {
                info!("[directory_watcher] Watch thread did not exit within 5s, waiting on join...");
                if let Some(handle) = self.thread_handle.take() {
                    match handle.join() {
                        Ok(_) => {}
                        Err(e) => { log::error!("[directory_watcher] thread panicked: {:?}", e); }
                    }
                }
            }
        } else if let Some(handle) = self.thread_handle.take() {
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

fn watch_loop(root: &str, stop_flag: &Arc<AtomicBool>, app_handle: &tauri::AppHandle, SendHandle(event_handle): SendHandle) {
    let dir_handle = match open_dir_handle(root) {
        Ok(h) => h,
        Err(e) => {
            error!("[directory_watcher] Failed to open handle for {}: {}", root, e);
            unsafe { let _ = CloseHandle(event_handle); }
            return;
        }
    };

    let mut buffer = vec![0u8; WATCH_BUFFER_SIZE as usize];
    let mut overlap = OVERLAPPED::default();
    overlap.hEvent = event_handle;

    let notify_filter = FILE_NOTIFY_CHANGE_FILE_NAME
        | FILE_NOTIFY_CHANGE_DIR_NAME
        | FILE_NOTIFY_CHANGE_LAST_WRITE;

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        // Submit ReadDirectoryChangesW (overlapped, returns immediately)
        let ok = unsafe {
            ReadDirectoryChangesW(
                dir_handle,
                buffer.as_mut_ptr() as *mut _,
                WATCH_BUFFER_SIZE,
                true, // watch subtree
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

        // Wait for event or timeout
        let wait_result = unsafe { WaitForSingleObject(event_handle, WAIT_TIMEOUT_MS) };

        if wait_result == WAIT_OBJECT_0 {
            // Event signaled — check if stop was requested
            if stop_flag.load(Ordering::SeqCst) {
                unsafe { let _ = CancelIo(dir_handle); }
                break;
            }

            // Check overlapped result (Internal = NTSTATUS, InternalHigh = bytes transferred)
            let status = overlap.Internal as i32;
            let bytes_returned = overlap.InternalHigh as u32;

            if status != 0 {
                info!("[directory_watcher] Overlapped I/O error (status={}), stopping", status);
                break;
            }

            if bytes_returned == 0 {
                // Buffer overflow — discard and resubmit
                info!("[directory_watcher] Buffer overflow, discarding and resubmitting");
                continue;
            }

            // Parse and emit events
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

    // Cleanup
    unsafe {
        let _ = CloseHandle(event_handle);
        let _ = CloseHandle(dir_handle);
    }

    info!("[directory_watcher] Watch thread exiting for {}", root);
}

fn open_dir_handle(path: &str) -> Result<HANDLE, String> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_MODE(FILE_SHARE_DELETE.0 | 2 | 4), // DELETE | READ | WRITE
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

        // Only process meaningful actions
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