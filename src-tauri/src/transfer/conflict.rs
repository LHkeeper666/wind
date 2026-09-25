use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

/// Recursively scan a source directory tree and emit a
/// `transfer-conflict-found` event for each conflicting file.
pub fn scan_dir_conflicts(src: &Path, dst: &Path, root: &Path, app: &AppHandle, skip_rel_paths: &[String]) {
    let Ok(entries) = fs::read_dir(src) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        let rel = path.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| name.to_string_lossy().to_string());
        if skip_rel_paths.iter().any(|excluded| rel == *excluded || rel.starts_with(&format!("{excluded}\\"))) {
            continue;
        }
        if path.is_dir() {
            scan_dir_conflicts(&path, &dst_path, root, app, skip_rel_paths);
        } else if dst_path.exists() {
            let _ = app.emit("transfer-conflict-found", serde_json::json!({
                "kind": "dir",
                "dir_source": root.to_string_lossy().to_string(),
                "source": path.to_string_lossy(),
                "destination": dst_path.to_string_lossy(),
                "rel_path": rel,
            }));
        }
    }
}

/// Recursively walk a local directory, collecting (full_path, size, is_dir).
pub fn walk_local_dir(
    current: &PathBuf,
    base: &Path,
    out: &mut Vec<(String, u64, bool)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        if path.is_dir() {
            out.push((path_str.clone(), 0, true));
            walk_local_dir(&path.to_path_buf(), base, out)?;
        } else {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            out.push((path_str, size, false));
        }
    }
    Ok(())
}