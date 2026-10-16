use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::FileEntry;

use super::ExtractProgress;
use super::password::{
    password_incorrect_error, password_required_error, remember_archive_password,
};
use super::shared::{collect_entries_at_path, normalize_internal};

/// Entries at or above this uncompressed size are logged at info level so they
/// remain visible at the default log level.
const LARGE_ENTRY_THRESHOLD: u64 = 256 * 1024 * 1024;

// ── RAR Helpers ─────────────────────────────────────────────────────

fn open_rar_archive<'a>(path: &'a str, password: Option<&'a str>) -> unrar::Archive<'a> {
    match password {
        Some(pw) => unrar::Archive::with_password(path, pw),
        None => unrar::Archive::new(path),
    }
}

fn map_unrar_error(path: &str, err: unrar::error::UnrarError) -> String {
    match err.code {
        unrar::error::Code::MissingPassword => password_required_error(path),
        unrar::error::Code::BadPassword => password_incorrect_error(path),
        _ => {
            log::error!("[archive] rar operation failed: {}", err);
            format!("Failed to read rar: {}", err)
        }
    }
}

fn log_entry_extract(name: &str, size: u64) {
    if size >= LARGE_ENTRY_THRESHOLD {
        log::info!("[archive] extract entry: {} size={}", name, size);
    } else {
        log::debug!("[archive] extract entry: {} size={}", name, size);
    }
}

// ── RAR Operations ──────────────────────────────────────────────────

pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let _guard = super::try_lock_busy().ok_or_else(|| super::archive_busy_error(path))?;
    let archive = open_rar_archive(path, password);
    let listing = archive
        .open_for_listing()
        .map_err(|e| map_unrar_error(path, e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = HashSet::new();
    let mut size_map: HashMap<String, u64> = HashMap::new();

    for entry in listing {
        let entry = entry.map_err(|e| map_unrar_error(path, e))?;
        let path_str = entry.filename.to_string_lossy().replace('\\', "/");

        if entry.is_directory() {
            all_dirs.insert(path_str.trim_end_matches('/').to_string());
        } else {
            let norm = path_str.trim_end_matches('/').to_string();
            size_map.insert(norm, entry.unpacked_size as u64);
            all_paths.push(path_str.clone());
            let mut parent = Path::new(&path_str).parent();
            while let Some(p) = parent {
                let s = p.to_string_lossy().replace('\\', "/");
                if s.is_empty() {
                    break;
                }
                all_dirs.insert(s);
                parent = p.parent();
            }
        }
    }

    remember_archive_password(path, password);
    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal), Some(&size_map)))
}

pub(crate) fn total_uncompressed_size(path: &str, password: Option<&str>) -> Result<u64, String> {
    let _guard = super::try_lock_busy().ok_or_else(|| super::archive_busy_error(path))?;
    let archive = open_rar_archive(path, password);
    let listing = archive.open_for_listing().map_err(|e| map_unrar_error(path, e))?;

    let mut total: u64 = 0;
    for entry in listing {
        let entry = entry.map_err(|e| map_unrar_error(path, e))?;
        if !entry.is_directory() {
            total += entry.unpacked_size as u64;
        }
    }
    remember_archive_password(path, password);
    Ok(total)
}

pub(crate) fn read_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String> {
    let _guard = super::try_lock_busy().ok_or_else(|| super::archive_busy_error(path))?;
    let archive = open_rar_archive(path, password);
    let mut open_archive = archive
        .open_for_processing()
        .map_err(|e| map_unrar_error(path, e))?;
    let target = normalize_internal(internal);

    loop {
        let header = match open_archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => return Err(map_unrar_error(path, e)),
        };

        let norm = header.entry().filename.to_string_lossy().replace('\\', "/")
            .trim_end_matches('/').to_string();

        if norm == target && header.entry().is_file() {
            let (data, _) = header.read().map_err(|e| map_unrar_error(path, e))?;
            remember_archive_password(path, password);
            return Ok(data);
        }

        open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
    }

    remember_archive_password(path, password);
    Err(format!("Entry not found in archive: {}", internal))
}

pub(crate) fn extract_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<&str>,
    on_progress: &ExtractProgress,
    on_entry: Option<&super::EntryStartProgress>,
) -> Result<(), String> {
    let _guard = super::lock_blocking();
    let archive = open_rar_archive(path, password);
    let mut open_archive = archive
        .open_for_processing()
        .map_err(|e| map_unrar_error(path, e))?;
    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    loop {
        let header = match open_archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => return Err(map_unrar_error(path, e)),
        };

        let filename = header.entry().filename.clone();
        let norm = filename.to_string_lossy().replace('\\', "/")
            .trim_end_matches('/').to_string();

        if !target_set.contains(&norm) {
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
            continue;
        }

        let entry_size = header.entry().unpacked_size;
        let is_dir = header.entry().is_directory();
        let dest = Path::new(dest_dir).join(&filename);

        if is_dir {
            fs::create_dir_all(&dest).map_err(|e| {
                log::error!("[archive] extract_files failed: {}", e);
                format!("Failed to create dir: {}", e)
            })?;
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
            continue;
        }

        // Cancellation is only observable between entries: unrar's extract path
        // offers no in-flight abort.
        if !on_progress(0) {
            return Err("Cancelled".to_string());
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                log::error!("[archive] extract_files failed: {}", e);
                format!("Failed to create dir: {}", e)
            })?;
        }

        log_entry_extract(&filename.to_string_lossy(), entry_size);
        if let Some(notify) = on_entry {
            notify(&filename.to_string_lossy(), entry_size);
        }
        open_archive = header.extract_with_base(dest_dir).map_err(|e| map_unrar_error(path, e))?;
        on_progress(entry_size);
    }

    remember_archive_password(path, password);
    Ok(())
}

pub(crate) fn extract_all(
    path: &str,
    dest_dir: &str,
    password: Option<&str>,
    skip_paths: Option<&HashSet<String>>,
    on_progress: &ExtractProgress,
    on_entry: Option<&super::EntryStartProgress>,
) -> Result<u64, String> {
    let _guard = super::lock_blocking();
    let archive = open_rar_archive(path, password);
    let mut open_archive = archive
        .open_for_processing()
        .map_err(|e| map_unrar_error(path, e))?;
    let mut total_bytes: u64 = 0;
    let mut entry_count: u64 = 0;

    loop {
        let header = match open_archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => return Err(map_unrar_error(path, e)),
        };

        let filename = header.entry().filename.clone();
        let norm = filename.to_string_lossy().replace('\\', "/")
            .trim_end_matches('/').to_string();

        if skip_paths.map_or(false, |skip| skip.contains(&norm)) {
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
            continue;
        }

        let entry_size = header.entry().unpacked_size;
        let is_dir = header.entry().is_directory();
        let dest = Path::new(dest_dir).join(&filename);

        if is_dir {
            fs::create_dir_all(&dest).map_err(|e| {
                log::error!("[archive] extract_all failed: {}", e);
                format!("Failed to create dir: {}", e)
            })?;
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
            continue;
        }

        // Cancellation is only observable between entries: unrar's extract path
        // offers no in-flight abort.
        if !on_progress(0) {
            return Err("Cancelled".to_string());
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                log::error!("[archive] extract_all failed: {}", e);
                format!("Failed to create dir: {}", e)
            })?;
        }

        log_entry_extract(&filename.to_string_lossy(), entry_size);
        if let Some(notify) = on_entry {
            notify(&filename.to_string_lossy(), entry_size);
        }
        open_archive = header.extract_with_base(dest_dir).map_err(|e| map_unrar_error(path, e))?;
        total_bytes += entry_size;
        entry_count += 1;
        on_progress(entry_size);
    }

    log::debug!("[archive] extract_all complete: entries={} bytes={}", entry_count, total_bytes);
    remember_archive_password(path, password);
    Ok(total_bytes)
}
