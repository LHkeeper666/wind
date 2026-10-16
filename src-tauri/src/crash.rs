//! Crash forensics: keep fatal-path evidence inside the app's own log directory
//! even when the process dies without unwinding (a fail-fast abort such as
//! `0xC0000409`). See `docs/crash-forensics.md` for the artifact layout and how
//! to tell an abnormal termination from a normal shutdown.

use std::path::PathBuf;
use std::time::Duration;

/// Heartbeat cadence. One line per interval; a missing `[shutdown]` line (rather
/// than a time gap) is what marks a run as abnormally terminated, so this value
/// only needs to be short enough to show the process was alive recently.
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// `%LOCALAPPDATA%/<identifier>/logs` — the same directory the regular file
/// logger resolves through `AppHandle::path().app_local_data_dir()`.
pub fn log_dir(identifier: &str) -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join(identifier).join("logs"))
}

/// Crash log artifact for the day the process *started*. The stderr handle is
/// fixed for the process lifetime, so a long-running instance keeps appending to
/// the file it opened at startup even across midnight.
pub fn crash_log_path(identifier: &str) -> Option<PathBuf> {
    let today = chrono::Local::now().format("%Y-%m-%d");
    log_dir(identifier).map(|dir| dir.join(format!("wind-crash-{}.log", today)))
}

/// Send the process's stderr to the crash log. Returns a note describing the
/// outcome so the real logger can record it once it is up.
///
/// Must run before anything writes to stderr: Rust std and the MSVC CRT both
/// resolve `STD_ERROR_HANDLE` on first use and cache it, so a later redirect is
/// silently ignored. Dev builds keep the console and are left alone.
pub fn init_stderr_redirect(identifier: &str) -> String {
    if cfg!(debug_assertions) {
        return "[crash] dev build: stderr left on the console".to_string();
    }
    #[cfg(windows)]
    {
        redirect_stderr_windows(identifier)
    }
    #[cfg(not(windows))]
    {
        let _ = identifier;
        "[crash] stderr redirect not implemented on this platform".to_string()
    }
}

#[cfg(windows)]
fn redirect_stderr_windows(identifier: &str) -> String {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{SetStdHandle, STD_ERROR_HANDLE};

    let Some(path) = crash_log_path(identifier) else {
        return "[crash] stderr redirect skipped: no local data directory".to_string();
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return format!("[crash] stderr redirect failed: create {}: {e}", parent.display());
        }
    }

    let file = match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(e) => return format!("[crash] stderr redirect failed: open {}: {e}", path.display()),
    };

    let handle = HANDLE(file.as_raw_handle());
    if let Err(e) = unsafe { SetStdHandle(STD_ERROR_HANDLE, handle) } {
        return format!("[crash] stderr redirect failed: SetStdHandle: {e}");
    }

    // Keep the handle open for the whole process; closing it would leave
    // STD_ERROR_HANDLE dangling.
    std::mem::forget(file);

    // Write through stderr (not the file directly) so this line proves the
    // redirect took effect, and leaves the crash log non-empty even if the
    // failing process never writes anything itself.
    eprintln!(
        "[crash] stderr redirected at {} -> {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        path.display()
    );

    format!("[crash] stderr redirected to {}", path.display())
}

/// Log a panic (message, location, and — only when requested — a backtrace)
/// before handing off to the previous hook, so exit-code semantics are unchanged.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown location>".to_string());

        if backtrace_requested() {
            log::error!(
                target: "crash",
                "panic: {} at {}\n{}",
                message,
                location,
                std::backtrace::Backtrace::force_capture()
            );
        } else {
            log::error!(target: "crash", "panic: {} at {}", message, location);
        }

        previous(info);
    }));
}

fn backtrace_requested() -> bool {
    ["RUST_BACKTRACE", "RUST_LIB_BACKTRACE"].iter().any(|key| {
        std::env::var(key)
            .map(|v| !v.is_empty() && v != "0")
            .unwrap_or(false)
    })
}

/// One line every [`HEARTBEAT_INTERVAL`], from a plain thread so it survives an
/// async-runtime failure.
pub fn spawn_heartbeat() {
    let _ = std::thread::Builder::new()
        .name("crash-heartbeat".into())
        .spawn(|| loop {
            std::thread::sleep(HEARTBEAT_INTERVAL);
            log::info!(target: "crash", "[heartbeat] alive");
        });
}

/// A delimiter line plus version/build/level/memory, so multiple runs appended
/// to one file are separable at a glance.
pub fn log_startup_banner(level: log::LevelFilter) {
    let separator = "=".repeat(72);
    log::info!(target: "crash", "{separator}");
    log::info!(
        target: "crash",
        "wind run start  version={} build={} log_level={}",
        env!("CARGO_PKG_VERSION"),
        build_time(),
        format!("{level:?}").to_lowercase()
    );
    match memory_info() {
        Some((physical, pagefile)) => log::info!(
            target: "crash",
            "memory: physical={} MiB pagefile={} MiB",
            physical / (1024 * 1024),
            pagefile / (1024 * 1024)
        ),
        None => log::info!(target: "crash", "memory: unavailable"),
    }
    log::info!(target: "crash", "{separator}");
}

fn build_time() -> String {
    match env!("WIND_BUILD_EPOCH").parse::<i64>() {
        Ok(secs) => chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "unknown".to_string()),
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(windows)]
fn memory_info() -> Option<(u64, u64)> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) }.is_ok() {
        Some((status.ullTotalPhys, status.ullTotalPageFile))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn memory_info() -> Option<(u64, u64)> {
    None
}
