use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;
use tauri::Emitter;

use log::{error, info};

use crate::{AppState, FileEntry};

static FOLDER_SIZE_CALCS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn register_folder_size_calc(path: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    FOLDER_SIZE_CALCS.lock().unwrap().insert(path.to_string(), flag.clone());
    flag
}

fn unregister_folder_size_calc(path: &str) {
    FOLDER_SIZE_CALCS.lock().unwrap().remove(path);
}

fn cancel_folder_size_calc(path: &str) -> bool {
    if let Some(flag) = FOLDER_SIZE_CALCS.lock().unwrap().get(path) {
        flag.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

fn walk_dir(
    dir: &Path,
    root_path: &str,
    cancel_flag: &AtomicBool,
    total_bytes: &mut u64,
    files: &mut u64,
    dirs: &mut u64,
    last_emit: &mut Instant,
    app: &tauri::AppHandle,
) {
    if cancel_flag.load(Ordering::Relaxed) {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_type = match path.symlink_metadata() {
            Ok(m) => m.file_type(),
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            *dirs += 1;
            walk_dir(&path, root_path, cancel_flag, total_bytes, files, dirs, last_emit, app);
        } else {
            *files += 1;
            *total_bytes += metadata.len();
        }

        if last_emit.elapsed() >= std::time::Duration::from_millis(100) {
            let _ = app.emit(
                "folder-size-tick",
                serde_json::json!({
                    "path": root_path,
                    "total_bytes": *total_bytes,
                    "files": *files,
                    "dirs": *dirs,
                }),
            );
            *last_emit = Instant::now();
        }
    }
}

#[tauri::command]
pub async fn calculate_folder_size(path: String, app: tauri::AppHandle) -> Result<(), String> {
    let cancel_flag = register_folder_size_calc(&path);

    let path_owned = path.clone();
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let mut total_bytes = 0u64;
        let mut files = 0u64;
        let mut dirs = 0u64;
        let mut last_emit = Instant::now();
        let dir_path = Path::new(&path_owned);

        walk_dir(
            dir_path,
            &path_owned,
            &cancel_flag,
            &mut total_bytes,
            &mut files,
            &mut dirs,
            &mut last_emit,
            &app_handle,
        );

        let _ = app_handle.emit(
            "folder-size-done",
            serde_json::json!({
                "path": path_owned,
                "total_bytes": total_bytes,
                "files": files,
                "dirs": dirs,
            }),
        );
        unregister_folder_size_calc(&path_owned);
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_folder_size(path: String) -> Result<(), String> {
    cancel_folder_size_calc(&path);
    Ok(())
}

#[tauri::command]
pub async fn list_virtual_root(state: tauri::State<'_, AppState>) -> Result<Vec<FileEntry>, String> {
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
pub async fn list_drives(state: tauri::State<'_, AppState>) -> Result<Vec<FileEntry>, String> {
    list_virtual_root(state).await
}

#[tauri::command]
pub async fn read_directory(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    // Route FTP paths
    if path.starts_with("ftp://") {
        info!("[FTP] read_directory routing: FTP path — {}", path);
        return crate::commands::ftp_cmd::ftp_read_directory(path, state).await;
    }
    // Route virtual root
    if path == "\\" {
        info!("[FTP] read_directory routing: virtual root");
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
                            || entry
                                .metadata()
                                .map(|m| {
                                    use std::os::windows::fs::MetadataExt;
                                    m.file_attributes() & 0x2 != 0 // FILE_ATTRIBUTE_HIDDEN
                                })
                                .unwrap_or(false);

                        let modified = entry
                            .metadata()
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs());
                        let created = entry
                            .metadata()
                            .ok()
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
                        error!("Error reading entry: {}", e);
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
