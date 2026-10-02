use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

static ARCHIVE_PASSWORD_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn is_rar_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    if lower.ends_with(".rar") {
        return true;
    }
    // .rXX pattern (e.g., .r00, .r01)
    if lower.len() >= 4 {
        let tail = &lower[lower.len() - 4..];
        if tail.starts_with(".r") && tail[2..].chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    // .partN.rar pattern
    if let Some(idx) = lower.rfind(".part") {
        let after = &lower[idx + 5..];
        if after.ends_with(".rar") && after[..after.len() - 4].chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

fn rar_first_part(path: &Path) -> std::path::PathBuf {
    let s = path.to_string_lossy().to_string();
    let lower = s.to_lowercase();
    // .partN.rar → .part1.rar
    if let Some(idx) = lower.rfind(".part") {
        let after = &lower[idx + 5..];
        if after.ends_with(".rar") && after[..after.len() - 4].chars().all(|c| c.is_ascii_digit()) {
            let prefix = &s[..idx];
            return std::path::PathBuf::from(format!("{}.part1.rar", prefix));
        }
    }
    // .rXX → .rar (first volume)
    if lower.len() >= 4 {
        let tail = &lower[lower.len() - 4..];
        if tail.starts_with(".r") && tail[2..].chars().all(|c| c.is_ascii_digit()) {
            let base = &s[..s.len() - 4];
            return std::path::PathBuf::from(format!("{}.rar", base));
        }
    }
    path.to_path_buf()
}

fn archive_cache_key(archive_path: &str) -> String {
    let path = Path::new(archive_path);
    // For multipart RAR, normalize to first volume so all volumes share the password cache
    if is_rar_path(path) {
        let first = rar_first_part(path);
        let canonical = fs::canonicalize(&first).unwrap_or(first);
        return format!("rar:{}", canonical.to_string_lossy());
    }
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let metadata = fs::metadata(path).ok();
    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = metadata
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}|{}|{}", canonical.to_string_lossy(), size, modified)
}

fn cached_archive_password(archive_path: &str) -> Option<String> {
    let key = archive_cache_key(archive_path);
    let mut cache = ARCHIVE_PASSWORD_CACHE.lock().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);
    map.get(&key).cloned()
}

fn store_archive_password(archive_path: &str, password: &str) {
    let key = archive_cache_key(archive_path);
    let mut cache = ARCHIVE_PASSWORD_CACHE.lock().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);
    map.insert(key, password.to_string());
}

pub(crate) fn resolve_archive_password(archive_path: &str, password: Option<String>) -> Option<String> {
    password.or_else(|| cached_archive_password(archive_path))
}

pub(crate) fn remember_archive_password(archive_path: &str, password: Option<&str>) {
    if let Some(password) = password {
        store_archive_password(archive_path, password);
    }
}

pub(crate) fn password_required_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_REQUIRED: {}", archive_path)
}

pub(crate) fn password_incorrect_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_INCORRECT: {}", archive_path)
}
