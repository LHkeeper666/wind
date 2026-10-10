use log::{debug, error, info};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use tauri::Emitter;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_ABANDONED_0, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadDirectoryChangesW, FILE_ACTION_ADDED, FILE_ACTION_MODIFIED,
    FILE_ACTION_REMOVED, FILE_ACTION_RENAMED_NEW_NAME, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME,
    FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_INFORMATION, FILE_SHARE_DELETE,
    FILE_SHARE_MODE, OPEN_EXISTING,
};
use windows::Win32::System::IO::{CancelIo, OVERLAPPED};
use windows::Win32::System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForMultipleObjects, WaitForSingleObject};
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
    stop_event: Option<SendHandle>,
    thread_handle: Option<JoinHandle<()>>,
    done_rx: Option<mpsc::Receiver<()>>,
}

impl DirectoryWatcher {
    pub fn new() -> Self {
        Self {
            stop_event: None,
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

        // Create a dedicated stop event — separate from the I/O event used by ReadDirectoryChangesW
        let stop_event = SendHandle(create_event().map_err(|e| format!("Failed to create stop event: {}", e))?);
        let stop_event_for_thread = stop_event;

        // Channel for the watch thread to signal it has exited
        let (done_tx, done_rx) = mpsc::sync_channel::<()>(1);

        let handle = std::thread::spawn(move || {
            watch_loop(&root_string, &app_handle, stop_event_for_thread);
            let _ = done_tx.send(());
        });

        self.stop_event = Some(stop_event);
        self.thread_handle = Some(handle);
        self.done_rx = Some(done_rx);
        Ok(())
    }

    pub fn stop(&mut self) {
        // Signal the dedicated stop event to wake the watch thread
        if let Some(SendHandle(event)) = self.stop_event.take() {
            debug!("[directory_watcher] stop(): signaling stop_event={:?}", event);
            unsafe { let _ = SetEvent(event); }
        } else {
            debug!("[directory_watcher] stop(): no stop_event to signal");
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

fn watch_loop(root: &str, app_handle: &tauri::AppHandle, SendHandle(stop_event): SendHandle) {
    debug!("[directory_watcher] watch_loop starting for root={} stop_event={:?}", root, stop_event);

    let dir_handle = match open_dir_handle(root) {
        Ok(h) => h,
        Err(e) => {
            error!("[directory_watcher] Failed to open handle for {}: {}", root, e);
            unsafe { let _ = CloseHandle(stop_event); }
            return;
        }
    };
    debug!("[directory_watcher] dir_handle={:?}", dir_handle);

    // Create a dedicated I/O completion event — separate from the stop event
    let io_event = match create_event() {
        Ok(h) => h,
        Err(e) => {
            error!("[directory_watcher] Failed to create I/O event: {}", e);
            unsafe {
                let _ = CloseHandle(dir_handle);
                let _ = CloseHandle(stop_event);
            }
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

    // Wait on both: index 0 = I/O event, index 1 = stop event
    let wait_handles = [io_event, stop_event];
    let mut iteration: u64 = 0;

    loop {
        iteration += 1;
        debug!("[directory_watcher] loop iter={} submitting ReadDirectoryChangesW", iteration);

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

        // Wait for I/O completion, stop signal, or timeout
        debug!("[directory_watcher] loop iter={} entering WaitForMultipleObjects", iteration);
        let wait_result = unsafe {
            WaitForMultipleObjects(&wait_handles, false, WAIT_TIMEOUT_MS)
        };
        debug!("[directory_watcher] loop iter={} WaitForMultipleObjects returned 0x{:08X}", iteration, wait_result.0);

        if wait_result.0 >= WAIT_OBJECT_0.0 && wait_result.0 < WAIT_ABANDONED_0.0 {
            let index = (wait_result.0 - WAIT_OBJECT_0.0) as u32;

            if index == 1 {
                // Stop event signaled
                debug!("[directory_watcher] loop iter={} stop event signaled, canceling I/O", iteration);
                unsafe { let _ = CancelIo(dir_handle); }
                info!("[directory_watcher] Stop signaled, exiting");
                break;
            }

            // index == 0: I/O event signaled — reset it so WaitForMultipleObjects can block again
            unsafe { let _ = ResetEvent(io_event); }

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
                debug!("[directory_watcher] loop iter={} emitting {} changed paths", iteration, filtered.len());
                if let Err(e) = app_handle.emit("directory-changed", filtered) {
                    log::debug!("[emit] directory-changed failed: {}", e);
                }
            }

            // Safety: check stop_event after processing I/O (in case both events fired simultaneously)
            unsafe {
                if WaitForSingleObject(stop_event, 0) == WAIT_OBJECT_0 {
                    debug!("[directory_watcher] loop iter={} stop event detected after I/O, canceling", iteration);
                    let _ = CancelIo(dir_handle);
                    info!("[directory_watcher] Stop signaled (post-I/O check), exiting");
                    break;
                }
            }
        } else {
            // Unexpected return value (WAIT_FAILED, WAIT_ABANDONED, etc.)
            error!(
                "[directory_watcher] WaitForMultipleObjects unexpected result=0x{:08X}, io_event={:?}, stop_event={:?}",
                wait_result.0, io_event, stop_event
            );
            break;
        }
        // Timeout: loop back to resubmit
    }

    // Cleanup
    debug!("[directory_watcher] cleaning up handles: io_event={:?} dir_handle={:?} stop_event={:?}", io_event, dir_handle, stop_event);
    unsafe {
        let _ = CloseHandle(io_event);
        let _ = CloseHandle(dir_handle);
        let _ = CloseHandle(stop_event);
    }

    info!("[directory_watcher] Watch thread exiting for {} (ran {} iterations)", root, iteration);
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