use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const APP_DIRECTORY: &str = "wind";

fn roaming_root() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIRECTORY)
}

fn local_root() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIRECTORY)
}

pub fn config_dir() -> PathBuf {
    roaming_root().join("config")
}

pub fn cache_dir() -> PathBuf {
    local_root().join("cache")
}

pub fn state_dir() -> PathBuf {
    local_root().join("state")
}

pub fn config_file(name: &str) -> PathBuf {
    config_dir().join(name)
}

pub fn cache_file(name: &str) -> PathBuf {
    cache_dir().join(name)
}

pub fn state_file(name: &str) -> PathBuf {
    state_dir().join(name)
}

pub fn legacy_roaming_file(name: &str) -> PathBuf {
    roaming_root().join(name)
}

pub fn legacy_local_file(name: &str) -> PathBuf {
    local_root().join(name)
}

pub fn migrate_legacy_file(destination: &Path, legacy_source: &Path) -> io::Result<bool> {
    if destination.exists() || !legacy_source.is_file() {
        return Ok(false);
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(legacy_source, destination)?;
    Ok(true)
}
