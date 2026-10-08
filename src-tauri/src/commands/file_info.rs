use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
}

#[tauri::command]
pub fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(FileMetadata {
        size: metadata.len(),
        modified,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
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
    let s = secs as i64;
    let days = s / 86400;
    let time_of_day = s % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

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
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    while m < 12 && remaining_days >= month_days[m] {
        remaining_days -= month_days[m];
        m += 1;
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y,
        m + 1,
        remaining_days + 1,
        hour,
        minute,
        second
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
