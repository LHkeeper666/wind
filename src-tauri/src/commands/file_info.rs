use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[tauri::command]
pub fn get_home_dir() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\".to_string())
}

#[tauri::command]
pub fn file_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[derive(Serialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: u64,
    pub modified_ns: String,
}

#[tauri::command]
pub fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let modified_time = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
    Ok(FileMetadata {
        size: metadata.len(),
        modified: modified_time.map(|d| d.as_secs()).unwrap_or(0),
        modified_ns: modified_time.map(|d| d.as_nanos().to_string()).unwrap_or_default(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub created: Option<i64>,
    pub modified: Option<i64>,
    pub accessed: Option<i64>,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub is_system: bool,
    pub item_count: Option<usize>,
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    log::debug!("[file_info] get_file_info: {}", path);
    let file_path = Path::new(&path);
    if !file_path.exists() {
        log::error!("[file_info] get_file_info failed: file does not exist: {}", path);
        return Err(format!("File does not exist: {}", path));
    }

    let metadata =
        fs::metadata(&path).map_err(|e| {
            log::error!("[file_info] get_file_info failed: {}", e);
            format!("Failed to get file metadata: {}", e)
        })?;

    let name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    use std::os::windows::fs::MetadataExt;
    let attrs = metadata.file_attributes();
    let is_hidden = name.starts_with('.') || (attrs & 0x2 != 0);
    let is_system = attrs & 0x4 != 0;

    let created = metadata.created().ok().and_then(unix_millis);
    let modified = metadata.modified().ok().and_then(unix_millis);
    let accessed = metadata.accessed().ok().and_then(unix_millis);

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

fn unix_millis(time: SystemTime) -> Option<i64> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_millis()).ok(),
        Err(error) => i64::try_from(error.duration().as_millis()).ok()?.checked_neg(),
    }
}

#[cfg(test)]
mod tests {
    use super::unix_millis;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn converts_times_on_both_sides_of_epoch() {
        assert_eq!(unix_millis(UNIX_EPOCH + Duration::from_millis(1_234)), Some(1_234));
        assert_eq!(unix_millis(UNIX_EPOCH - Duration::from_millis(1_234)), Some(-1_234));
    }
}
