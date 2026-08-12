mod terminal;
mod neovim;
mod pdf;
mod video;
mod file_ops;
mod file_watcher;
mod ftp;
mod python_completion;
mod transfer;

use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use std::fs::File;
use tauri::{Emitter, Manager, State};
use windows::Win32::UI::Input::Ime::{ImmGetContext, ImmGetOpenStatus, ImmReleaseContext, ImmSetOpenStatus};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

static SEARCH_CANCELLED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
async fn list_virtual_root(state: State<'_, AppState>) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    // Local drives
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        if Path::new(&drive).exists() {
            entries.push(FileEntry {
                name: drive.clone(),
                path: drive,
                is_dir: true,
                size: None,
                is_hidden: false,
                modified: None,
                created: None,
                children: None,
            });
        }
    }

    // FTP connections
    let mgr = state.ftp_manager.lock().await;
    for config in mgr.get_configs() {
        entries.push(FileEntry {
            name: format!("[FTP] {}", config.name),
            path: format!("ftp://{}/", config.name),
            is_dir: true,
            size: None,
            is_hidden: false,
            modified: None,
            created: None,
            children: None,
        });
    }

    Ok(entries)
}

// Legacy alias — called by frontend as `invoke('list_drives')`.
#[tauri::command]
async fn list_drives(state: State<'_, AppState>) -> Result<Vec<FileEntry>, String> {
    list_virtual_root(state).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    is_hidden: bool,
    modified: Option<u64>,
    created: Option<u64>,
    children: Option<Vec<FileEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

struct AppState {
    terminal: terminal::TerminalManager,
    neovim: Mutex<neovim::Neovim>,
    file_watcher: Mutex<file_watcher::FileWatcher>,
    ftp_manager: Arc<TokioMutex<ftp::FtpManager>>,
    transfer_scheduler: Arc<TokioMutex<transfer::TransferScheduler>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_home_dir() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\".to_string())
}

#[tauri::command]
fn file_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[derive(Serialize)]
struct FileMetadata {
    size: u64,
    modified: u64,
}

#[tauri::command]
fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let modified = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(FileMetadata {
        size: metadata.len(),
        modified,
    })
}

#[tauri::command]
fn start_watch_file(path: String, app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.file_watcher.lock().map_err(|e| e.to_string())?.start(&path, app_handle);
    Ok(())
}

#[tauri::command]
fn stop_watch_file(state: State<'_, AppState>) -> Result<(), String> {
    state.file_watcher.lock().map_err(|e| e.to_string())?.stop();
    Ok(())
}

#[tauri::command]
async fn read_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    // Route FTP paths
    if path.starts_with("ftp://") {
        eprintln!("[FTP] read_directory routing: FTP path — {}", path);
        return ftp_read_directory(path, state).await;
    }
    // Route virtual root
    if path == "\\" {
        eprintln!("[FTP] read_directory routing: virtual root");
        return list_virtual_root(state).await;
    }

    // 规范化驱动器路径：D: → D:\
    let normalized = if path.len() == 2 && path.ends_with(':') {
        format!("{}\\", path)
    } else {
        path.clone()
    };
    let dir_path = Path::new(&normalized);

    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut entries = Vec::new();

    match fs::read_dir(dir_path) {
        Ok(read_dir) => {
            for entry in read_dir {
                match entry {
                    Ok(entry) => {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        let file_path = entry.path().to_string_lossy().to_string();
                        let is_dir = entry.path().is_dir();
                        let size = if is_dir {
                            None
                        } else {
                            entry.metadata().ok().map(|m| m.len())
                        };
                        let is_hidden = file_name.starts_with('.')
                            || entry.metadata().map(|m| {
                                use std::os::windows::fs::MetadataExt;
                                m.file_attributes() & 0x2 != 0  // FILE_ATTRIBUTE_HIDDEN
                            }).unwrap_or(false);

                        let modified = entry.metadata().ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs());
                        let created = entry.metadata().ok()
                            .and_then(|m| m.created().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs());

                        entries.push(FileEntry {
                            name: file_name,
                            path: file_path,
                            is_dir,
                            size,
                            is_hidden,
                            modified,
                            created,
                            children: None,
                        });
                    }
                    Err(e) => {
                        eprintln!("Error reading entry: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read directory: {}", e));
        }
    }

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

#[tauri::command]
fn list_archive_entries(path: String) -> Result<Vec<ArchiveEntry>, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;

    let mut files = Vec::new();
    let mut dir_set = std::collections::HashSet::new();

    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string());

        if entry.is_dir() {
            continue;
        }

        // Collect parent directories from file paths
        let mut parent = std::path::Path::new(&entry_path).parent();
        while let Some(p) = parent {
            if p.as_os_str().is_empty() {
                break;
            }
            dir_set.insert(p.to_string_lossy().to_string());
            parent = p.parent();
        }

        let name = std::path::Path::new(&entry_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| entry_path.clone());
        let size = entry.size();

        files.push(ArchiveEntry {
            name,
            path: entry_path,
            is_dir: false,
            size,
        });
    }

    let mut entries: Vec<ArchiveEntry> = dir_set
        .into_iter()
        .map(|dir_path| {
            let name = std::path::Path::new(&dir_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| dir_path.clone());
            ArchiveEntry {
                name,
                path: dir_path,
                is_dir: true,
                size: 0,
            }
        })
        .collect();
    entries.append(&mut files);

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.path.to_lowercase().cmp(&b.path.to_lowercase())
        }
    });

    Ok(entries)
}

#[tauri::command]
fn delete_file(path: String) -> Result<(), String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    trash::delete(file_path)
        .map_err(|e| format!("Failed to move to trash: {}", e))
}

#[tauri::command]
fn permanent_delete(path: String) -> Result<(), String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if file_path.is_dir() {
        fs::remove_dir_all(file_path)
            .map_err(|e| format!("Failed to permanently delete directory: {}", e))
    } else {
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to permanently delete file: {}", e))
    }
}

#[tauri::command]
fn rename_file(old_path: String, new_name: String) -> Result<String, String> {
    let old = Path::new(&old_path);

    if !old.exists() {
        return Err(format!("Path does not exist: {}", old_path));
    }

    let parent = old.parent()
        .ok_or_else(|| "Cannot get parent directory".to_string())?;

    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return Err(format!("A file or directory with name '{}' already exists", new_name));
    }

    fs::rename(old, &new_path)
        .map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(new_path.to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameEntry {
    old_path: String,
    new_name: String,
}

#[tauri::command]
fn batch_rename(entries: Vec<RenameEntry>) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut renamed = Vec::new();

    for entry in &entries {
        let old = Path::new(&entry.old_path);
        if !old.exists() {
            errors.push(format!("{}: file not found", entry.old_path));
            continue;
        }

        let parent = match old.parent() {
            Some(p) => p,
            None => {
                errors.push(format!("{}: cannot get parent", entry.old_path));
                continue;
            }
        };

        let new_path = parent.join(&entry.new_name);
        if new_path.exists() {
            errors.push(format!("{}: destination already exists", entry.new_name));
            continue;
        }

        match fs::rename(old, &new_path) {
            Ok(()) => renamed.push(entry.old_path.clone()),
            Err(e) => errors.push(format!("{}: {}", entry.old_path, e)),
        }
    }

    if errors.is_empty() {
        Ok(renamed)
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
fn create_batch_rename_temp_file(files: Vec<String>) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("wind_batch_rename.txt");
    fs::write(&temp_file, files.join("\n"))
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    Ok(temp_file.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_temp_file(path: String) -> Result<(), String> {
    let _ = fs::remove_file(path);
    Ok(())
}

#[tauri::command]
fn create_file(path: String, is_dir: bool) -> Result<(), String> {
    let file_path = Path::new(&path);

    if file_path.exists() {
        return Err(format!("A file or directory already exists at: {}", path));
    }

    if is_dir {
        fs::create_dir_all(file_path)
            .map_err(|e| format!("Failed to create directory: {}", e))
    } else {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directories: {}", e))?;
            }
        }
        fs::write(file_path, "")
            .map_err(|e| format!("Failed to create file: {}", e))
    }
}

#[tauri::command]
fn copy_file(source: String, destination: String) -> Result<(), String> {
    let src = Path::new(&source);
    let dst = Path::new(&destination);

    if !src.exists() {
        return Err(format!("Source path does not exist: {}", source));
    }

    if dst.exists() {
        return Err(format!("Destination already exists: {}", destination));
    }

    // Ensure destination parent directory exists
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create destination directory {}: {}", parent.display(), e))?;
        }
    }

    if src.is_dir() {
        // Copy directory recursively
        copy_dir_recursive(src, dst)
            .map_err(|e| format!("Failed to copy directory: {}", e))
    } else {
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy {} -> {}: {}", source, destination, e))?;
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

#[tauri::command]
fn move_file(source: String, destination: String) -> Result<(), String> {
    let src = Path::new(&source);
    let dst = Path::new(&destination);

    if !src.exists() {
        return Err(format!("Source path does not exist: {}", source));
    }

    if dst.exists() {
        return Err(format!("Destination already exists: {}", destination));
    }

    // Ensure destination parent directory exists
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create destination directory {}: {}", parent.display(), e))?;
        }
    }

    // Same drive: atomic rename. Cross drive: copy + delete.
    let src_drive = source.chars().next().map(|c| c.to_ascii_uppercase());
    let dst_drive = destination.chars().next().map(|c| c.to_ascii_uppercase());

    if src_drive == dst_drive {
        fs::rename(src, dst).map_err(|e| format!("Failed to move {} -> {}: {}", source, destination, e))
    } else {
        // Cross-drive: copy then delete
        if src.is_dir() {
            copy_dir_recursive(src, dst)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
            fs::remove_dir_all(src)
                .map_err(|e| format!("Failed to remove source directory: {}", e))
        } else {
            fs::copy(src, dst)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            fs::remove_file(src)
                .map_err(|e| format!("Failed to remove source file: {}", e))
        }
    }
}

// ── Async file operations ──

#[tauri::command]
async fn check_copy_conflicts(
    sources: Vec<String>,
    dest_dir: String,
) -> Result<Vec<String>, String> {
    let pairs: Vec<(std::path::PathBuf, String)> = sources
        .iter()
        .map(|s| (std::path::PathBuf::from(s), String::new()))
        .collect();
    let dest = std::path::Path::new(&dest_dir);
    Ok(file_ops::check_conflicts(&pairs, dest))
}

#[tauri::command]
async fn copy_file_async(
    app: tauri::AppHandle,
    sources: Vec<String>,
    dest_dir: String,
) -> Result<u64, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let mut progress = file_ops::Progress::new(
            file_ops::OpType::Copy,
            format!("Copying {} items", sources.len()),
        );
        let op_id = progress.op_id;
        let dest_path = std::path::Path::new(&dest_dir);

        // Ensure destination directory exists
        if !dest_path.exists() {
            std::fs::create_dir_all(dest_path)
                .map_err(|e| format!("Failed to create dest dir: {}", e))?;
        }

        // Phase 1: scan
        let mut total_bytes: u64 = 0;
        let mut total_files: u32 = 0;
        for src in &sources {
            let src_path = std::path::Path::new(src);
            if src_path.is_dir() {
                if let Ok((b, f)) = file_ops::scan_directory(src_path, 20) {
                    total_bytes += b;
                    total_files += f;
                }
            } else if let Ok(meta) = src_path.metadata() {
                total_bytes += meta.len();
                total_files += 1;
            }
        }
        progress.set_scan_result(total_bytes, total_files);
        file_ops::emit_scan_complete(op_id, total_bytes, total_files, &app_clone);

        // Phase 2: execute
        let mut total_copied: u64 = 0;
        for src in &sources {
            if progress.cancelled() {
                file_ops::emit_cancelled(op_id, &app_clone);
                return Err("Cancelled".into());
            }
            let src_path = std::path::Path::new(src);
            let file_name = src_path.file_name().unwrap_or_default();
            let dst_path = dest_path.join(file_name);

            if src_path.is_dir() {
                match file_ops::copy_dir_chunked(src_path, &dst_path, &mut progress, &app_clone) {
                    Ok(n) => total_copied += n,
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::Interrupted {
                            file_ops::emit_cancelled(op_id, &app_clone);
                            return Err("Cancelled".into());
                        }
                        let _ = std::fs::remove_dir_all(&dst_path);
                        file_ops::emit_failed(op_id, &e.to_string(), &app_clone);
                        return Err(e.to_string());
                    }
                }
            } else {
                match file_ops::copy_file_chunked(src_path, &dst_path, &mut progress, &app_clone) {
                    Ok(n) => {
                        total_copied += n;
                        progress.file_done(&app_clone);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::Interrupted {
                            file_ops::emit_cancelled(op_id, &app_clone);
                            return Err("Cancelled".into());
                        }
                        let _ = std::fs::remove_file(&dst_path);
                        file_ops::emit_failed(op_id, &e.to_string(), &app_clone);
                        return Err(e.to_string());
                    }
                }
            }
        }

        file_ops::emit_complete(op_id, &app_clone);
        Ok(total_copied)
    })
    .await
    .map_err(|e| format!("Join error: {}", e))?
}

#[tauri::command]
async fn move_file_async(
    app: tauri::AppHandle,
    source: String,
    destination: String,
) -> Result<(), String> {
    let src_drive = source.chars().next().map(|c| c.to_ascii_uppercase());
    let dst_drive = destination.chars().next().map(|c| c.to_ascii_uppercase());

    if src_drive == dst_drive {
        std::fs::rename(&source, &destination)
            .map_err(|e| format!("Failed to move: {}", e))
    } else {
        let app_clone = app.clone();
        tokio::task::spawn_blocking(move || {
            let mut progress = file_ops::Progress::new(
                file_ops::OpType::Move,
                format!("Moving {}", source),
            );
            let op_id = progress.op_id;
            let src_path = std::path::Path::new(&source);
            let dst_path = std::path::Path::new(&destination);

            let (total_bytes, total_files) = if src_path.is_dir() {
                file_ops::scan_directory(src_path, 20).unwrap_or((0, 0))
            } else {
                (src_path.metadata().map(|m| m.len()).unwrap_or(0), 1)
            };
            progress.set_scan_result(total_bytes, total_files);
            file_ops::emit_scan_complete(op_id, total_bytes, total_files, &app_clone);

            let result = if src_path.is_dir() {
                file_ops::copy_dir_chunked(src_path, dst_path, &mut progress, &app_clone).map(|_| ())
            } else {
                file_ops::copy_file_chunked(src_path, dst_path, &mut progress, &app_clone).map(|_| ())
            };

            match result {
                Ok(()) => {
                    let _ = if src_path.is_dir() {
                        std::fs::remove_dir_all(src_path)
                    } else {
                        std::fs::remove_file(src_path)
                    };
                    file_ops::emit_complete(op_id, &app_clone);
                    Ok(())
                }
                Err(e) => {
                    let _ = if dst_path.is_dir() {
                        std::fs::remove_dir_all(dst_path)
                    } else {
                        std::fs::remove_file(dst_path)
                    };
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        file_ops::emit_cancelled(op_id, &app_clone);
                    } else {
                        file_ops::emit_failed(op_id, &e.to_string(), &app_clone);
                    }
                    Err(e.to_string())
                }
            }
        })
        .await
        .map_err(|e| format!("Join error: {}", e))?
    }
}

#[tauri::command]
async fn delete_file_async(
    app: tauri::AppHandle,
    path: String,
    permanent: Option<bool>,
) -> Result<(), String> {
    let is_permanent = permanent.unwrap_or(false);
    let app_clone = app.clone();
    let path_clone = path.clone();

    tokio::task::spawn_blocking(move || {
        let mut progress = file_ops::Progress::new(
            file_ops::OpType::Delete,
            format!("Deleting {}", path_clone),
        );
        let op_id = progress.op_id;
        let p = std::path::Path::new(&path_clone);

        let (total_bytes, total_files) = if p.is_dir() {
            file_ops::scan_directory(p, 20).unwrap_or((0, 0))
        } else {
            (p.metadata().map(|m| m.len()).unwrap_or(0), 1)
        };
        progress.set_scan_result(total_bytes, total_files);
        file_ops::emit_scan_complete(op_id, total_bytes, total_files, &app_clone);

        let result = if is_permanent {
            file_ops::delete_recursive(p, &mut progress, &app_clone)
        } else {
            match trash::delete(p) {
                Ok(()) => {
                    progress.file_done(&app_clone);
                    Ok(())
                }
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
            }
        };

        match result {
            Ok(()) => {
                file_ops::emit_complete(op_id, &app_clone);
                Ok(())
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    file_ops::emit_cancelled(op_id, &app_clone);
                } else {
                    file_ops::emit_failed(op_id, &e.to_string(), &app_clone);
                }
                Err(e.to_string())
            }
        }
    })
    .await
    .map_err(|e| format!("Join error: {}", e))?
}

#[tauri::command]
async fn cancel_file_op(id: u64) -> Result<(), String> {
    if file_ops::cancel_op(id) {
        Ok(())
    } else {
        Err(format!("Operation {} not found", id))
    }
}

// ── Transfer Manager commands ──

#[tauri::command]
async fn transfer_enqueue(
    tasks: Vec<transfer::EnqueueTask>,
    state: State<'_, AppState>,
) -> Result<Vec<u64>, String> {
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    Ok(scheduler.enqueue(tasks, sched.clone()))
}

#[tauri::command]
async fn transfer_cancel(
    id: u64,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    Ok(scheduler.cancel(id, sched.clone()))
}

#[tauri::command]
async fn transfer_reorder(
    ids: Vec<u64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.reorder(ids);
    Ok(())
}

#[tauri::command]
async fn transfer_get_history(
    state: State<'_, AppState>,
) -> Result<Vec<transfer::TransferHistoryRecord>, String> {
    let scheduler = state.transfer_scheduler.lock().await;
    Ok(scheduler.get_history())
}

#[tauri::command]
async fn transfer_clear_history(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.clear_history();
    Ok(())
}

#[tauri::command]
async fn transfer_set_ftp_slots(
    n: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.set_ftp_max_slots(n);
    Ok(())
}

#[tauri::command]
async fn transfer_set_local_slots(
    n: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.set_local_max_slots(n);
    Ok(())
}

#[tauri::command]
async fn transfer_get_slots(
    state: State<'_, AppState>,
) -> Result<(usize, usize), String> {
    let scheduler = state.transfer_scheduler.lock().await;
    Ok(scheduler.get_slot_config())
}

#[tauri::command]
fn terminal_spawn(tab_id: u32, shell: String, cwd: Option<String>, cols: u16, rows: u16, state: State<'_, AppState>) -> Result<(), String> {
    state.terminal.spawn(tab_id, &shell, cwd.as_deref(), cols, rows)
}

#[tauri::command]
fn terminal_input(tab_id: u32, data: String, state: State<'_, AppState>) -> Result<(), String> {
    state.terminal.write_input(tab_id, &data)
}

#[tauri::command]
fn terminal_resize(tab_id: u32, cols: u32, rows: u32, state: State<'_, AppState>) -> Result<(), String> {
    state.terminal.resize(tab_id, cols, rows)
}

#[tauri::command]
fn terminal_kill(tab_id: u32, state: State<'_, AppState>) {
    state.terminal.kill(tab_id);
}

#[tauri::command]
fn neovim_spawn(state: State<'_, AppState>) -> Result<(), String> {
    let mut neovim = state.neovim.lock().unwrap();
    neovim.spawn()
}

#[tauri::command]
fn neovim_input(keys: String, state: State<'_, AppState>) -> Result<(), String> {
    let neovim = state.neovim.lock().unwrap();
    neovim.send_input(&keys)
}

#[tauri::command]
fn neovim_command(cmd: String, state: State<'_, AppState>) -> Result<(), String> {
    let neovim = state.neovim.lock().unwrap();
    neovim.send_input(&format!(":{}", cmd))
}

#[tauri::command]
fn get_file_size(path: String) -> Result<u64, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    file_path
        .metadata()
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file size: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    path: String,
    size: u64,
    is_dir: bool,
    created: Option<String>,
    modified: Option<String>,
    accessed: Option<String>,
    is_readonly: bool,
    is_hidden: bool,
    is_system: bool,
    item_count: Option<usize>,
}

#[tauri::command]
fn get_file_info(path: String) -> Result<FileInfo, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    use std::os::windows::fs::MetadataExt;
    let attrs = metadata.file_attributes();
    let is_hidden = name.starts_with('.') || (attrs & 0x2 != 0);
    let is_system = attrs & 0x4 != 0;

    let format_time = |t: std::io::Result<std::time::SystemTime>| -> Option<String> {
        t.ok().map(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let secs = duration.as_secs();
            let datetime = chrono_like(secs);
            datetime
        })
    };

    let created = format_time(metadata.created());
    let modified = format_time(metadata.modified());
    let accessed = format_time(metadata.accessed());

    let item_count = if metadata.is_dir() {
        fs::read_dir(&path).ok().map(|entries| entries.count())
    } else {
        None
    };

    Ok(FileInfo {
        name,
        path,
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        created,
        modified,
        accessed,
        is_readonly: metadata.permissions().readonly(),
        is_hidden,
        is_system,
        item_count,
    })
}

fn chrono_like(secs: u64) -> String {
    // Simple timestamp formatting without chrono dependency
    let s = secs as i64;
    let days = s / 86400;
    let time_of_day = s % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Days since epoch to Y-M-D (simplified leap year calculation)
    let mut y = 1970;
    let mut remaining_days = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days: [i64; 12] = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 0usize;
    while m < 12 && remaining_days >= month_days[m] {
        remaining_days -= month_days[m];
        m += 1;
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m + 1, remaining_days + 1, hour, minute, second)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("C:\\"))
        .join("wind")
}

fn config_path() -> std::path::PathBuf {
    config_dir().join("windrc.json")
}

#[tauri::command]
fn read_config() -> Result<serde_json::Value, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

#[tauri::command]
fn write_config(options: serde_json::Value) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    let content = serde_json::to_string_pretty(&options)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(config_path(), content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    open::that(&path)
        .map_err(|e| format!("Failed to open: {}", e))
}

#[tauri::command]
fn open_with_dialog(path: String) -> Result<(), String> {
    // Use rundll32 shell32.dll,OpenAs_RunDLL which is the standard Windows "Open With" dialog
    std::process::Command::new("rundll32")
        .args(["shell32.dll,OpenAs_RunDLL", &path])
        .spawn()
        .map_err(|e| format!("Failed to open with dialog: {}", e))?;
    Ok(())
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    let bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(decode_text(&bytes))
}

#[tauri::command]
fn read_file_partial(path: String, max_bytes: u64) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    use std::io::Read;
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = vec![0u8; max_bytes as usize];
    let bytes_read = file.read(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    buffer.truncate(bytes_read);

    Ok(decode_text(&buffer))
}

/// Decode bytes to String: strict UTF-8 first, then chardetng + encoding_rs
fn decode_text(bytes: &[u8]) -> String {
    // Strip UTF-8 BOM if present (EF BB BF)
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let (decoded, _had_errors) = encoding.decode_without_bom_handling(bytes);
    decoded.into_owned()
}

#[derive(Debug, Serialize)]
pub struct ImageThumbnail {
    data: String,
    width: u32,
    height: u32,
    original_size: u64,
    is_thumbnail: bool,
}

#[tauri::command]
fn read_binary_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    let bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(STANDARD.encode(bytes))
}

#[tauri::command]
fn read_binary_file_partial(path: String, max_bytes: u64) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    use std::io::Read;
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = vec![0u8; max_bytes as usize];
    let bytes_read = file.read(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    buffer.truncate(bytes_read);

    Ok(STANDARD.encode(buffer))
}

// Thumbnail: scale to max 1200px on the longest side. 4/5 layout on 1920px≃1500px panel.
const THUMBNAIL_MAX_SIDE: u32 = 1200;
// Files smaller than 512KB are fast to transfer — skip thumbnail & send original quality.
const THUMBNAIL_SKIP_SIZE: u64 = 512 * 1024;
// JPEG re-encode quality after scaling (85 = good balance, avoids double-compression artifacts).
const THUMBNAIL_JPEG_QUALITY: u8 = 85;

#[tauri::command]
async fn read_image_thumbnail(path: String) -> Result<ImageThumbnail, String> {
    let file_path = Path::new(&path).to_path_buf();

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    tokio::task::spawn_blocking(move || {
        let original_size = fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // 读取文件头检测实际格式
        let file_data = fs::read(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let is_actual_jpeg = file_data.len() >= 2 && file_data[0] == 0xFF && file_data[1] == 0xD8;
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let is_jpeg_ext = ext == "jpg" || ext == "jpeg";

        eprintln!("[thumbnail] Processing: {} ({} bytes, ext={}, actual_jpeg={})", path, original_size, ext, is_actual_jpeg);

        if is_actual_jpeg {
            // JPEG: use turbojpeg + fast_image_resize
            let t0 = std::time::Instant::now();

            let mut decompressor = turbojpeg::Decompressor::new()
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to create decompressor: {}", e);
                    format!("Failed to create decompressor: {}", e)
                })?;
            let header = decompressor.read_header(&file_data)
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to read JPEG headers {}: {}", path, e);
                    format!("Failed to read JPEG headers: {}", e)
                })?;
            let orig_w = header.width as u32;
            let orig_h = header.height as u32;
            let max_side = orig_w.max(orig_h);

            eprintln!("[thumbnail] JPEG dimensions: {}x{}, max_side={}", orig_w, orig_h, max_side);

            if max_side <= THUMBNAIL_MAX_SIDE || original_size <= THUMBNAIL_SKIP_SIZE {
                eprintln!("[thumbnail] Skip thumbnail ({}x{}, {}B)", orig_w, orig_h, original_size);
                return Ok(ImageThumbnail {
                    data: String::new(),
                    width: orig_w,
                    height: orig_h,
                    original_size,
                    is_thumbnail: false,
                });
            }

            let t1 = std::time::Instant::now();
            // turbojpeg can't decode CMYK / grayscale JPEGs to RGB.
            // Fall back to the image crate for those.
            let rgb_pixels: Vec<u8> = match turbojpeg::decompress(&file_data, turbojpeg::PixelFormat::RGB) {
                Ok(image) => image.pixels,
                Err(e) => {
                    eprintln!("[thumbnail] turbojpeg failed ({}), falling back to image crate", e);
                    let img = image::load_from_memory(&file_data)
                        .map_err(|e2| format!("Failed to decode JPEG: {} (turbojpeg: {})", e2, e))?;
                    img.to_rgb8().into_raw()
                }
            };
            let t2 = std::time::Instant::now();

            let ratio = THUMBNAIL_MAX_SIDE as f64 / max_side as f64;
            let final_w = (orig_w as f64 * ratio).round() as u32;
            let final_h = (orig_h as f64 * ratio).round() as u32;

            let src_image = fast_image_resize::images::Image::from_vec_u8(
                orig_w,
                orig_h,
                rgb_pixels,
                fast_image_resize::PixelType::U8x3,
            ).map_err(|e| format!("Failed to create source image: {}", e))?;

            let mut dst_image = fast_image_resize::images::Image::new(
                final_w,
                final_h,
                fast_image_resize::PixelType::U8x3,
            );

            let mut resizer = fast_image_resize::Resizer::new();
            #[cfg(target_arch = "x86_64")]
            unsafe {
                resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
            }
            let options = fast_image_resize::ResizeOptions::new()
                .resize_alg(fast_image_resize::ResizeAlg::Nearest);
            resizer.resize(&src_image, &mut dst_image, &options)
                .map_err(|e| format!("Failed to resize image: {}", e))?;
            let t3 = std::time::Instant::now();

            let mut buf: Vec<u8> = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY);
            let resized_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
                image::ImageBuffer::from_raw(final_w, final_h, dst_image.into_vec())
                    .ok_or("Failed to create resized image buffer")?;
            resized_img.write_with_encoder(encoder)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            let t4 = std::time::Instant::now();

            eprintln!("[turbojpeg+fast-resize] {}x{} -> {}x{}, read={}ms decode={}ms resize={}ms encode={}ms total={}ms",
                orig_w, orig_h, final_w, final_h,
                (t1-t0).as_millis(), (t2-t1).as_millis(), (t3-t2).as_millis(), (t4-t3).as_millis(), (t4-t0).as_millis());

            Ok(ImageThumbnail {
                data: STANDARD.encode(&buf),
                width: final_w,
                height: final_h,
                original_size,
                is_thumbnail: true,
            })
        } else {
            // Non-JPEG: use image crate + fast_image_resize
            // 包括扩展名是 jpg 但实际不是 JPEG 的文件
            if is_jpeg_ext && !is_actual_jpeg {
                eprintln!("[thumbnail] Warning: {} has .jpg extension but is not JPEG format", path);
            }

            // 从文件内容猜测实际格式
            let format = image::guess_format(&file_data)
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to guess format {}: {}", path, e);
                    format!("Failed to guess image format: {}", e)
                })?;
            eprintln!("[thumbnail] Detected format: {:?}", format);

            let reader = image::ImageReader::new(std::io::Cursor::new(&file_data))
                .with_guessed_format()
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to create reader {}: {}", path, e);
                    format!("Failed to create image reader: {}", e)
                })?;
            let (orig_w, orig_h) = reader.into_dimensions()
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to read dimensions {}: {}", path, e);
                    format!("Failed to read image dimensions: {}", e)
                })?;
            let max_side = orig_w.max(orig_h);

            eprintln!("[thumbnail] Non-JPEG dimensions: {}x{}, max_side={}", orig_w, orig_h, max_side);

            if max_side <= THUMBNAIL_MAX_SIDE || original_size <= THUMBNAIL_SKIP_SIZE {
                eprintln!("[thumbnail] Skip thumbnail ({}x{}, {}B)", orig_w, orig_h, original_size);
                return Ok(ImageThumbnail {
                    data: String::new(),
                    width: orig_w,
                    height: orig_h,
                    original_size,
                    is_thumbnail: false,
                });
            }

            let img = image::load_from_memory(&file_data)
                .map_err(|e| {
                    eprintln!("[thumbnail] Failed to load image {}: {}", path, e);
                    format!("Failed to load image: {}", e)
                })?;
            let rgb_img = img.to_rgb8();

            let ratio = THUMBNAIL_MAX_SIDE as f64 / max_side as f64;
            let final_w = (orig_w as f64 * ratio).round() as u32;
            let final_h = (orig_h as f64 * ratio).round() as u32;

            let src_image = fast_image_resize::images::Image::from_vec_u8(
                orig_w,
                orig_h,
                rgb_img.into_raw(),
                fast_image_resize::PixelType::U8x3,
            ).map_err(|e| format!("Failed to create source image: {}", e))?;

            let mut dst_image = fast_image_resize::images::Image::new(
                final_w,
                final_h,
                fast_image_resize::PixelType::U8x3,
            );

            let mut resizer = fast_image_resize::Resizer::new();
            #[cfg(target_arch = "x86_64")]
            unsafe {
                resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
            }
            let options = fast_image_resize::ResizeOptions::new()
                .resize_alg(fast_image_resize::ResizeAlg::Nearest);
            resizer.resize(&src_image, &mut dst_image, &options)
                .map_err(|e| format!("Failed to resize image: {}", e))?;

            let mut buf: Vec<u8> = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY);
            let resized_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
                image::ImageBuffer::from_raw(final_w, final_h, dst_image.into_vec())
                    .ok_or("Failed to create resized image buffer")?;
            resized_img.write_with_encoder(encoder)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

            Ok(ImageThumbnail {
                data: STANDARD.encode(&buf),
                width: final_w,
                height: final_h,
                original_size,
                is_thumbnail: true,
            })
        }
    })
    .await
    .map_err(|e| format!("Image processing task failed: {}", e))?
}

#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    let file_path = Path::new(&path);

    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }
    }

    fs::write(file_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    name: String,
    path: String,
    relative_path: String,
    is_dir: bool,
}

fn is_rg_available() -> bool {
    std::process::Command::new("rg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_fd_path() -> Option<String> {
    // 尝试从常见位置找 fd
    let possible_paths = vec![
        "fd".to_string(),
        "D:\\Application\\fd\\fd-v10.2.0-x86_64-pc-windows-msvc\\fd.exe".to_string(),
    ];

    for path in possible_paths {
        if std::process::Command::new(&path)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

fn search_with_fd(fd_path: &str, root: &str, pattern: &str, max: usize, max_depth: usize) -> Result<Vec<SearchResult>, String> {
    let re = Regex::new(&format!("(?i){}", pattern)).map_err(|e| format!("Invalid regex: {}", e))?;
    let depth_str = max_depth.to_string();
    let mut child = std::process::Command::new(fd_path)
        .args([
            "-H",  // 包含隐藏文件/目录
            "--no-ignore",  // 不遵守 .gitignore
            "--max-results", &max.to_string(),
            "--max-depth", &depth_str,
            "-i",  // 大小写不敏感
            pattern,
            root,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(root)
        .spawn()
        .map_err(|e| format!("Failed to run fd: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let root_path = Path::new(root);
    let mut results = Vec::new();

    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        if results.len() >= max || SEARCH_CANCELLED.load(Ordering::Relaxed) {
            let _ = child.kill();
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let path = Path::new(&line);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // fd 匹配完整路径，这里额外检查文件名是否匹配，避免路径中偶然包含 pattern 的误匹配
        if !re.is_match(&name) {
            continue;
        }

        let relative_path = path
            .strip_prefix(root_path)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('/', "\\");
        results.push(SearchResult {
            name,
            path: line.replace('/', "\\").trim_end_matches('\\').to_string(),
            relative_path,
            is_dir: path.is_dir(),
        });
    }

    // 按类型排序：目录在前，文件在后
    results.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(results)
}

fn search_with_rg(root: &str, pattern: &str, max: usize, max_depth: usize) -> Result<Vec<SearchResult>, String> {
    // rg --files 只列出文件，不列出目录
    // 改用 Rust 原生搜索，这样可以同时搜索文件和目录
    search_with_rust(root, pattern, max, max_depth)
}

fn search_with_rust(root: &str, pattern: &str, max: usize, max_depth: usize) -> Result<Vec<SearchResult>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    let root_path = Path::new(root);
    let mut results = Vec::new();

    // 使用递归函数搜索，忽略权限错误
    fn search_dir(
        dir: &Path,
        root_path: &Path,
        re: &Regex,
        results: &mut Vec<SearchResult>,
        max: usize,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth || results.len() >= max {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return, // 忽略权限错误
        };

        for entry in entries {
            if results.len() >= max || SEARCH_CANCELLED.load(Ordering::Relaxed) {
                return;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();

            // 跳过隐藏文件/目录
            if name.starts_with('.') {
                continue;
            }

            let path = entry.path();
            let is_dir = path.is_dir();

            // 匹配文件名（忽略大小写）
            if re.is_match(&name) {
                let relative_path = path
                    .strip_prefix(root_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('/', "\\");
                results.push(SearchResult {
                    name: name.clone(),
                    path: path.to_string_lossy().to_string(),
                    relative_path,
                    is_dir,
                });
            }

            // 递归搜索子目录
            if is_dir {
                search_dir(&path, root_path, re, results, max, depth + 1, max_depth);
            }
        }
    }

    search_dir(root_path, root_path, &re, &mut results, max, 0, max_depth);

    // 按类型排序：目录在前，文件在后
    results.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(results)
}

#[tauri::command]
fn cancel_search() {
    SEARCH_CANCELLED.store(true, Ordering::Relaxed);
}

#[tauri::command]
async fn search_files(
    root_path: String,
    pattern: String,
    max_results: Option<usize>,
    recursive: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let max = max_results.unwrap_or(50);
    let is_recursive = recursive.unwrap_or(true);

    if pattern.is_empty() {
        return Ok(Vec::new());
    }

    // FTP: search within current directory by listing + filtering
    if root_path.starts_with("ftp://") {
        let (conn_name, remote_path) = parse_ftp_url(&root_path)?;
        let mgr = state.ftp_manager.lock().await;
        let entries = try_list_dir(&mgr, conn_name, remote_path).await?;
        drop(mgr);
        let lower = pattern.to_lowercase();
        let results: Vec<SearchResult> = entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&lower))
            .take(max)
            .map(|e| SearchResult {
                name: e.name.clone(),
                path: e.path.clone(),
                relative_path: e.name.clone(),
                is_dir: e.is_dir,
            })
            .collect();
        return Ok(results);
    }

    let max_depth: usize = if is_recursive { 10 } else { 1 };

    // 重置取消标志
    SEARCH_CANCELLED.store(false, Ordering::Relaxed);

    // 在独立线程中执行搜索，不阻塞主线程
    tokio::task::spawn_blocking(move || {
        // 优先使用 fd，然后 rg，最后 Rust 原生
        if let Some(fd_path) = get_fd_path() {
            search_with_fd(&fd_path, &root_path, &pattern, max, max_depth)
        } else if is_rg_available() {
            search_with_rg(&root_path, &pattern, max, max_depth)
        } else {
            search_with_rust(&root_path, &pattern, max, max_depth)
        }
    })
    .await
    .map_err(|e| format!("Search task failed: {}", e))?
}

#[tauri::command]
fn check_search_tools() -> serde_json::Value {
    let fd_available = get_fd_path().is_some();
    let rg_available = is_rg_available();

    serde_json::json!({
        "fd": fd_available,
        "rg": rg_available
    })
}

// ── FTP helpers ──

/// Parse `ftp://<name>/<path>` into (connection_name, remote_path).
fn parse_ftp_url(url: &str) -> Result<(&str, &str), String> {
    let url = url.strip_prefix("ftp://")
        .ok_or_else(|| format!("Not an FTP URL: {}", url))?;
    let slash_pos = url.find('/');
    let (name, path) = match slash_pos {
        Some(pos) => {
            let (n, p) = url.split_at(pos);
            (n, if p.is_empty() { "/" } else { p })
        }
        None => (url, "/"),
    };
    Ok((name, path))
}

// ── FTP Tauri commands ──

#[tauri::command]
async fn ftp_connect(
    name: String,
    host: String,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let port = port.unwrap_or(21);
    let user = user.unwrap_or_else(|| "anonymous".to_string());
    let password = password.unwrap_or_else(|| "anonymous".to_string());
    eprintln!("[FTP] command ftp_connect: name={} host={}:{} user={}", name, host, port, user);
    let mut mgr = state.ftp_manager.lock().await;
    mgr.connect(&name, &host, port, &user, &password).await?;
    Ok(format!("Connected to {}", name))
}

#[tauri::command]
async fn ftp_disconnect(name: String, state: State<'_, AppState>) -> Result<String, String> {
    eprintln!("[FTP] command ftp_disconnect: name={}", name);
    let mut mgr = state.ftp_manager.lock().await;
    mgr.disconnect(&name).await?;
    Ok(format!("Disconnected from {}", name))
}

#[tauri::command]
async fn ftp_read_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    eprintln!("[FTP] command ftp_read_directory: conn={} path={}", conn_name, remote_path);

    // Try listing; if session is stale, reconnect once and retry
    let mut mgr = state.ftp_manager.lock().await;
    let entries = match try_list_dir(&mgr, conn_name, remote_path).await {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("[FTP] read_directory: listing failed ({}), reconnecting...", e);
            mgr.ensure_connected(conn_name).await?;
            try_list_dir(&mgr, conn_name, remote_path).await?
        }
    };
    drop(mgr);

    Ok(entries)
}

async fn try_list_dir(
    mgr: &ftp::FtpManager,
    conn_name: &str,
    remote_path: &str,
) -> Result<Vec<FileEntry>, String> {
    let session = mgr.get(conn_name)?;
    let mut ftp = session.lock().await;

    // Try MLSD first, fall back to LIST
    let mut entries = match ftp.client.mlsd(Some(remote_path)).await {
        Ok(lines) => {
            let mut entries = Vec::new();
            for line in &lines {
                if let Some((name, is_dir, size, modified)) = ftp::parse_mld_line(line) {
                    let full_path = format!("ftp://{}{}/{}", conn_name, remote_path.trim_end_matches('/'), name);
                    entries.push(FileEntry {
                        name,
                        path: full_path,
                        is_dir,
                        size,
                        is_hidden: false,
                        modified,
                        created: None,
                        children: None,
                    });
                }
            }
            eprintln!("[FTP] read_directory: MLSD returned {} entries", entries.len());
            entries
        }
        Err(e) => {
            eprintln!("[FTP] read_directory: MLSD failed ({}), falling back to LIST", e);
            let lines = ftp.client.list(Some(remote_path))
                .await
                .map_err(|e| format!("Failed to list directory: {}", e))?;
            let mut entries = Vec::new();
            for line in &lines {
                if let Some((name, is_dir, size, _modified)) = ftp::parse_list_line(line) {
                    let full_path = format!("ftp://{}{}/{}", conn_name, remote_path.trim_end_matches('/'), name);
                    entries.push(FileEntry {
                        name,
                        path: full_path,
                        is_dir,
                        size,
                        is_hidden: false,
                        modified: None,
                        created: None,
                        children: None,
                    });
                }
            }
            eprintln!("[FTP] read_directory: LIST returned {} entries", entries.len());
            entries
        }
    };

    // Deduplicate by path
    {
        let before = entries.len();
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| seen.insert(e.path.clone()));
        let removed = before - entries.len();
        if removed > 0 {
            eprintln!("[FTP] read_directory: dedup removed {} duplicate entries, {} remaining", removed, entries.len());
        }
    }

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

#[tauri::command]
async fn ftp_download(
    app: tauri::AppHandle,
    conn_name: String,
    remote_path: String,
    local_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.ftp_manager.lock().await;
    let session = mgr.get(&conn_name)?;
    drop(mgr);

    let file_name = remote_path.rsplit('/').next().unwrap_or(&remote_path);
    eprintln!("[FTP] command ftp_download: conn={} remote={} → local={}", conn_name, remote_path, local_path);

    let mut ftp = session.lock().await;

    // Stream download directly to file
    let local = Path::new(&local_path);
    if let Some(parent) = local.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    use tokio::io::AsyncWriteExt;
    let mut stream = ftp.client.retr_as_stream(&remote_path)
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let mut file = tokio::fs::File::create(local)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let bytes = tokio::io::copy(&mut stream, &mut file)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;
    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;

    eprintln!("[FTP] download: wrote {} bytes for {}", bytes, file_name);

    let _ = app.emit("ftp-progress", serde_json::json!({
        "file": file_name,
        "done": bytes,
        "total": bytes,
        "op": "download"
    }));

    Ok(())
}

#[tauri::command]
async fn ftp_upload(
    app: tauri::AppHandle,
    conn_name: String,
    local_path: String,
    remote_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.ftp_manager.lock().await;
    let session = mgr.get(&conn_name)?;
    drop(mgr);

    let file_name = local_path.rsplit('\\').next().unwrap_or(&local_path);
    eprintln!("[FTP] command ftp_upload: conn={} local={} → remote={}", conn_name, local_path, remote_path);

    let mut ftp = session.lock().await;

    // Stream upload directly from file
    let mut file = tokio::fs::File::open(&local_path)
        .await
        .map_err(|e| format!("Failed to open local file: {}", e))?;
    let file_size = file.metadata().await
        .map(|m| m.len())
        .unwrap_or(0);
    eprintln!("[FTP] upload: streaming {} bytes for {}", file_size, file_name);

    ftp.client.put_file(&remote_path, &mut file)
        .await
        .map_err(|e| format!("Failed to upload: {}", e))?;

    let _ = app.emit("ftp-progress", serde_json::json!({
        "file": file_name,
        "done": file_size,
        "total": file_size,
        "op": "upload"
    }));

    Ok(())
}

#[tauri::command]
async fn ftp_delete(
    path: String,
    _permanent: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    eprintln!("[FTP] command ftp_delete: conn={} path={}", conn_name, remote_path);

    let mgr = state.ftp_manager.lock().await;
    let session = mgr.get(conn_name)?;
    drop(mgr);

    let mut ftp = session.lock().await;
    // Try file delete first, fall back to directory delete
    match ftp.client.rm(&remote_path).await {
        Ok(()) => {
            eprintln!("[FTP] delete: removed file {}", remote_path);
            return Ok(());
        }
        Err(e) => {
            eprintln!("[FTP] delete: rm failed ({}), trying rmdir...", e);
        }
    }
    ftp.client.rmdir(&remote_path)
        .await
        .map_err(|e| format!("Failed to delete: {}", e))?;
    eprintln!("[FTP] delete: removed directory {}", remote_path);
    Ok(())
}

#[tauri::command]
async fn ftp_rename(
    old_path: String,
    new_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (conn_name, remote_path) = parse_ftp_url(&old_path)?;
    let (_, new_remote) = parse_ftp_url(&new_path)?;
    eprintln!("[FTP] command ftp_rename: conn={} from={} to={}", conn_name, remote_path, new_remote);

    // Try server-side rename first (fast path)
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;
        match ftp.client.rename(remote_path, new_remote).await {
            Ok(()) => {
                eprintln!("[FTP] rename: server-side rename OK");
                return Ok(new_path.to_string());
            }
            Err(e) => {
                eprintln!("[FTP] rename: server-side rename failed ({}), falling back to copy+delete", e);
            }
        }
    }

    // Fallback: download → re-upload → delete source
    ftp_copy_move_fallback(conn_name, remote_path, new_remote, &state).await?;
    eprintln!("[FTP] rename: copy+delete fallback completed");
    Ok(new_path.to_string())
}

async fn ftp_copy_move_fallback(
    conn_name: &str,
    src_remote: &str,
    dst_remote: &str,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "wind_ftp_move_{}_{}",
        std::process::id(),
        src_remote.rsplit('/').next().unwrap_or("file")
    ));

    // Download
    eprintln!("[FTP] move-fallback: downloading to temp {}", tmp.display());
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;
        let mut stream = ftp.client.retr_as_stream(src_remote)
            .await
            .map_err(|e| format!("Failed to open download: {}", e))?;
        let mut file = tokio::fs::File::create(&tmp)
            .await
            .map_err(|e| format!("Failed to create temp: {}", e))?;
        tokio::io::copy(&mut stream, &mut file)
            .await
            .map_err(|e| format!("Failed to download: {}", e))?;
    }

    // Upload
    eprintln!("[FTP] move-fallback: uploading to {}", dst_remote);
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;
        let mut file = tokio::fs::File::open(&tmp)
            .await
            .map_err(|e| format!("Failed to read temp: {}", e))?;
        ftp.client.put_file(dst_remote, &mut file)
            .await
            .map_err(|e| format!("Failed to upload: {}", e))?;
    }

    // Delete source
    eprintln!("[FTP] move-fallback: deleting source {}", src_remote);
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;
        ftp.client.rm(src_remote)
            .await
            .map_err(|e| format!("Failed to delete source: {}", e))?;
    }

    let _ = tokio::fs::remove_file(&tmp).await;
    Ok(())
}

#[tauri::command]
async fn ftp_copy(
    conn_name: String,
    src_path: String,
    dst_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    eprintln!("[FTP] command ftp_copy: conn={} src={} dst={}", conn_name, src_path, dst_path);

    // Download to temp file, re-upload (no native server-side copy in FTP)
    let tmp = std::env::temp_dir().join(format!("wind_ftp_copy_{}_{}", std::process::id(), src_path.rsplit('/').next().unwrap_or("file")));

    // Download
    eprintln!("[FTP] copy: downloading to temp {}", tmp.display());
    let bytes_dl: u64;
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(&conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;

        let mut stream = ftp.client.retr_as_stream(&src_path)
            .await
            .map_err(|e| format!("Failed to open download stream: {}", e))?;
        let mut file = tokio::fs::File::create(&tmp)
            .await
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        bytes_dl = tokio::io::copy(&mut stream, &mut file)
            .await
            .map_err(|e| format!("Failed to download: {}", e))?;
    }
    eprintln!("[FTP] copy: downloaded {} bytes", bytes_dl);

    // Upload
    eprintln!("[FTP] copy: uploading from temp");
    let bytes_ul: u64;
    {
        let mgr = state.ftp_manager.lock().await;
        let session = mgr.get(&conn_name)?;
        drop(mgr);
        let mut ftp = session.lock().await;

        let mut file = tokio::fs::File::open(&tmp)
            .await
            .map_err(|e| format!("Failed to read temp file: {}", e))?;
        bytes_ul = ftp.client.put_file(&dst_path, &mut file)
            .await
            .map_err(|e| format!("Failed to upload: {}", e))?;
    }
    eprintln!("[FTP] copy: uploaded {} bytes", bytes_ul);

    // Cleanup
    let _ = tokio::fs::remove_file(&tmp).await;
    eprintln!("[FTP] copy: completed {} → {}", src_path, dst_path);
    Ok(())
}

#[tauri::command]
async fn ftp_create_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    eprintln!("[FTP] command ftp_create_file: conn={} path={}", conn_name, remote_path);

    let mgr = state.ftp_manager.lock().await;
    let session = mgr.get(conn_name)?;
    drop(mgr);

    let mut ftp = session.lock().await;
    // Upload empty content
    match ftp.client.put_with_stream(&remote_path).await {
        Ok(stream) => {
            drop(stream);
            eprintln!("[FTP] create_file: created {}", remote_path);
        }
        Err(e) => {
            eprintln!("[FTP] create_file: FAILED {} — {}", remote_path, e);
            return Err(format!("Failed to create file: {}", e));
        }
    }
    Ok(())
}

#[tauri::command]
async fn ftp_mkdir(
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    eprintln!("[FTP] command ftp_mkdir: conn={} path={}", conn_name, remote_path);

    let mgr = state.ftp_manager.lock().await;
    let session = mgr.get(conn_name)?;
    drop(mgr);

    let mut ftp = session.lock().await;
    // Use custom_command to accept both 250 and 257 (Android servers return 250)
    match ftp.client.custom_command(
        format!("MKD {}", remote_path),
        &[suppaftp::Status::RequestedFileActionOk, suppaftp::Status::PathCreated],
    ).await {
        Ok(_) => eprintln!("[FTP] mkdir: created {}", remote_path),
        Err(e) => {
            eprintln!("[FTP] mkdir: FAILED {} — {}", remote_path, e);
            return Err(format!("Failed to create directory: {}", e));
        }
    }
    Ok(())
}

#[tauri::command]
async fn ftp_download_folder(
    conn_name: String,
    remote_path: String,
    local_path: String,
    move_mode: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    eprintln!("[FTP] command ftp_download_folder: conn={conn_name} remote={remote_path} → local={local_path} move={move_mode}");
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    let (total_bytes, ids) = scheduler.enqueue_ftp_folder_download(
        sched.clone(),
        &conn_name,
        &remote_path,
        &local_path,
        move_mode,
    ).await?;
    Ok(serde_json::json!({ "total_bytes": total_bytes, "task_ids": ids }).to_string())
}

#[tauri::command]
async fn ftp_upload_folder(
    conn_name: String,
    local_path: String,
    remote_path: String,
    move_mode: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    eprintln!("[FTP] command ftp_upload_folder: local={local_path} → conn={conn_name} remote={remote_path} move={move_mode}");
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    let (total_bytes, ids) = scheduler.enqueue_ftp_folder_upload(
        sched.clone(),
        &conn_name,
        &local_path,
        &remote_path,
        move_mode,
    ).await?;
    Ok(serde_json::json!({ "total_bytes": total_bytes, "task_ids": ids }).to_string())
}

#[tauri::command]
async fn list_ftp_connections(state: State<'_, AppState>) -> Result<Vec<ftp::FtpConnectionConfig>, String> {
    let mgr = state.ftp_manager.lock().await;
    let configs = mgr.get_configs();
    eprintln!("[FTP] command list_ftp_connections: {} connection(s)", configs.len());
    Ok(configs)
}

#[tauri::command]
async fn check_ftp_connection(name: String, state: State<'_, AppState>) -> Result<bool, String> {
    eprintln!("[FTP] command check_ftp_connection: name={}", name);
    let mut mgr = state.ftp_manager.lock().await;
    mgr.ensure_connected(&name).await?;
    Ok(true)
}

#[tauri::command]
fn set_ime_enabled(enabled: bool) {
    unsafe {
        let hwnd = GetForegroundWindow();
        let himc = ImmGetContext(hwnd);
        if himc.is_invalid() {
            eprintln!("[IME] ImmGetContext returned invalid handle");
            return;
        }
        let current = ImmGetOpenStatus(himc).as_bool();
        eprintln!("[IME] current={current}, requested={enabled}");
        if current != enabled {
            let _ = ImmSetOpenStatus(himc, enabled.into());
            eprintln!("[IME] toggled to {enabled}");
        }
        let _ = ImmReleaseContext(hwnd, himc);
    }
}

#[derive(Debug, Serialize)]
struct ShellOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn detect_bash_path() -> Option<String> {
    let candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\msys64\usr\bin\bash.exe",
        r"C:\cygwin64\bin\bash.exe",
    ];
    for path in &candidates {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    // Fallback: check if bash is in PATH
    if std::process::Command::new("bash")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("bash".to_string());
    }
    None
}

#[tauri::command]
fn exec_shell_command(command: String, cwd: Option<String>) -> Result<ShellOutput, String> {
    let bash = detect_bash_path()
        .ok_or_else(|| "bash not found. Install Git for Windows or MSYS2.".to_string())?;

    let output = std::process::Command::new(&bash)
        .args(["-c", &command])
        .current_dir(cwd.unwrap_or_else(|| ".".to_string()))
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    Ok(ShellOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let mut terminal = terminal::TerminalManager::new();
            terminal.set_app_handle(handle.clone());
            let mut ftp_manager = ftp::FtpManager::new();
            ftp_manager.load_on_startup();
            let ftp_manager = Arc::new(TokioMutex::new(ftp_manager));
            let transfer_scheduler = Arc::new(TokioMutex::new(
                transfer::TransferScheduler::new(handle.clone(), ftp_manager.clone())
            ));
            app.manage(AppState {
                terminal,
                neovim: Mutex::new(neovim::Neovim::new()),
                file_watcher: Mutex::new(file_watcher::FileWatcher::new()),
                ftp_manager: ftp_manager.clone(),
                transfer_scheduler: transfer_scheduler.clone(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_home_dir,
            file_exists,
            get_file_metadata,
            start_watch_file,
            stop_watch_file,
            read_directory,
            list_archive_entries,
            list_drives,
            delete_file,
            permanent_delete,
            rename_file,
            batch_rename,
            create_batch_rename_temp_file,
            delete_temp_file,
            create_file,
            copy_file,
            copy_file_async,
            move_file,
            move_file_async,
            delete_file_async,
            cancel_file_op,
            check_copy_conflicts,
            transfer_enqueue,
            transfer_cancel,
            transfer_reorder,
            transfer_get_history,
            transfer_clear_history,
            transfer_set_ftp_slots,
            transfer_set_local_slots,
            transfer_get_slots,
            get_file_size,
            get_file_info,
            open_file,
            open_with_dialog,
            read_file,
            read_file_partial,
            read_binary_file,
            read_binary_file_partial,
            read_image_thumbnail,
            write_file,
            terminal_spawn,
            terminal_input,
            terminal_resize,
            terminal_kill,
            neovim_spawn,
            neovim_input,
            neovim_command,
            exec_shell_command,
            search_files,
            cancel_search,
            check_search_tools,
            set_ime_enabled,
            read_config,
            write_config,
            pdf::get_pdf_info,
            pdf::render_pdf_page,
            pdf::search_pdf_text,
            video::get_video_thumbnail,
            video::start_video_server,
            video::stop_video_server,
            ftp_connect,
            ftp_disconnect,
            ftp_read_directory,
            ftp_download,
            ftp_upload,
            ftp_download_folder,
            ftp_upload_folder,
            ftp_delete,
            ftp_rename,
            ftp_copy,
            ftp_create_file,
            ftp_mkdir,
            list_ftp_connections,
            check_ftp_connection,
            python_completion::scan_python_packages,
            python_completion::get_package_api,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
