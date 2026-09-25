use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::FileEntry;

use super::password::{
    password_incorrect_error, password_required_error, remember_archive_password,
};
use super::shared::{collect_entries_at_path, normalize_internal};

// ── 7z Helpers ─────────────────────────────────────────────────────

fn make_7z_password(password: Option<&str>) -> sevenz_rust::Password {
    match password {
        Some(password) => sevenz_rust::Password::from(password),
        None => sevenz_rust::Password::empty(),
    }
}

fn map_7z_error(path: &str, err: sevenz_rust::Error) -> String {
    match err {
        sevenz_rust::Error::PasswordRequired => password_required_error(path),
        sevenz_rust::Error::MaybeBadPassword(_) => password_incorrect_error(path),
        other => format!("Failed to read 7z: {}", other),
    }
}

// ── 7z Operations ──────────────────────────────────────────────────

pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, file_len, make_7z_password(password))
        .map_err(|e| map_7z_error(path, e))?;

    reader
        .for_each_entries(|entry, reader| {
            if !entry.is_directory && entry.has_stream && entry.size > 0 {
                std::io::copy(reader, &mut std::io::sink()).map_err(sevenz_rust::Error::io)?;
                return Ok(false);
            }
            Ok(true)
        })
        .map_err(|e| map_7z_error(path, e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = HashSet::new();
    for entry in &reader.archive().files {
        let norm = entry.name.replace('\\', "/");
        if entry.is_directory {
            all_dirs.insert(norm.trim_end_matches('/').to_string());
        } else {
            all_paths.push(norm);
            let mut parent = Path::new(&entry.name).parent();
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
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, file_len, make_7z_password(password))
        .map_err(|e| map_7z_error(path, e))?;
    let internal = normalize_internal(internal);

    let mut result: Option<Vec<u8>> = None;
    reader
        .for_each_entries(|entry, reader| {
            if result.is_some() {
                return Ok(false);
            }
            let norm = entry.name.replace('\\', "/").trim_end_matches('/').to_string();
            if norm == internal && !entry.is_directory {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                result = Some(buf);
                return Ok(false);
            }
            if !entry.is_directory {
                std::io::copy(reader, &mut std::io::sink()).map_err(sevenz_rust::Error::io)?;
            }
            Ok(true)
        })
        .map_err(|e| map_7z_error(path, e))?;

    remember_archive_password(path, password);
    result.ok_or_else(|| format!("Entry not found in archive: {}", internal))
}

pub(crate) fn extract_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<&str>,
) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, file_len, make_7z_password(password))
        .map_err(|e| map_7z_error(path, e))?;
    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();
    let mut remaining_targets = target_set.clone();

    reader
        .for_each_entries(|entry, reader| {
            if remaining_targets.is_empty() {
                return Ok(false);
            }
            let norm = entry.name.replace('\\', "/").trim_end_matches('/').to_string();
            if target_set.contains(&norm) {
                let dest = Path::new(dest_dir).join(&entry.name);
                if entry.is_directory {
                    fs::create_dir_all(&dest)?;
                    remaining_targets.remove(&norm);
                } else {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut buf = Vec::new();
                    reader.read_to_end(&mut buf)?;
                    let mut out = File::create(&dest)?;
                    out.write_all(&buf)?;
                    remaining_targets.remove(&norm);
                }
            } else if !entry.is_directory {
                std::io::copy(reader, &mut std::io::sink()).map_err(sevenz_rust::Error::io)?;
            }
            Ok(true)
        })
        .map_err(|e| map_7z_error(path, e))?;
    remember_archive_password(path, password);
    Ok(())
}

pub(crate) fn extract_all(path: &str, dest_dir: &str, password: Option<&str>) -> Result<u64, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, file_len, make_7z_password(password))
        .map_err(|e| map_7z_error(path, e))?;
    let mut total_bytes: u64 = 0;

    reader
        .for_each_entries(|entry, reader| {
            let dest = Path::new(dest_dir).join(&entry.name);
            if entry.is_directory {
                fs::create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                total_bytes += buf.len() as u64;
                let mut out = File::create(&dest)?;
                out.write_all(&buf)?;
            }
            Ok(true)
        })
        .map_err(|e| map_7z_error(path, e))?;
    remember_archive_password(path, password);
    Ok(total_bytes)
}
