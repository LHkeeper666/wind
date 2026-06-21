mod terminal;
mod neovim;

use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::fs::File;
use tauri::{Manager, State};

static SEARCH_CANCELLED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn list_drives() -> Vec<FileEntry> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        if Path::new(&drive).exists() {
            drives.push(FileEntry {
                name: drive.clone(),
                path: drive,
                is_dir: true,
                size: None,
                children: None,
            });
        }
    }
    drives
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
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
    terminal: Mutex<terminal::Terminal>,
    neovim: Mutex<neovim::Neovim>,
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
fn read_directory(path: String) -> Result<Vec<FileEntry>, String> {
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

                        entries.push(FileEntry {
                            name: file_name,
                            path: file_path,
                            is_dir,
                            size,
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

    if file_path.is_dir() {
        fs::remove_dir_all(file_path)
            .map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))
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

    if src.is_dir() {
        // Copy directory recursively
        copy_dir_recursive(src, dst)
            .map_err(|e| format!("Failed to copy directory: {}", e))
    } else {
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
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
fn terminal_spawn(shell: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut terminal = state.terminal.lock().unwrap();
    terminal.spawn(&shell)
}

#[tauri::command]
fn terminal_input(data: String, state: State<'_, AppState>) -> Result<(), String> {
    let terminal = state.terminal.lock().unwrap();
    terminal.write_input(&data)
}

#[tauri::command]
fn terminal_resize(_cols: u32, _rows: u32, _state: State<'_, AppState>) -> Result<(), String> {
    // TODO: Implement resize for ConPTY
    Ok(())
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
fn read_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
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
) -> Result<Vec<SearchResult>, String> {
    let max = max_results.unwrap_or(50);
    let is_recursive = recursive.unwrap_or(true);
    let max_depth: usize = if is_recursive { 10 } else { 1 };

    // 重置取消标志
    SEARCH_CANCELLED.store(false, Ordering::Relaxed);

    if pattern.is_empty() {
        return Ok(Vec::new());
    }

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let mut terminal = terminal::Terminal::new();
            terminal.set_app_handle(handle);
            app.manage(AppState {
                terminal: Mutex::new(terminal),
                neovim: Mutex::new(neovim::Neovim::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_home_dir,
            read_directory,
            list_archive_entries,
            list_drives,
            delete_file,
            rename_file,
            create_file,
            copy_file,
            read_file,
            read_binary_file,
            write_file,
            terminal_spawn,
            terminal_input,
            terminal_resize,
            neovim_spawn,
            neovim_input,
            neovim_command,
            search_files,
            cancel_search,
            check_search_tools
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
