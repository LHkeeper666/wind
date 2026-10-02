mod encoding;
mod password;
mod seven_z;
mod shared;
mod tar;
mod zip;

use crate::FileEntry;

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
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::list_entries(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => tar::list_entries(archive_path, internal_path),
        ArchiveFormat::TarGz => tar::list_gz_entries(archive_path, internal_path),
        ArchiveFormat::SevenZ => seven_z::list_entries(archive_path, internal_path, password.as_deref()),
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
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::extract_files(archive_path, internal_paths, dest_dir, password.as_deref()),
        ArchiveFormat::Tar => tar::extract_files(archive_path, internal_paths, dest_dir),
        ArchiveFormat::TarGz => tar::extract_gz_files(archive_path, internal_paths, dest_dir),
        ArchiveFormat::SevenZ => seven_z::extract_files(archive_path, internal_paths, dest_dir, password.as_deref()),
    }
}

pub fn extract_all(
    archive_path: &str,
    dest_dir: &str,
    password: Option<String>,
    skip_paths: Option<&std::collections::HashSet<String>>,
) -> Result<u64, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::extract_all(archive_path, dest_dir, password.as_deref(), skip_paths),
        ArchiveFormat::Tar => tar::extract_all(archive_path, dest_dir, skip_paths),
        ArchiveFormat::TarGz => tar::extract_gz_all(archive_path, dest_dir, skip_paths),
        ArchiveFormat::SevenZ => seven_z::extract_all(archive_path, dest_dir, password.as_deref(), skip_paths),
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
