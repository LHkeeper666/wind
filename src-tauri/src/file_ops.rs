use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use serde::Serialize;

static FILE_OPS: LazyLock<Mutex<HashMap<u64, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpType {
    Copy,
    Move,
    Delete,
}

pub struct Progress {
    pub op_id: u64,
    pub op_type: OpType,
    bytes_done: u64,
    total_bytes: u64,
    files_done: u32,
    total_files: u32,
    current_file: String,
    cancel_flag: Arc<AtomicBool>,
    last_emit: Instant,
    description: String,
}

impl Progress {
    pub fn new(op_type: OpType, description: String) -> Self {
        let id = NEXT_OP_ID.fetch_add(1, Ordering::Relaxed);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        FILE_OPS.lock().unwrap().insert(id, cancel_flag.clone());
        Progress {
            op_id: id,
            op_type,
            bytes_done: 0,
            total_bytes: 0,
            files_done: 0,
            total_files: 0,
            current_file: String::new(),
            cancel_flag,
            last_emit: Instant::now(),
            description,
        }
    }

    pub fn cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    pub fn set_scan_result(&mut self, total_bytes: u64, total_files: u32) {
        self.total_bytes = total_bytes;
        self.total_files = total_files;
    }

    fn emit_progress(&mut self, app: &AppHandle) {
        let now = Instant::now();
        if now.duration_since(self.last_emit).as_millis() < 100 {
            return;
        }
        self.last_emit = now;
        let _ = app.emit("op-progress", serde_json::json!({
            "id": self.op_id,
            "op_type": self.op_type,
            "bytes_done": self.bytes_done,
            "total_bytes": self.total_bytes,
            "files_done": self.files_done,
            "total_files": self.total_files,
            "current_file": self.current_file,
            "description": self.description,
        }));
    }

    pub fn add_bytes(&mut self, n: u64, current_file: &str, app: &AppHandle) {
        self.bytes_done += n;
        self.current_file = current_file.to_string();
        self.emit_progress(app);
    }

    pub fn file_done(&mut self, app: &AppHandle) {
        self.files_done += 1;
        self.emit_progress(app);
    }
}

/// Copy a single file with chunked I/O, reporting progress.
pub fn copy_file_chunked(
    src: &Path,
    dst: &Path,
    progress: &mut Progress,
    app: &AppHandle,
) -> io::Result<u64> {
    let mut reader = File::open(src)?;
    let mut writer = File::create(dst)?;
    let mut buf = [0u8; 65536];
    let mut written: u64 = 0;

    loop {
        if progress.cancelled() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
        }
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        written += n as u64;
        progress.add_bytes(n as u64, &src.to_string_lossy(), app);
    }

    writer.flush()?;
    Ok(written)
}

/// Scan a directory recursively to compute total bytes and file count.
pub fn scan_directory(root: &Path, max_depth: u32) -> io::Result<(u64, u32)> {
    let mut total_bytes: u64 = 0;
    let mut total_files: u32 = 0;
    scan_dir_recursive(root, root, 0, max_depth, &mut total_bytes, &mut total_files)?;
    Ok((total_bytes, total_files))
}

fn scan_dir_recursive(
    _base: &Path,
    dir: &Path,
    depth: u32,
    max_depth: u32,
    total_bytes: &mut u64,
    total_files: &mut u32,
) -> io::Result<()> {
    if depth > max_depth {
        return Ok(());
    }
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(_base, &path, depth + 1, max_depth, total_bytes, total_files)?;
        } else if let Ok(meta) = entry.metadata() {
            *total_bytes += meta.len();
            *total_files += 1;
        }
    }
    Ok(())
}

/// Copy a directory recursively with progress.
pub fn copy_dir_chunked(
    src: &Path,
    dst: &Path,
    progress: &mut Progress,
    app: &AppHandle,
) -> io::Result<u64> {
    let mut total_copied: u64 = 0;

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    let entries = fs::read_dir(src)?;
    for entry in entries {
        if progress.cancelled() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
        }
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            total_copied += copy_dir_chunked(&path, &dst_path, progress, app)?;
        } else {
            total_copied += copy_file_chunked(&path, &dst_path, progress, app)?;
            progress.file_done(app);
        }
    }

    Ok(total_copied)
}

/// Delete a file or directory recursively with progress.
pub fn delete_recursive(
    path: &Path,
    progress: &mut Progress,
    app: &AppHandle,
) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            if progress.cancelled() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
            }
            let entry = entry?;
            delete_recursive(&entry.path(), progress, app)?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
        progress.file_done(app);
    }

    Ok(())
}

/// Check for conflicts: which destination files already exist.
pub fn check_conflicts(
    sources: &[(PathBuf, String)],
    dest_dir: &Path,
) -> Vec<String> {
    let mut conflicts = Vec::new();
    for (src_path, _src_name) in sources {
        let file_name = src_path.file_name().unwrap_or_default();
        let dest_path = dest_dir.join(file_name);
        if dest_path.exists() {
            conflicts.push(file_name.to_string_lossy().to_string());
        }
    }
    conflicts
}

// Public API for commands

pub fn cancel_op(id: u64) -> bool {
    if let Some(flag) = FILE_OPS.lock().unwrap().get(&id) {
        flag.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

fn cleanup_op(id: u64) {
    FILE_OPS.lock().unwrap().remove(&id);
}

pub fn emit_scan_complete(id: u64, total_bytes: u64, total_files: u32, app: &AppHandle) {
    let _ = app.emit("op-scan-complete", serde_json::json!({
        "id": id,
        "total_bytes": total_bytes,
        "total_files": total_files,
    }));
}

pub fn emit_complete(id: u64, app: &AppHandle) {
    cleanup_op(id);
    let _ = app.emit("op-complete", serde_json::json!({ "id": id }));
}

pub fn emit_failed(id: u64, error: &str, app: &AppHandle) {
    cleanup_op(id);
    let _ = app.emit("op-failed", serde_json::json!({ "id": id, "error": error }));
}

pub fn emit_cancelled(id: u64, app: &AppHandle) {
    cleanup_op(id);
    let _ = app.emit("op-cancelled", serde_json::json!({ "id": id }));
}
