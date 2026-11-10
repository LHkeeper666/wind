use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static RENAME_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn entry_named(parent: &Path, name: &std::ffi::OsStr) -> io::Result<bool> {
    for entry in fs::read_dir(parent)? {
        if entry?.file_name() == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn occupied(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

fn temporary_rename_path(parent: &Path) -> Result<PathBuf, String> {
    for _ in 0..100 {
        let sequence = RENAME_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let candidate = parent.join(format!(
            ".wind-rename-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        if !occupied(&candidate).map_err(|e| e.to_string())? {
            return Ok(candidate);
        }
    }
    Err("Could not find a free temporary rename path".to_string())
}

fn rename_local_with<F>(old: &Path, new_name: &str, mut rename: F) -> Result<PathBuf, String>
where
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    if !occupied(old).map_err(|e| e.to_string())? {
        return Err(format!("Path does not exist: {}", old.display()));
    }
    let parent = old.parent().ok_or("Cannot get parent directory")?;
    let original_name = old.file_name().ok_or("Cannot get original name")?;
    let new_path = parent.join(new_name);
    if original_name == new_name {
        return Ok(new_path);
    }

    // An exact directory entry is a collision even when it is a hard link to the source.
    if entry_named(parent, new_path.file_name().ok_or("Invalid new name")?)
        .map_err(|e| e.to_string())?
    {
        return Err(format!(
            "A file or directory with name '{new_name}' already exists"
        ));
    }
    let case_alias = if occupied(&new_path).map_err(|e| e.to_string())? {
        if !same_file::is_same_file(old, &new_path).map_err(|e| e.to_string())? {
            return Err(format!(
                "A file or directory with name '{new_name}' already exists"
            ));
        }
        true
    } else {
        false
    };

    let direct = rename(old, &new_path);
    if direct.is_ok()
        && entry_named(parent, new_path.file_name().unwrap()).map_err(|e| e.to_string())?
    {
        return Ok(new_path);
    }
    if !case_alias {
        return Err(direct
            .err()
            .map(|e| format!("Failed to rename: {e}"))
            .unwrap_or_else(|| "Rename did not apply the requested name".to_string()));
    }

    // Some Windows filesystems accept a case-only rename without updating stored casing.
    let temp = temporary_rename_path(parent)?;
    rename(old, &temp).map_err(|e| format!("Failed to rename through temporary path: {e}"))?;
    if occupied(&new_path).map_err(|e| e.to_string())? {
        let error = format!("Destination became occupied: {}", new_path.display());
        return Err(restore_after_failed_rename(&mut rename, &temp, old, error));
    }
    if let Err(e) = rename(&temp, &new_path) {
        return Err(restore_after_failed_rename(
            &mut rename,
            &temp,
            old,
            format!("Failed to finish rename: {e}"),
        ));
    }
    if !entry_named(parent, new_path.file_name().unwrap()).map_err(|e| e.to_string())? {
        return Err(format!(
            "Rename returned success but requested name is absent: {}",
            new_path.display()
        ));
    }
    Ok(new_path)
}

fn restore_after_failed_rename<F>(rename: &mut F, temp: &Path, old: &Path, error: String) -> String
where
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    match rename(temp, old) {
        Ok(()) => error,
        Err(recovery) => format!(
            "{error}; restore failed: {recovery}; item remains at {}",
            temp.display()
        ),
    }
}

fn rename_local(old: &Path, new_name: &str) -> Result<PathBuf, String> {
    rename_local_with(old, new_name, |from, to| fs::rename(from, to))
}

#[cfg(test)]
mod rename_tests {
    use super::*;

    #[test]
    fn case_only_file_and_directory_renames() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("Report.txt");
        fs::write(&file, b"content").unwrap();
        let renamed = rename_local(&file, "report.txt").unwrap();
        assert_eq!(renamed.file_name().unwrap(), "report.txt");
        assert!(entry_named(root.path(), "report.txt".as_ref()).unwrap());
        let renamed = rename_local(&renamed, "REPORT.txt").unwrap();
        assert!(entry_named(root.path(), "REPORT.txt".as_ref()).unwrap());
        assert_eq!(fs::read(&renamed).unwrap(), b"content");

        let dir = root.path().join("Folder");
        fs::create_dir(&dir).unwrap();
        let dir = rename_local(&dir, "folder").unwrap();
        assert!(entry_named(root.path(), "folder".as_ref()).unwrap());
        let dir = rename_local(&dir, "FOLDER").unwrap();
        assert!(entry_named(root.path(), "FOLDER".as_ref()).unwrap());
        assert!(dir.is_dir());
        assert_eq!(rename_local(&dir, "FOLDER").unwrap(), dir);
    }

    #[test]
    fn batch_rename_and_collision() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("First.txt");
        fs::write(&first, b"first").unwrap();
        let result = batch_rename(vec![RenameEntry {
            old_path: first.to_string_lossy().into_owned(),
            new_name: "first.txt".into(),
        }])
        .unwrap();
        assert_eq!(result.len(), 1);
        assert!(entry_named(root.path(), "first.txt".as_ref()).unwrap());

        let other = root.path().join("other.txt");
        fs::write(&other, b"other").unwrap();
        assert!(rename_local(&root.path().join("first.txt"), "other.txt").is_err());
        assert_eq!(fs::read(&other).unwrap(), b"other");

        let hardlink = root.path().join("linked.txt");
        fs::hard_link(root.path().join("first.txt"), &hardlink).unwrap();
        assert!(rename_local(&root.path().join("first.txt"), "linked.txt").is_err());
        assert!(hardlink.exists());
    }

    #[cfg(windows)]
    #[test]
    fn distinct_case_sensitive_entries_are_not_overwritten() {
        use std::process::Command;
        let root = tempfile::tempdir().unwrap();
        let status = Command::new("fsutil")
            .args(["file", "setCaseSensitiveInfo"])
            .arg(root.path())
            .arg("enable")
            .status();
        if !status.is_ok_and(|status| status.success()) {
            return; // This Windows installation does not permit case-sensitive directories.
        }
        let upper = root.path().join("Foo");
        let lower = root.path().join("foo");
        fs::write(&upper, b"upper").unwrap();
        fs::write(&lower, b"lower").unwrap();
        assert!(rename_local(&upper, "foo").is_err());
        assert_eq!(fs::read(&upper).unwrap(), b"upper");
        assert_eq!(fs::read(&lower).unwrap(), b"lower");
    }

    #[test]
    fn fallback_changes_casing_and_reports_failed_recovery() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("Name.txt");
        fs::write(&old, b"data").unwrap();
        let mut calls = 0;
        let new = rename_local_with(&old, "name.txt", |from, to| {
            calls += 1;
            if calls == 1 {
                Ok(())
            } else {
                fs::rename(from, to)
            }
        })
        .unwrap();
        assert_eq!(calls, 3);
        assert!(entry_named(root.path(), "name.txt".as_ref()).unwrap());
        let old = new;
        let mut calls = 0;
        let error = rename_local_with(&old, "NAME.txt", |from, to| {
            calls += 1;
            match calls {
                1 => Ok(()),
                2 => fs::rename(from, to),
                _ => Err(io::Error::other("injected failure")),
            }
        })
        .unwrap_err();
        assert!(error.contains("restore failed"));
        assert!(error.contains(".wind-rename-"));
    }
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    log::info!("[file_ops] delete_file: {}", path);
    let file_path = Path::new(&path);

    if !file_path.exists() {
        log::error!(
            "[file_ops] delete_file failed: path does not exist: {}",
            path
        );
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
    let new_path = rename_local(Path::new(&old_path), &new_name).map_err(|e| {
        log::error!("[file_ops] rename_file failed: {}", e);
        e
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
        match rename_local(Path::new(&entry.old_path), &entry.new_name) {
            Ok(_) => renamed.push(entry.old_path.clone()),
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
        log::error!(
            "[file_ops] create_file failed: path already exists: {}",
            path
        );
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
