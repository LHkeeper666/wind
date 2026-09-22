use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::tool_cache;

pub(crate) static SEARCH_CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub is_dir: bool,
    pub is_hidden: bool,
}

fn is_hidden_search_path(path: &Path, root_path: &Path) -> bool {
    let relative_path = path.strip_prefix(root_path).unwrap_or(path);
    let mut current_path = root_path.to_path_buf();

    for component in relative_path.components() {
        let name = component.as_os_str().to_string_lossy();
        current_path.push(component.as_os_str());
        if name.starts_with('.') {
            return true;
        }
        let is_hidden = fs::metadata(&current_path)
            .map(|metadata| {
                use std::os::windows::fs::MetadataExt;
                metadata.file_attributes() & 0x2 != 0
            })
            .unwrap_or(false);
        if is_hidden {
            return true;
        }
    }

    false
}

fn is_rg_available() -> bool {
    tool_cache::rg_path().is_some()
}

fn get_fd_path() -> Option<String> {
    tool_cache::fd_path().map(|path| path.to_string_lossy().to_string())
}

fn search_with_fd(
    fd_path: &str,
    root: &str,
    pattern: &str,
    max: usize,
    max_depth: usize,
) -> Result<Vec<SearchResult>, String> {
    let re =
        Regex::new(&format!("(?i){}", pattern)).map_err(|e| format!("Invalid regex: {}", e))?;
    let depth_str = max_depth.to_string();
    let mut child = tool_cache::background_command(fd_path)
        .args([
            "-H",
            "--no-ignore",
            "--max-results",
            &max.to_string(),
            "--max-depth",
            &depth_str,
            "-i",
            pattern,
            root,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(root)
        .spawn()
        .map_err(|e| {
            tool_cache::invalidate("fd");
            format!("Failed to run fd: {}", e)
        })?;

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
            is_hidden: is_hidden_search_path(path, root_path),
        });
    }

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

fn search_with_rust(
    root: &str,
    pattern: &str,
    max: usize,
    max_depth: usize,
) -> Result<Vec<SearchResult>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    let root_path = Path::new(root);
    let mut results = Vec::new();

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
            Err(_) => return,
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

            if name.starts_with('.') {
                continue;
            }

            let path = entry.path();
            let is_dir = path.is_dir();

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
                    is_hidden: is_hidden_search_path(&path, root_path),
                });
            }

            if is_dir {
                search_dir(&path, root_path, re, results, max, depth + 1, max_depth);
            }
        }
    }

    search_dir(root_path, root_path, &re, &mut results, max, 0, max_depth);

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
pub fn cancel_search() {
    SEARCH_CANCELLED.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn search_files(
    root_path: String,
    pattern: String,
    max_results: Option<usize>,
    recursive: Option<bool>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<SearchResult>, String> {
    let max = max_results.unwrap_or(50);
    let is_recursive = recursive.unwrap_or(true);

    if pattern.is_empty() {
        return Ok(Vec::new());
    }

    // FTP: search within current directory by listing + filtering
    if root_path.starts_with("ftp://") {
        let (conn_name, remote_path) = crate::commands::ftp_cmd::parse_ftp_url(&root_path)?;
        let mgr = state.ftp_manager.lock().await;
        let entries = crate::commands::ftp_cmd::try_list_dir(&mgr, conn_name, remote_path).await?;
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
                is_hidden: false,
            })
            .collect();
        return Ok(results);
    }

    let max_depth: usize = if is_recursive { 10 } else { 1 };

    SEARCH_CANCELLED.store(false, Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        if let Some(fd_path) = get_fd_path() {
            search_with_fd(&fd_path, &root_path, &pattern, max, max_depth)
        } else {
            search_with_rust(&root_path, &pattern, max, max_depth)
        }
    })
    .await
    .map_err(|e| format!("Search task failed: {}", e))?
}

#[tauri::command]
pub fn check_search_tools() -> serde_json::Value {
    let fd_available = get_fd_path().is_some();
    let rg_available = is_rg_available();

    serde_json::json!({
        "fd": fd_available,
        "rg": rg_available
    })
}
