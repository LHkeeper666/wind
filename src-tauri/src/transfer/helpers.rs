use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    collect_dir_size(path, &mut total);
    total
}

fn collect_dir_size(dir: &Path, total: &mut u64) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_dir_size(&path, total);
            } else if let Ok(meta) = path.metadata() {
                *total += meta.len();
            }
        }
    }
}

pub fn extract_ftp_conn(path: &str) -> Option<String> {
    if path.starts_with("ftp://") {
        let rest = &path[6..];
        rest.split('/').next().map(|s| s.to_string())
    } else {
        None
    }
}

pub fn check_cancelled(cancel_flag: &Arc<AtomicBool>) -> bool {
    cancel_flag.load(Ordering::Relaxed)
}