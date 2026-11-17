use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::Emitter;

static CALCULATIONS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Deserialize)]
pub struct SelectionRoot {
    path: String,
    #[serde(default)]
    skip_rel_paths: Vec<String>,
}

#[derive(Clone, Default, Serialize)]
struct SelectionProgress {
    request_id: String,
    selected_items: u64,
    files: u64,
    dirs: u64,
    total_bytes: u64,
    incomplete: bool,
    cancelled: bool,
}

fn path_key(path: &str) -> String {
    path.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

fn within(path: &str, parent: &str) -> bool {
    path == parent || path.strip_prefix(parent).is_some_and(|rest| rest.starts_with('\\'))
}

fn excluded(path: &str, root: &SelectionRoot) -> bool {
    let root_key = path_key(&root.path);
    root.skip_rel_paths.iter().any(|skip| {
        let skip_key = path_key(skip);
        !skip_key.is_empty() && within(path, &format!("{}\\{}", root_key, skip_key))
    })
}

fn walk(
    path: &Path,
    root: &SelectionRoot,
    seen: &mut HashSet<String>,
    progress: &mut SelectionProgress,
    cancel: &AtomicBool,
    last_emit: &mut Instant,
    app: Option<&tauri::AppHandle>,
) {
    if cancel.load(Ordering::Relaxed) { return; }
    let key = path_key(&path.to_string_lossy());
    if excluded(&key, root) || !seen.insert(key) { return; }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => { progress.incomplete = true; return; }
    };
    // Windows junctions and symbolic links both carry FILE_ATTRIBUTE_REPARSE_POINT.
    if metadata.file_attributes() & 0x400 != 0 { return; }
    if metadata.is_dir() {
        progress.dirs += 1;
        match fs::read_dir(path) {
            Ok(entries) => for entry in entries {
                if cancel.load(Ordering::Relaxed) { return; }
                match entry {
                    Ok(entry) => walk(&entry.path(), root, seen, progress, cancel, last_emit, app),
                    Err(_) => progress.incomplete = true,
                }
            },
            Err(_) => progress.incomplete = true,
        }
    } else if metadata.is_file() {
        progress.files += 1;
        progress.total_bytes = progress.total_bytes.saturating_add(metadata.len());
    }
    if last_emit.elapsed() >= Duration::from_millis(100) {
        if let Some(app) = app { let _ = app.emit("selection-stats-tick", &*progress); }
        *last_emit = Instant::now();
    }
}

fn scan_roots(request_id: String, mut roots: Vec<SelectionRoot>, cancel: &AtomicBool, app: Option<&tauri::AppHandle>) -> SelectionProgress {
    let mut progress = SelectionProgress { request_id, ..Default::default() };
    let mut seen = HashSet::new();
    let mut last_emit = Instant::now();
    roots.sort_by_key(|root| path_key(&root.path).len());
    for root in &roots {
        if cancel.load(Ordering::Relaxed) { break; }
        let key = path_key(&root.path);
        if !seen.contains(&key) && !excluded(&key, root) {
            progress.selected_items += 1;
        }
        walk(Path::new(&root.path), root, &mut seen, &mut progress, cancel, &mut last_emit, app);
    }
    progress.cancelled = cancel.load(Ordering::Relaxed);
    progress
}

#[tauri::command]
pub fn calculate_selection_stats(request_id: String, roots: Vec<SelectionRoot>, app: tauri::AppHandle) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    CALCULATIONS.lock().map_err(|e| e.to_string())?.insert(request_id.clone(), cancel.clone());
    std::thread::spawn(move || {
        let progress = scan_roots(request_id.clone(), roots, &cancel, Some(&app));
        let _ = app.emit("selection-stats-done", &progress);
        if let Ok(mut calculations) = CALCULATIONS.lock() { calculations.remove(&request_id); }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn exclusion_respects_path_boundaries_and_case() {
        let root = SelectionRoot { path: r"C:\Work".into(), skip_rel_paths: vec![r"nested\excluded".into()] };
        assert!(excluded(&path_key(r"C:\WORK\nested\excluded\a.txt"), &root));
        assert!(!excluded(&path_key(r"C:\Work\nested\excluded-other\a.txt"), &root));
        assert!(!excluded(&path_key(r"C:\Work\nested\included\a.txt"), &root));
    }

    #[test]
    fn missing_root_marks_result_incomplete_and_cancel_stops_scan() {
        let root = SelectionRoot { path: std::env::temp_dir().join("wind-selection-stats-missing-13579").to_string_lossy().into_owned(), skip_rel_paths: vec![] };
        let result = scan_roots("missing".into(), vec![root.clone()], &AtomicBool::new(false), None);
        assert!(result.incomplete);
        let result = scan_roots("cancel".into(), vec![root], &AtomicBool::new(true), None);
        assert!(result.cancelled);
        assert_eq!(result.selected_items, 0);
    }

    #[test]
    fn selected_file_is_counted_once() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let size = fs::metadata(&path).unwrap().len();
        let root = SelectionRoot { path: path.to_string_lossy().into_owned(), skip_rel_paths: vec![] };
        let result = scan_roots("file".into(), vec![root.clone(), root], &AtomicBool::new(false), None);
        assert_eq!((result.selected_items, result.files, result.dirs, result.total_bytes), (1, 1, 0, size));
        assert!(!result.incomplete);
    }

    #[test]
    fn directory_exclusion_and_separate_file_are_counted() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"));
        let directory = base.join("src").join("commands");
        let separate = base.join("Cargo.toml");
        let excluded_file = directory.join("file_info.rs");
        let directory_root = SelectionRoot { path: directory.to_string_lossy().into_owned(), skip_rel_paths: vec![] };
        let file_root = SelectionRoot { path: separate.to_string_lossy().into_owned(), skip_rel_paths: vec![] };
        let full = scan_roots("full".into(), vec![directory_root.clone(), file_root.clone()], &AtomicBool::new(false), None);
        let partial = scan_roots("partial".into(), vec![SelectionRoot { skip_rel_paths: vec!["file_info.rs".into()], ..directory_root }, file_root], &AtomicBool::new(false), None);
        assert_eq!(full.selected_items, 2);
        assert_eq!(partial.selected_items, 2);
        assert_eq!(full.dirs, partial.dirs);
        assert_eq!(full.files, partial.files + 1);
        assert_eq!(full.total_bytes, partial.total_bytes + fs::metadata(excluded_file).unwrap().len());
        assert!(!full.incomplete && !partial.incomplete);
    }
}

#[tauri::command]
pub fn cancel_selection_stats(request_id: String) {
    if let Ok(calculations) = CALCULATIONS.lock() {
        if let Some(cancel) = calculations.get(&request_id) { cancel.store(true, Ordering::Relaxed); }
    }
}
