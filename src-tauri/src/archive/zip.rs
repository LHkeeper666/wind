use std::collections::{HashSet};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::FileEntry;

use super::encoding::{decode_name, detect_archive_encoding};
use super::password::{
    password_incorrect_error, password_required_error, remember_archive_password,
};
use super::shared::{collect_entries_at_path, normalize_internal};

// ── ZIP Error Mapping ──────────────────────────────────────────────

fn is_zip_password_error(err: &zip::result::ZipError) -> bool {
    matches!(
        err,
        zip::result::ZipError::InvalidPassword
            | zip::result::ZipError::UnsupportedArchive(zip::result::ZipError::PASSWORD_REQUIRED)
    )
}

fn map_zip_error(path: &str, err: zip::result::ZipError) -> String {
    if is_zip_password_error(&err) {
        match err {
            zip::result::ZipError::UnsupportedArchive(zip::result::ZipError::PASSWORD_REQUIRED) => {
                password_required_error(path)
            }
            zip::result::ZipError::InvalidPassword => password_incorrect_error(path),
            _ => format!("Failed to read zip: {}", err),
        }
    } else {
        format!("Failed to read zip: {}", err)
    }
}

// ── ZIP Password Helpers ───────────────────────────────────────────

fn zip_entry_requires_password(archive: &mut zip::ZipArchive<File>) -> Result<Option<usize>, String> {
    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry.encrypted() {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

fn validate_zip_password(
    archive: &mut zip::ZipArchive<File>,
    path: &str,
    password: &str,
) -> Result<(), String> {
    let Some(index) = zip_entry_requires_password(archive)? else {
        return Ok(());
    };
    archive
        .by_index_decrypt(index, password.as_bytes())
        .map_err(|e| map_zip_error(path, e))?;
    Ok(())
}

fn read_zip_entry<'a>(
    archive: &'a mut zip::ZipArchive<File>,
    path: &str,
    index: usize,
    password: Option<&str>,
) -> Result<zip::read::ZipFile<'a>, String> {
    match password {
        Some(password) => archive
            .by_index_decrypt(index, password.as_bytes())
            .map_err(|e| map_zip_error(path, e)),
        None => archive
            .by_index(index)
            .map_err(|e| map_zip_error(path, e)),
    }
}

// ── Encoding: ZIP Central Directory ────────────────────────────────

/// Parsed ZIP central directory entry with raw filename bytes
struct ZipCdEntry {
    name_bytes: Vec<u8>,
    utf8_flag: bool,
}

/// Parse ZIP central directory to extract raw filename bytes and UTF-8 flags.
/// Returns a Vec of ZipCdEntry.
fn parse_zip_central_dir(path: &str) -> Result<Vec<ZipCdEntry>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.seek(SeekFrom::End(0)).map_err(|e| format!("Failed to seek: {}", e))?;

    let eocd_offset = find_eocd(&mut file, file_len)?;
    let cd_offset = read_cd_info(&mut file, eocd_offset)?;

    if cd_offset >= file_len {
        return Err("Invalid central directory offset".to_string());
    }

    file.seek(SeekFrom::Start(cd_offset))
        .map_err(|e| format!("Failed to seek to central directory: {}", e))?;

    let mut entries = Vec::new();
    let cd_sig: u32 = 0x02014b50;

    loop {
        let mut sig_buf = [0u8; 4];
        if file.read_exact(&mut sig_buf).is_err() {
            break;
        }
        let sig = u32::from_le_bytes(sig_buf);
        if sig != cd_sig {
            break;
        }

        let mut header = [0u8; 42];
        file.read_exact(&mut header)
            .map_err(|e| format!("Failed to read CD header: {}", e))?;

        let gp_flag = u16::from_le_bytes([header[4], header[5]]);
        let name_len = u16::from_le_bytes([header[24], header[25]]) as usize;
        let extra_len = u16::from_le_bytes([header[26], header[27]]) as usize;
        let comment_len = u16::from_le_bytes([header[28], header[29]]) as usize;

        let utf8_flag = (gp_flag & 0x0800) != 0;

        let mut name_bytes = vec![0u8; name_len];
        file.read_exact(&mut name_bytes)
            .map_err(|e| format!("Failed to read filename: {}", e))?;

        if extra_len > 0 {
            file.seek(SeekFrom::Current(extra_len as i64))
                .map_err(|e| format!("Failed to skip extra field: {}", e))?;
        }
        if comment_len > 0 {
            file.seek(SeekFrom::Current(comment_len as i64))
                .map_err(|e| format!("Failed to skip comment: {}", e))?;
        }

        entries.push(ZipCdEntry {
            name_bytes,
            utf8_flag,
        });
    }

    Ok(entries)
}

fn find_eocd(file: &mut File, file_len: u64) -> Result<u64, String> {
    let search_start = if file_len > 65557 { file_len - 65557 } else { 0 };
    let search_len = file_len - search_start;
    file.seek(SeekFrom::Start(search_start))
        .map_err(|e| format!("Failed to seek: {}", e))?;

    let mut buf = vec![0u8; search_len as usize];
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read EOCD search area: {}", e))?;

    let eocd_sig: u32 = 0x06054b50;
    for i in (0..buf.len().saturating_sub(4)).rev() {
        let sig = u32::from_le_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
        if sig == eocd_sig {
            return Ok(search_start + i as u64);
        }
    }
    Err("EOCD not found in ZIP file".to_string())
}

fn read_cd_info(file: &mut File, eocd_offset: u64) -> Result<u64, String> {
    file.seek(SeekFrom::Start(eocd_offset + 16))
        .map_err(|e| format!("Failed to seek to CD info: {}", e))?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read CD offset: {}", e))?;
    Ok(u64::from(u32::from_le_bytes(buf)))
}

/// Decode entry name using central directory raw bytes and encoding detection.
fn decode_cd_entry_name(
    cd_entries: &[ZipCdEntry],
    index: usize,
    entry: &zip::read::ZipFile<'_>,
    archive_path: &str,
) -> String {
    if let Some(cd) = cd_entries.get(index) {
        let encoding = if cd.utf8_flag {
            Some(encoding_rs::UTF_8)
        } else {
            detect_archive_encoding(archive_path, &cd.name_bytes)
        };
        decode_name(&cd.name_bytes, encoding)
    } else {
        entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string())
    }
}

// ── ZIP Operations ─────────────────────────────────────────────────

pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let cd_entries = parse_zip_central_dir(path).unwrap_or_default();

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    if let Some(password) = password {
        validate_zip_password(&mut archive, path, password)?;
        remember_archive_password(path, Some(password));
    } else if zip_entry_requires_password(&mut archive)?.is_some() {
        return Err(password_required_error(path));
    }

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = HashSet::new();

    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|e| map_zip_error(path, e))?;
        let entry_path = if let Some(cd) = cd_entries.get(i) {
            let encoding = if cd.utf8_flag {
                Some(encoding_rs::UTF_8)
            } else {
                detect_archive_encoding(path, &cd.name_bytes)
            };
            decode_name(&cd.name_bytes, encoding)
        } else {
            entry
                .enclosed_name()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.name().to_string())
        };
        let norm = entry_path.replace('\\', "/");

        if entry.is_dir() {
            all_dirs.insert(norm.trim_end_matches('/').to_string());
        } else {
            all_paths.push(norm);
            // Collect parent directories
            let mut parent = Path::new(&entry_path).parent();
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

    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal)))
}

pub(crate) fn read_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String> {
    let cd_entries = parse_zip_central_dir(path).unwrap_or_default();

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let needs_password = zip_entry_requires_password(&mut archive)?;
    if needs_password.is_some() {
        if let Some(password) = password {
            validate_zip_password(&mut archive, path, password)?;
            remember_archive_password(path, Some(password));
        } else {
            return Err(password_required_error(path));
        }
    }
    let internal = normalize_internal(internal);

    for i in 0..archive.len() {
        let mut entry = read_zip_entry(&mut archive, path, i, password)?;
        let entry_path = decode_cd_entry_name(&cd_entries, i, &entry, path);
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();
        if norm == internal {
            if entry.is_dir() {
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

pub(crate) fn extract_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<&str>,
) -> Result<(), String> {
    let cd_entries = parse_zip_central_dir(path).unwrap_or_default();

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let needs_password = zip_entry_requires_password(&mut archive)?;
    if needs_password.is_some() {
        if let Some(password) = password {
            validate_zip_password(&mut archive, path, password)?;
            remember_archive_password(path, Some(password));
        } else {
            return Err(password_required_error(path));
        }
    }

    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    for i in 0..archive.len() {
        let mut entry = read_zip_entry(&mut archive, path, i, password)?;
        let entry_path = decode_cd_entry_name(&cd_entries, i, &entry, path);
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();

        if target_set.contains(&norm) {
            let dest = Path::new(dest_dir).join(&entry_path);
            if entry.is_dir() {
                fs::create_dir_all(&dest)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dir: {}", e))?;
                }
                let mut out = File::create(&dest)
                    .map_err(|e| format!("Failed to create file: {}", e))?;
                let mut buf = Vec::new();
                entry
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Failed to read entry: {}", e))?;
                out.write_all(&buf)
                    .map_err(|e| format!("Failed to write: {}", e))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn extract_all(path: &str, dest_dir: &str, password: Option<&str>, skip_paths: Option<&HashSet<String>>) -> Result<u64, String> {
    let cd_entries = parse_zip_central_dir(path).unwrap_or_default();

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let needs_password = zip_entry_requires_password(&mut archive)?;
    if needs_password.is_some() {
        if let Some(password) = password {
            validate_zip_password(&mut archive, path, password)?;
            remember_archive_password(path, Some(password));
        } else {
            return Err(password_required_error(path));
        }
    }

    let mut total_bytes: u64 = 0;
    for i in 0..archive.len() {
        let mut entry = read_zip_entry(&mut archive, path, i, password)?;
        let entry_path = decode_cd_entry_name(&cd_entries, i, &entry, path);

        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();
        if let Some(skip) = skip_paths {
            if skip.contains(&norm) && !entry.is_dir() {
                continue;
            }
        }

        let dest = Path::new(dest_dir).join(&entry_path);
        if entry.is_dir() {
            fs::create_dir_all(&dest)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            let mut out =
                File::create(&dest).map_err(|e| format!("Failed to create file: {}", e))?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read entry: {}", e))?;
            total_bytes += buf.len() as u64;
            out.write_all(&buf)
                .map_err(|e| format!("Failed to write: {}", e))?;
        }
    }
    Ok(total_bytes)
}

pub(crate) fn delete_entries(path: &str, internal_paths: &[String]) -> Result<(), String> {
    let target_set: HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let tmp_path = format!("{}.tmp", path);
    let tmp_file = File::create(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = zip::ZipWriter::new(tmp_file);

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string());
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();

        if target_set.contains(&norm) {
            continue;
        }

        if entry.is_dir() {
            writer
                .add_directory(
                    &entry_path,
                    zip::write::SimpleFileOptions::default(),
                )
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy file entry: {}", e))?;
        }
    }

    let _ = writer
        .finish()
        .map_err(|e| format!("Failed to finalize: {}", e))?;

    fs::remove_file(path).map_err(|e| format!("Failed to remove original: {}", e))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

pub(crate) fn rename_entry(path: &str, old_path: &str, new_path: &str) -> Result<(), String> {
    let old_norm = normalize_internal(old_path);
    let new_norm = normalize_internal(new_path);

    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let tmp_path = format!("{}.tmp", path);
    let tmp_file = File::create(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = zip::ZipWriter::new(tmp_file);
    let options = zip::write::SimpleFileOptions::default();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string());
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();

        let write_path = if norm == old_norm {
            &new_norm
        } else {
            &entry_path
        };

        if entry.is_dir() {
            writer
                .add_directory(write_path, options)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            writer
                .start_file(write_path, options)
                .map_err(|e| format!("Failed to start renamed entry: {}", e))?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read entry data: {}", e))?;
            writer
                .write_all(&buf)
                .map_err(|e| format!("Failed to write renamed entry: {}", e))?;
        }
    }

    let _ = writer
        .finish()
        .map_err(|e| format!("Failed to finalize: {}", e))?;

    fs::remove_file(path).map_err(|e| format!("Failed to remove original: {}", e))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

pub(crate) fn add_files(
    archive_path: &str,
    source_paths: &[String],
    internal_path: &str,
) -> Result<(), String> {
    let internal_prefix = normalize_internal(internal_path);
    let prefix = if internal_prefix.is_empty() {
        String::new()
    } else {
        format!("{}/", internal_prefix)
    };

    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let tmp_path = format!("{}.tmp", archive_path);
    let tmp_file = File::create(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = zip::ZipWriter::new(tmp_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Copy all existing entries
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry.is_dir() {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy file entry: {}", e))?;
        }
    }

    // Add new files
    for source in source_paths {
        let src_path = Path::new(source);
        if src_path.is_dir() {
            let dir_name = src_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| source.clone());
            let dir_prefix = format!("{}{}", prefix, dir_name);
            writer
                .add_directory(&dir_prefix, options)
                .map_err(|e| format!("Failed to add directory: {}", e))?;
            add_dir_to_zip(&mut writer, src_path, &dir_prefix, &options, &mut 0)?;
        } else {
            let name = src_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| source.clone());
            let entry_path = format!("{}{}", prefix, name);
            writer
                .start_file(&entry_path, options)
                .map_err(|e| format!("Failed to add file to archive: {}", e))?;
            let mut f = File::open(src_path)
                .map_err(|e| format!("Failed to open source file: {}", e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read source file: {}", e))?;
            writer
                .write_all(&buf)
                .map_err(|e| format!("Failed to write to archive: {}", e))?;
        }
    }

    let _ = writer
        .finish()
        .map_err(|e| format!("Failed to finalize: {}", e))?;

    fs::remove_file(archive_path)
        .map_err(|e| format!("Failed to remove original: {}", e))?;
    fs::rename(&tmp_path, archive_path)
        .map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

pub(crate) fn write_file(archive_path: &str, internal_path: &str, content: &[u8]) -> Result<(), String> {
    let target = normalize_internal(internal_path);

    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let tmp_path = format!("{}.tmp", archive_path);
    let tmp_file = File::create(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = zip::ZipWriter::new(tmp_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut found = false;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string());
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();

        if norm == target {
            found = true;
            writer
                .start_file(&entry_path, options)
                .map_err(|e| format!("Failed to start updated entry: {}", e))?;
            writer
                .write_all(content)
                .map_err(|e| format!("Failed to write updated entry: {}", e))?;
        } else if entry.is_dir() {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy file entry: {}", e))?;
        }
    }

    if !found {
        return Err(format!("Entry not found in archive: {}", internal_path));
    }

    let _ = writer
        .finish()
        .map_err(|e| format!("Failed to finalize: {}", e))?;

    fs::remove_file(archive_path)
        .map_err(|e| format!("Failed to remove original: {}", e))?;
    fs::rename(&tmp_path, archive_path)
        .map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

pub(crate) fn create_entry(archive_path: &str, internal_path: &str, is_dir: bool) -> Result<(), String> {
    let target = normalize_internal(internal_path);

    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    // Check if entry already exists
    let target_with_slash = format!("{}/", target);
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.name().to_string());
        let norm = entry_path.replace('\\', "/").trim_end_matches('/').to_string();
        if norm == target || norm == target_with_slash {
            return Err(format!("Entry already exists: {}", internal_path));
        }
    }

    let tmp_path = format!("{}.tmp", archive_path);
    let tmp_file =
        File::create(&tmp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = zip::ZipWriter::new(tmp_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Copy all existing entries
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry.is_dir() {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            writer
                .raw_copy_file(entry)
                .map_err(|e| format!("Failed to copy file entry: {}", e))?;
        }
    }

    // Add the new entry
    if is_dir {
        writer
            .add_directory(&target, options)
            .map_err(|e| format!("Failed to create directory entry: {}", e))?;
    } else {
        writer
            .start_file(&target, options)
            .map_err(|e| format!("Failed to create file entry: {}", e))?;
        writer
            .write_all(&[])
            .map_err(|e| format!("Failed to write empty file: {}", e))?;
    }

    let _ = writer
        .finish()
        .map_err(|e| format!("Failed to finalize: {}", e))?;

    fs::remove_file(archive_path)
        .map_err(|e| format!("Failed to remove original: {}", e))?;
    fs::rename(&tmp_path, archive_path)
        .map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

pub(crate) fn compress_files(sources: &[String], dest_path: &str) -> Result<u64, String> {
    let dest = Path::new(dest_path);
    let file = File::create(dest)
        .map_err(|e| format!("Failed to create archive: {}", e))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut total_bytes: u64 = 0;

    for source in sources {
        let src_path = Path::new(source);
        if src_path.is_dir() {
            let dir_name = src_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| source.clone());
            add_dir_to_zip(&mut writer, src_path, &dir_name, &options, &mut total_bytes)?;
        } else {
            let name = src_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| source.clone());
            writer
                .start_file(&name, options)
                .map_err(|e| format!("Failed to add file to archive: {}", e))?;
            let mut f = File::open(src_path)
                .map_err(|e| format!("Failed to open source file: {}", e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read source file: {}", e))?;
            total_bytes += buf.len() as u64;
            writer
                .write_all(&buf)
                .map_err(|e| format!("Failed to write to archive: {}", e))?;
        }
    }

    writer
        .finish()
        .map_err(|e| format!("Failed to finalize archive: {}", e))?;
    Ok(total_bytes)
}

fn add_dir_to_zip<W: Write + Seek>(
    writer: &mut zip::ZipWriter<W>,
    dir: &Path,
    prefix: &str,
    options: &zip::write::SimpleFileOptions,
    total_bytes: &mut u64,
) -> Result<(), String> {
    writer
        .add_directory(prefix, *options)
        .map_err(|e| format!("Failed to add directory: {}", e))?;

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let rel = format!("{}/{}", prefix, name);

        if path.is_dir() {
            add_dir_to_zip(writer, &path, &rel, options, total_bytes)?;
        } else {
            writer
                .start_file(&rel, *options)
                .map_err(|e| format!("Failed to add file: {}", e))?;
            let mut f =
                File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            *total_bytes += buf.len() as u64;
            writer
                .write_all(&buf)
                .map_err(|e| format!("Failed to write: {}", e))?;
        }
    }
    Ok(())
}
