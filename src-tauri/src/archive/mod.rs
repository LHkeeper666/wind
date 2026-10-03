mod encoding;
mod password;
mod rar;
mod seven_z;
mod shared;
mod tar;
mod zip;

use std::sync::Arc;

use crate::FileEntry;

/// Callback for extraction progress. Receives bytes extracted by the current file.
/// Returns `true` to continue, `false` to cancel.
pub type ExtractProgress = Arc<dyn Fn(u64) -> bool + Send + Sync>;

/// Returns a no-op progress callback that always continues.
pub fn no_progress() -> ExtractProgress {
    Arc::new(|_| true)
}

fn is_rar_extension(lower: &str) -> bool {
    if lower.ends_with(".rar") {
        return true;
    }
    // Match .rXX pattern (e.g., .r00, .r01, .r99)
    if let Some(rest) = lower.strip_suffix(|c: char| c.is_ascii_digit()) {
        if let Some(rest2) = rest.strip_suffix(|c: char| c.is_ascii_digit()) {
            return rest2.ends_with(".r");
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    SevenZ,
    Rar,
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
        } else if is_rar_extension(&lower) {
            Some(ArchiveFormat::Rar)
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
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::list_entries(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => tar::list_entries(archive_path, internal_path),
        ArchiveFormat::TarGz => tar::list_gz_entries(archive_path, internal_path),
        ArchiveFormat::SevenZ => seven_z::list_entries(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Rar => rar::list_entries(archive_path, internal_path, password.as_deref()),
    }
}

pub fn read_file_bytes(
    archive_path: &str,
    internal_path: &str,
    password: Option<String>,
) -> Result<Vec<u8>, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::read_file(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => tar::read_file(archive_path, internal_path),
        ArchiveFormat::TarGz => tar::read_gz_file(archive_path, internal_path),
        ArchiveFormat::SevenZ => seven_z::read_file(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Rar => rar::read_file(archive_path, internal_path, password.as_deref()),
    }
}

/// Compute total uncompressed size of all files in the archive (no path filtering).
pub fn total_uncompressed_size(
    archive_path: &str,
    password: Option<String>,
) -> Result<u64, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::total_uncompressed_size(archive_path, password.as_deref()),
        ArchiveFormat::Tar => tar::total_uncompressed_size(archive_path),
        ArchiveFormat::TarGz => tar::total_gz_uncompressed_size(archive_path),
        ArchiveFormat::SevenZ => seven_z::total_uncompressed_size(archive_path, password.as_deref()),
        ArchiveFormat::Rar => rar::total_uncompressed_size(archive_path, password.as_deref()),
    }
}

/// Compute total uncompressed size of specific files in the archive.
pub fn files_uncompressed_size(
    archive_path: &str,
    internal_paths: &[String],
    password: Option<String>,
) -> Result<u64, String> {
    // Use list_entries to get sizes, then filter by internal_paths
    let entries = list_entries(archive_path, "", password)?;
    let target: std::collections::HashSet<String> = internal_paths.iter()
        .map(|p| p.replace('\\', "/").trim_end_matches('/').to_string())
        .collect();
    Ok(entries.iter()
        .filter(|e| !e.is_dir && target.contains(&e.path.replace('\\', "/").trim_end_matches('/').to_string()))
        .map(|e| e.size.unwrap_or(0))
        .sum())
}

pub fn extract_files(
    archive_path: &str,
    internal_paths: &[String],
    dest_dir: &str,
    password: Option<String>,
    on_progress: &ExtractProgress,
) -> Result<(), String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::extract_files(archive_path, internal_paths, dest_dir, password.as_deref(), on_progress),
        ArchiveFormat::Tar => tar::extract_files(archive_path, internal_paths, dest_dir, on_progress),
        ArchiveFormat::TarGz => tar::extract_gz_files(archive_path, internal_paths, dest_dir, on_progress),
        ArchiveFormat::SevenZ => seven_z::extract_files(archive_path, internal_paths, dest_dir, password.as_deref(), on_progress),
        ArchiveFormat::Rar => rar::extract_files(archive_path, internal_paths, dest_dir, password.as_deref(), on_progress),
    }
}

pub fn extract_all(
    archive_path: &str,
    dest_dir: &str,
    password: Option<String>,
    skip_paths: Option<&std::collections::HashSet<String>>,
    on_progress: &ExtractProgress,
) -> Result<u64, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::extract_all(archive_path, dest_dir, password.as_deref(), skip_paths, on_progress),
        ArchiveFormat::Tar => tar::extract_all(archive_path, dest_dir, skip_paths, on_progress),
        ArchiveFormat::TarGz => tar::extract_gz_all(archive_path, dest_dir, skip_paths, on_progress),
        ArchiveFormat::SevenZ => seven_z::extract_all(archive_path, dest_dir, password.as_deref(), skip_paths, on_progress),
        ArchiveFormat::Rar => rar::extract_all(archive_path, dest_dir, password.as_deref(), skip_paths, on_progress),
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
    zip::delete_entries(archive_path, internal_paths)
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
    zip::rename_entry(archive_path, old_path, new_path)
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
    zip::add_files(archive_path, source_paths, internal_path)
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
    zip::write_file(archive_path, internal_path, content)
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
    zip::create_entry(archive_path, internal_path, is_dir)
}

pub fn compress_files(sources: &[String], dest_path: &str) -> Result<u64, String> {
    zip::compress_files(sources, dest_path)
}
