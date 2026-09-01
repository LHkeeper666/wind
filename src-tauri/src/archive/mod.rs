use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::FileEntry;

static ENCODING_CACHE: Mutex<Option<HashMap<String, Option<&'static encoding_rs::Encoding>>>> =
    Mutex::new(None);
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

fn resolve_archive_password(archive_path: &str, password: Option<String>) -> Option<String> {
    password.or_else(|| cached_archive_password(archive_path))
}

fn remember_archive_password(archive_path: &str, password: Option<&str>) {
    if let Some(password) = password {
        store_archive_password(archive_path, password);
    }
}

fn password_required_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_REQUIRED: {}", archive_path)
}

fn password_incorrect_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_INCORRECT: {}", archive_path)
}

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

fn map_7z_error(path: &str, err: sevenz_rust::Error) -> String {
    match err {
        sevenz_rust::Error::PasswordRequired => password_required_error(path),
        sevenz_rust::Error::MaybeBadPassword(_) => password_incorrect_error(path),
        other => format!("Failed to read 7z: {}", other),
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    SevenZ,
}

impl ArchiveFormat {
    pub fn from_path(path: &str) -> Option<Self> {
        let lower = path.to_lowercase();
        if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            Some(ArchiveFormat::TarGz)
        } else if lower.ends_with(".tar") {
            Some(ArchiveFormat::Tar)
        } else if lower.ends_with(".zip") {
            Some(ArchiveFormat::Zip)
        } else if lower.ends_with(".7z") {
            Some(ArchiveFormat::SevenZ)
        } else {
            None
        }
    }

    pub fn supports_write(&self) -> bool {
        matches!(self, ArchiveFormat::Zip)
    }
}

pub fn list_entries(
    archive_path: &str,
    internal_path: &str,
    password: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => list_zip_entries(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => list_tar_entries(archive_path, internal_path),
        ArchiveFormat::TarGz => list_tar_gz_entries(archive_path, internal_path),
        ArchiveFormat::SevenZ => list_7z_entries(archive_path, internal_path, password.as_deref()),
    }
}

pub fn read_file_bytes(
    archive_path: &str,
    internal_path: &str,
    password: Option<String>,
) -> Result<Vec<u8>, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => read_zip_file(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => read_tar_file(archive_path, internal_path),
        ArchiveFormat::TarGz => read_tar_gz_file(archive_path, internal_path),
        ArchiveFormat::SevenZ => read_7z_file(archive_path, internal_path, password.as_deref()),
    }
}

pub fn extract_files(
    archive_path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<String>,
) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => extract_zip_files(archive_path, internal_paths, dest_dir, password.as_deref()),
        ArchiveFormat::Tar => extract_tar_files(archive_path, internal_paths, dest_dir),
        ArchiveFormat::TarGz => extract_tar_gz_files(archive_path, internal_paths, dest_dir),
        ArchiveFormat::SevenZ => extract_7z_files(archive_path, internal_paths, dest_dir, password.as_deref()),
    }
}

pub fn extract_all(
    archive_path: &str,
    dest_dir: &str,
    password: Option<String>,
) -> Result<u64, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => extract_zip_all(archive_path, dest_dir, password.as_deref()),
        ArchiveFormat::Tar => extract_tar_all(archive_path, dest_dir),
        ArchiveFormat::TarGz => extract_tar_gz_all(archive_path, dest_dir),
        ArchiveFormat::SevenZ => extract_7z_all(archive_path, dest_dir, password.as_deref()),
    }
}

pub fn delete_entries(archive_path: &str, internal_paths: &[String]) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    if !format.supports_write() {
        return Err(format!(
            "Delete is only supported for ZIP archives (not {:?})",
            format
        ));
    }
    delete_zip_entries(archive_path, internal_paths)
}

pub fn rename_entry(archive_path: &str, old_path: &str, new_path: &str) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    if !format.supports_write() {
        return Err(format!(
            "Rename is only supported for ZIP archives (not {:?})",
            format
        ));
    }
    rename_zip_entry(archive_path, old_path, new_path)
}

pub fn add_files(
    archive_path: &str,
    source_paths: &[String],
    internal_path: &str,
) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    if !format.supports_write() {
        return Err(format!(
            "Adding files is only supported for ZIP archives (not {:?})",
            format
        ));
    }
    add_files_to_zip(archive_path, source_paths, internal_path)
}

pub fn write_file_bytes(
    archive_path: &str,
    internal_path: &str,
    content: &[u8],
) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    if !format.supports_write() {
        return Err(format!(
            "Editing files is only supported for ZIP archives (not {:?})",
            format
        ));
    }
    write_zip_file(archive_path, internal_path, content)
}

pub fn create_entry(
    archive_path: &str,
    internal_path: &str,
    is_dir: bool,
) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    if !format.supports_write() {
        return Err(format!(
            "Creating entries is only supported for ZIP archives (not {:?})",
            format
        ));
    }
    create_zip_entry(archive_path, internal_path, is_dir)
}

pub fn compress_files(sources: &[String], dest_path: &str) -> Result<u64, String> {
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

fn normalize_internal(path: &str) -> String {
    let trimmed = path.trim_matches('/').trim_matches('\\');
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.replace('\\', "/")
    }
}

fn matches_internal_path(entry_path: &str, internal: &str) -> bool {
    let norm_entry = entry_path.replace('\\', "/").trim_end_matches('/').to_string();
    if internal.is_empty() {
        // At root: only show entries directly in root (no slash in relative path after stripping)
        !norm_entry.contains('/')
    } else {
        let norm_internal = internal.trim_end_matches('/');
        // Entry must be directly under internal_path
        if !norm_entry.starts_with(&format!("{}/", norm_internal)) {
            return false;
        }
        let relative = &norm_entry[norm_internal.len() + 1..];
        // relative should not contain '/' (direct child only)
        !relative.contains('/')
    }
}

fn is_dir_in_entries(all_paths: &[String], path: &str, internal: &str) -> bool {
    let norm = path.replace('\\', "/").trim_end_matches('/').to_string();
    let prefix = if internal.is_empty() {
        format!("{}/", norm)
    } else {
        let norm_internal = internal.trim_end_matches('/');
        if !norm.starts_with(&format!("{}/", norm_internal)) {
            return false;
        }
        norm.clone()
    };
    all_paths
        .iter()
        .any(|p| p.replace('\\', "/").starts_with(&format!("{}/", prefix)))
}

fn collect_entries_at_path(
    all_paths: &[String],
    all_dirs: &std::collections::HashSet<String>,
    internal: &str,
) -> Vec<FileEntry> {
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for path in all_paths {
        let norm = path.replace('\\', "/");
        if !matches_internal_path(&norm, internal) {
            continue;
        }
        let name = Path::new(&norm)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| norm.clone());
        let full_internal = norm.clone();

        if seen.contains(&full_internal) {
            continue;
        }
        seen.insert(full_internal.clone());

        let is_dir = all_dirs.contains(&full_internal)
            || is_dir_in_entries(all_paths, &full_internal, internal);
        entries.push(FileEntry {
            name,
            path: full_internal,
            is_dir,
            size: None,
            is_hidden: false,
            modified: None,
            created: None,
            children: None,
        });
    }

    // Also add directories that appear only as parents
    for dir in all_dirs {
        let norm = dir.replace('\\', "/");
        if !matches_internal_path(&norm, internal) {
            continue;
        }
        let name = Path::new(&norm)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| norm.clone());
        let full_internal = norm.clone();

        if seen.contains(&full_internal) {
            continue;
        }
        seen.insert(full_internal.clone());

        entries.push(FileEntry {
            name,
            path: full_internal,
            is_dir: true,
            size: None,
            is_hidden: false,
            modified: None,
            created: None,
            children: None,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
        }
    });

    entries
}

// ── Encoding Detection ──────────────────────────────────────────────

/// Parsed ZIP central directory entry with raw filename bytes
struct ZipCdEntry {
    name_bytes: Vec<u8>,
    utf8_flag: bool,
}

/// Try to detect the encoding for a ZIP archive's entry names.
/// Returns the detected encoding, or None if detection failed.
fn detect_archive_encoding(archive_path: &str, name_bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    // Check cache first
    {
        let cache = ENCODING_CACHE.lock().unwrap();
        if let Some(ref map) = *cache {
            if let Some(cached) = map.get(archive_path) {
                return *cached;
            }
        }
    }

    let encoding = detect_encoding_from_bytes(name_bytes);
    let mut cache = ENCODING_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
    cache.as_mut().unwrap().insert(archive_path.to_string(), encoding);
    encoding
}

fn detect_encoding_from_bytes(bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    if String::from_utf8(bytes.to_vec()).is_ok() {
        return Some(encoding_rs::UTF_8);
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    if encoding == encoding_rs::UTF_8 {
        // chardetng defaults to UTF-8 for short/ambiguous input; trust it
        Some(encoding_rs::UTF_8)
    } else {
        Some(encoding)
    }
}

fn decode_name(raw: &[u8], encoding: Option<&'static encoding_rs::Encoding>) -> String {
    match encoding {
        Some(enc) => enc.decode_without_bom_handling(raw).0.into_owned(),
        None => String::from_utf8_lossy(raw).to_string(),
    }
}

/// Parse ZIP central directory to extract raw filename bytes and UTF-8 flags.
/// Returns a Vec of (normalized_path, raw_name_bytes, utf8_flag).
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

// ── ZIP ────────────────────────────────────────────────────────────

fn list_zip_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    // Try to parse CD for raw name bytes and encoding detection
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
    let mut all_dirs = std::collections::HashSet::new();

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

fn read_zip_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String> {
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

fn extract_zip_files(
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

    let target_set: std::collections::HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    for i in 0..archive.len() {
        let mut entry = read_zip_entry(&mut archive, path, i, password)?;
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

fn extract_zip_all(path: &str, dest_dir: &str, password: Option<&str>) -> Result<u64, String> {
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

fn delete_zip_entries(path: &str, internal_paths: &[String]) -> Result<(), String> {
    let target_set: std::collections::HashSet<String> = internal_paths
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

fn rename_zip_entry(path: &str, old_path: &str, new_path: &str) -> Result<(), String> {
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

fn add_files_to_zip(
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

fn write_zip_file(archive_path: &str, internal_path: &str, content: &[u8]) -> Result<(), String> {
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

fn create_zip_entry(archive_path: &str, internal_path: &str, is_dir: bool) -> Result<(), String> {
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

// ── TAR ────────────────────────────────────────────────────────────

fn decode_tar_name(path: &str, entry: &tar::Entry<impl Read>) -> String {
    let raw = entry.path_bytes();
    let encoding = detect_archive_encoding(path, &raw);
    decode_name(&raw, encoding)
}

fn list_tar_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = std::collections::HashSet::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
        let norm = path_str.replace('\\', "/");

        if entry.header().entry_type().is_dir() {
            all_dirs.insert(norm.trim_end_matches('/').to_string());
        } else {
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

    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal)))
}

fn read_tar_file(path: &str, internal: &str) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
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

fn extract_tar_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    let target_set: std::collections::HashSet<String> = internal_paths
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

fn extract_tar_all(path: &str, dest_dir: &str) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar: {}", e))?;

    let mut total_bytes: u64 = 0;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
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

// ── TAR.GZ ─────────────────────────────────────────────────────────

fn list_tar_gz_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar.gz: {}", e))?;

    let mut all_paths: Vec<String> = Vec::new();
    let mut all_dirs = std::collections::HashSet::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
        let norm = path_str.replace('\\', "/");

        if entry.header().entry_type().is_dir() {
            all_dirs.insert(norm.trim_end_matches('/').to_string());
        } else {
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

    Ok(collect_entries_at_path(&all_paths, &all_dirs, &normalize_internal(internal)))
}

fn read_tar_gz_file(path: &str, internal: &str) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar.gz: {}", e))?;
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

fn extract_tar_gz_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let target_set: std::collections::HashSet<String> = internal_paths
        .iter()
        .map(|p| normalize_internal(p))
        .collect();

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar.gz: {}", e))?;
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

fn extract_tar_gz_all(path: &str, dest_dir: &str) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar.gz: {}", e))?;

    let mut total_bytes: u64 = 0;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_str = decode_tar_name(path, &entry);
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

// ── 7Z ─────────────────────────────────────────────────────────────

fn make_7z_password(password: Option<&str>) -> sevenz_rust::Password {
    match password {
        Some(password) => sevenz_rust::Password::from(password),
        None => sevenz_rust::Password::empty(),
    }
}

fn list_7z_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
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
    let mut all_dirs = std::collections::HashSet::new();
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

fn read_7z_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String> {
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

fn extract_7z_files(
    path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<&str>,
) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, file_len, make_7z_password(password))
        .map_err(|e| map_7z_error(path, e))?;
    let target_set: std::collections::HashSet<String> = internal_paths
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

fn extract_7z_all(path: &str, dest_dir: &str, password: Option<&str>) -> Result<u64, String> {
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
