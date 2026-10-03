use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::FileEntry;

use super::ExtractProgress;
use super::encoding::decode_tar_name;
use super::shared::{collect_entries_at_path, normalize_internal};

// ── TAR ────────────────────────────────────────────────────────────

fn list_entries_impl<R: Read>(
    archive: &mut tar::Archive<R>,
    path: &str,
    internal: &str,
) -> Result<Vec<FileEntry>, String> {
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = HashSet::new();
    let mut size_map: HashMap<String, u64> = HashMap::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
        let norm = path_str.replace('\\', "/");

        if entry.header().entry_type().is_dir() {
            all_dirs.insert(norm.trim_end_matches('/').to_string());
        } else {
            let size = entry.header().size().unwrap_or(0);
            size_map.insert(norm.trim_end_matches('/').to_string(), size);
            all_paths.push(norm);
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

    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal), Some(&size_map)))
}

pub(crate) fn list_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    list_entries_impl(&mut archive, path, internal)
}

fn total_size_impl<R: Read>(archive: &mut tar::Archive<R>) -> Result<u64, String> {
    let entries = archive.entries().map_err(|e| format!("Failed to read tar: {}", e))?;
    let mut total: u64 = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        if !entry.header().entry_type().is_dir() {
            total += entry.header().size().unwrap_or(0);
        }
    }
    Ok(total)
}

pub(crate) fn total_uncompressed_size(path: &str) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    total_size_impl(&mut archive)
}

pub(crate) fn total_gz_uncompressed_size(path: &str) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    total_size_impl(&mut archive)
}

pub(crate) fn list_gz_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    list_entries_impl(&mut archive, path, internal)
}

fn read_file_impl<R: Read>(
    archive: &mut tar::Archive<R>,
    path: &str,
    internal: &str,
) -> Result<Vec<u8>, String> {
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;
    let internal = normalize_internal(internal);

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry)
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string();
        if path_str == internal {
            if entry.header().entry_type().is_dir() {
                return Err(format!("'{}' is a directory, not a file", internal));
            }
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read entry: {}", e))?;
            return Ok(buf);
        }
    }
    Err(format!("Entry not found in archive: {}", internal))
}

pub(crate) fn read_file(path: &str, internal: &str) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    read_file_impl(&mut archive, path, internal)
}

pub(crate) fn read_gz_file(path: &str, internal: &str) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    read_file_impl(&mut archive, path, internal)
}

fn extract_files_impl<R: Read>(
    archive: &mut tar::Archive<R>,
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    on_progress: &ExtractProgress,
) -> Result<(), String> {
    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
        let norm = path_str.replace('\\', "/").trim_end_matches('/').to_string();

        if target_set.contains(&norm) {
            let dest = Path::new(dest_dir).join(&path_str);
            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&dest)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dir: {}", e))?;
                }
                let mut out = File::create(&dest)
                    .map_err(|e| format!("Failed to create file: {}", e))?;
                let mut buf = vec![0u8; 65536];
                loop {
                    let n = entry.read(&mut buf).map_err(|e| format!("Failed to read entry: {}", e))?;
                    if n == 0 { break; }
                    out.write_all(&buf[..n])
                        .map_err(|e| format!("Failed to write: {}", e))?;
                    if !on_progress(n as u64) {
                        return Err("Cancelled".to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn extract_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    on_progress: &ExtractProgress,
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    extract_files_impl(&mut archive, path, internal_paths, dest_dir, on_progress)
}

pub(crate) fn extract_gz_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    on_progress: &ExtractProgress,
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    extract_files_impl(&mut archive, path, internal_paths, dest_dir, on_progress)
}

fn extract_all_impl<R: Read>(
    archive: &mut tar::Archive<R>,
    path: &str,
    dest_dir: &str,
    skip_paths: Option<&std::collections::HashSet<String>>,
    on_progress: &ExtractProgress,
) -> Result<u64, String> {
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;

    let mut total_bytes: u64 = 0;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
        let norm = path_str.replace('\\', "/").trim_end_matches('/').to_string();
        if let Some(skip) = skip_paths {
            if skip.contains(&norm) && !entry.header().entry_type().is_dir() {
                continue;
            }
        }
        let dest = Path::new(dest_dir).join(&path_str);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&dest)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            let mut out =
                File::create(&dest).map_err(|e| format!("Failed to create file: {}", e))?;
            let mut buf = vec![0u8; 65536];
            loop {
                let n = entry.read(&mut buf).map_err(|e| format!("Failed to read entry: {}", e))?;
                if n == 0 { break; }
                out.write_all(&buf[..n])
                    .map_err(|e| format!("Failed to write: {}", e))?;
                total_bytes += n as u64;
                if !on_progress(n as u64) {
                    return Err("Cancelled".to_string());
                }
            }
        }
    }
    Ok(total_bytes)
}

pub(crate) fn extract_all(path: &str, dest_dir: &str, skip_paths: Option<&std::collections::HashSet<String>>, on_progress: &ExtractProgress) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    extract_all_impl(&mut archive, path, dest_dir, skip_paths, on_progress)
}

pub(crate) fn extract_gz_all(path: &str, dest_dir: &str, skip_paths: Option<&std::collections::HashSet<String>>, on_progress: &ExtractProgress) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    extract_all_impl(&mut archive, path, dest_dir, skip_paths, on_progress)
}
