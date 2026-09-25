use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

static ARCHIVE_PASSWORD_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn archive_cache_key(archive_path: &str) -> String {
    let path = Path::new(archive_path);
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
