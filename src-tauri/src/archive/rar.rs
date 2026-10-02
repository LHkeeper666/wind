use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::FileEntry;

use super::password::{
    password_incorrect_error, password_required_error, remember_archive_password,
};
use super::shared::{collect_entries_at_path, normalize_internal};

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
        _ => format!("Failed to read rar: {}", err),
    }
}

// ── RAR Operations ──────────────────────────────────────────────────

pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let archive = open_rar_archive(path, password).as_first_part();
    let listing = archive
        .open_for_listing()
        .map_err(|e| map_unrar_error(path, e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = HashSet::new();

    for entry in listing {
        let entry = entry.map_err(|e| map_unrar_error(path, e))?;
        let path_str = entry.filename.to_string_lossy().replace('\\', "/");

        if entry.is_directory() {
            all_dirs.insert(path_str.trim_end_matches('/').to_string());
        } else {
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
    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal)))
}

pub(crate) fn read_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String> {
    let archive = open_rar_archive(path, password).as_first_part();
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
) -> Result<(), String> {
    let archive = open_rar_archive(path, password).as_first_part();
    let mut open_archive = archive
        .open_for_processing()
        .map_err(|e| map_unrar_error(path, e))?;
    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();
    let mut remaining_targets = target_set.clone();

    loop {
        let header = match open_archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => return Err(map_unrar_error(path, e)),
        };

        let norm = header.entry().filename.to_string_lossy().replace('\\', "/")
            .trim_end_matches('/').to_string();

        if target_set.contains(&norm) {
            let dest = Path::new(dest_dir).join(header.entry().filename.as_path());
            if header.entry().is_directory() {
                fs::create_dir_all(&dest).map_err(|e| format!("Failed to create dir: {}", e))?;
                remaining_targets.remove(&norm);
                open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
                }
                let (data, archive_after_read) = header.read().map_err(|e| map_unrar_error(path, e))?;
                let mut out = File::create(&dest).map_err(|e| format!("Failed to create file: {}", e))?;
                out.write_all(&data).map_err(|e| format!("Failed to write file: {}", e))?;
                remaining_targets.remove(&norm);
                open_archive = archive_after_read;
            }
        } else {
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
        }
    }

    remember_archive_password(path, password);
    Ok(())
}

pub(crate) fn extract_all(
    path: &str,
    dest_dir: &str,
    password: Option<&str>,
    skip_paths: Option<&HashSet<String>>,
) -> Result<u64, String> {
    let archive = open_rar_archive(path, password).as_first_part();
    let mut open_archive = archive
        .open_for_processing()
        .map_err(|e| map_unrar_error(path, e))?;
    let mut total_bytes: u64 = 0;

    loop {
        let header = match open_archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => return Err(map_unrar_error(path, e)),
        };

        let norm = header.entry().filename.to_string_lossy().replace('\\', "/")
            .trim_end_matches('/').to_string();

        if let Some(skip) = skip_paths {
            if skip.contains(&norm) {
                open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
                continue;
            }
        }

        let dest = Path::new(dest_dir).join(header.entry().filename.as_path());
        if header.entry().is_directory() {
            fs::create_dir_all(&dest).map_err(|e| format!("Failed to create dir: {}", e))?;
            open_archive = header.skip().map_err(|e| map_unrar_error(path, e))?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
            }
            let (data, archive_after_read) = header.read().map_err(|e| map_unrar_error(path, e))?;
            total_bytes += data.len() as u64;
            let mut out = File::create(&dest).map_err(|e| format!("Failed to create file: {}", e))?;
            out.write_all(&data).map_err(|e| format!("Failed to write file: {}", e))?;
            open_archive = archive_after_read;
        }
    }

    remember_archive_password(path, password);
    Ok(total_bytes)
}