use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    log::info!("[file_ops] delete_file: {}", path);
    let file_path = Path::new(&path);

    if !file_path.exists() {
        log::error!("[file_ops] delete_file failed: path does not exist: {}", path);
        return Err(format!("Path does not exist: {}", path));
    }

    trash::delete(file_path).map_err(|e| {
        log::error!("[file_ops] delete_file failed: {}", e);
        format!("Failed to move to trash: {}", e)
    })
}

#[tauri::command]
pub fn rename_file(old_path: String, new_name: String) -> Result<String, String> {
    log::info!("[file_ops] rename_file: {} -> {}", old_path, new_name);
    let old = Path::new(&old_path);

    if !old.exists() {
        log::error!("[file_ops] rename_file failed: path does not exist: {}", old_path);
        return Err(format!("Path does not exist: {}", old_path));
    }

    let parent = old
        .parent()
        .ok_or_else(|| "Cannot get parent directory".to_string())?;

    let new_path = parent.join(&new_name);

    if new_path.exists() {
        log::error!("[file_ops] rename_file failed: destination already exists: {}", new_name);
        return Err(format!(
            "A file or directory with name '{}' already exists",
            new_name
        ));
    }

    fs::rename(old, &new_path).map_err(|e| {
        log::error!("[file_ops] rename_file failed: {}", e);
        format!("Failed to rename: {}", e)
    })?;

    Ok(new_path.to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameEntry {
    pub old_path: String,
    pub new_name: String,
}

#[tauri::command]
pub fn batch_rename(entries: Vec<RenameEntry>) -> Result<Vec<String>, String> {
    log::info!("[file_ops] batch_rename: {} entries", entries.len());
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
        log::error!("[file_ops] batch_rename failed: {}", errors.join("; "));
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub fn create_batch_rename_temp_file(files: Vec<String>) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("wind_batch_rename.txt");
    fs::write(&temp_file, files.join("\n"))
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    Ok(temp_file.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_temp_file(path: String) -> Result<(), String> {
    let _ = fs::remove_file(path);
    Ok(())
}

#[tauri::command]
pub fn create_file(path: String, is_dir: bool) -> Result<(), String> {
    log::info!("[file_ops] create_file: {} (is_dir={})", path, is_dir);
    let file_path = Path::new(&path);

    if file_path.exists() {
        log::error!("[file_ops] create_file failed: path already exists: {}", path);
        return Err(format!("A file or directory already exists at: {}", path));
    }

    if is_dir {
        fs::create_dir_all(file_path).map_err(|e| {
            log::error!("[file_ops] create_file failed: {}", e);
            format!("Failed to create directory: {}", e)
        })
    } else {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directories: {}", e))?;
            }
        }
        fs::write(file_path, "").map_err(|e| {
            log::error!("[file_ops] create_file failed: {}", e);
            format!("Failed to create file: {}", e)
        })
    }
}
