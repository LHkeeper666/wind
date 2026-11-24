use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use log::debug;

static ARCHIVE_PASSWORD_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RarSuffix {
    Ordinary,
    Legacy,
    Numbered { width: usize },
}

pub(super) fn rar_suffix(path: &Path) -> Option<RarSuffix> {
    let name = path.file_name()?.to_str()?;
    let (stem, extension) = name.rsplit_once('.')?;
    if extension.eq_ignore_ascii_case("rar") {
        if let Some((prefix, segment)) = stem.rsplit_once('.') {
            if let Some(marker) = segment.get(..4).filter(|s| s.eq_ignore_ascii_case("part")) {
                let digits = &segment[marker.len()..];
                if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) && !prefix.is_empty() {
                    return Some(RarSuffix::Numbered { width: digits.len() });
                }
            }
        }
        return Some(RarSuffix::Ordinary);
    }
    if extension.len() == 3
        && extension.as_bytes()[0].eq_ignore_ascii_case(&b'r')
        && extension.as_bytes()[1..].iter().all(u8::is_ascii_digit)
        && !stem.is_empty()
    {
        return Some(RarSuffix::Legacy);
    }
    None
}

fn rar_first_part(path: &Path, suffix: RarSuffix) -> std::path::PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    match suffix {
        RarSuffix::Ordinary => path.to_path_buf(),
        RarSuffix::Legacy => {
            let (stem, _) = name.rsplit_once('.').unwrap();
            path.with_file_name(format!("{stem}.rar"))
        }
        RarSuffix::Numbered { width } => {
            let (stem, _) = name.rsplit_once('.').unwrap();
            let (prefix, _) = stem.rsplit_once('.').unwrap();
            path.with_file_name(format!("{prefix}.part{number:0width$}.rar", number = 1))
        }
    }
}

fn archive_cache_key(archive_path: &str) -> String {
    let path = Path::new(archive_path);
    // For multipart RAR, normalize to first volume so all volumes share the password cache
    if let Some(suffix) = rar_suffix(path) {
        let first = rar_first_part(path, suffix);
        let canonical = fs::canonicalize(&first).unwrap_or(first);
        return format!("rar:{}", canonical.to_string_lossy());
    }
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
    let result = map.get(&key).cloned();
    if result.is_some() {
        debug!("[archive] password cache hit for {}", archive_path);
    }
    result
}

fn store_archive_password(archive_path: &str, password: &str) {
    let key = archive_cache_key(archive_path);
    let mut cache = ARCHIVE_PASSWORD_CACHE.lock().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);
    map.insert(key, password.to_string());
    debug!("[archive] password cached for {}", archive_path);
}

pub(crate) fn resolve_archive_password(archive_path: &str, password: Option<String>) -> Option<String> {
    password.or_else(|| cached_archive_password(archive_path))
}

pub(crate) fn remember_archive_password(archive_path: &str, password: Option<&str>) {
    if let Some(password) = password {
        store_archive_password(archive_path, password);
    }
}

pub(crate) fn password_required_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_REQUIRED: {}", archive_path)
}

pub(crate) fn password_incorrect_error(archive_path: &str) -> String {
    format!("ARCHIVE_PASSWORD_INCORRECT: {}", archive_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::ArchiveFormat;

    #[test]
    fn unicode_paths_reach_archive_reader_without_panicking() {
        for extension in ["7z", "zip", "tar", "tar.gz", "rar"] {
            let path = format!("Z:\\不存在目录\\非密计算机检查工具.{extension}");
            assert!(ArchiveFormat::from_path(&path).is_some());
            assert!(resolve_archive_password(&path, None).is_none());
            assert!(crate::archive::list_entries(&path, "", None).is_err());
        }
        for path in [
            "Z:\\不存在目录\\😀工具.7z",
            "Z:\\不存在目录\\工具具.7z",
            "Z:\\不存在目录\\İ工具.7z",
        ] {
            assert!(resolve_archive_password(path, None).is_none());
            assert!(crate::archive::list_entries(path, "", None).is_err());
        }
    }

    #[test]
    fn rar_suffix_classification_matches_format_detection() {
        for (name, expected) in [
            ("文件.rar", Some(RarSuffix::Ordinary)),
            ("文件.RAR", Some(RarSuffix::Ordinary)),
            ("文件.r00", Some(RarSuffix::Legacy)),
            ("文件.R99", Some(RarSuffix::Legacy)),
            ("文件.part2.rar", Some(RarSuffix::Numbered { width: 1 })),
            ("文件.PART12.RAR", Some(RarSuffix::Numbered { width: 2 })),
            ("文件.part02.rar", Some(RarSuffix::Numbered { width: 2 })),
            ("文件.partX.rar", Some(RarSuffix::Ordinary)),
            ("文件.part2.zip", None),
            ("文件.r0", None),
            ("文件.r001", None),
        ] {
            assert_eq!(rar_suffix(Path::new(name)), expected, "{name}");
            assert_eq!(ArchiveFormat::from_path(name) == Some(ArchiveFormat::Rar), expected.is_some(), "{name}");
        }
    }

    #[test]
    fn first_volume_preserves_unicode_prefix_and_padding() {
        for (name, first) in [
            ("非密😀.part2.rar", "非密😀.part1.rar"),
            ("非密😀.PART02.RAR", "非密😀.part01.rar"),
            ("非密😀.part002.rar", "非密😀.part001.rar"),
            ("非密😀.r00", "非密😀.rar"),
            ("非密😀.R99", "非密😀.rar"),
            ("非密😀.partX.rar", "非密😀.partX.rar"),
        ] {
            let path = Path::new(name);
            assert_eq!(rar_first_part(path, rar_suffix(path).unwrap()), Path::new(first));
        }
    }

    #[test]
    fn missing_first_volume_still_shares_cache_key() {
        let first = "Z:\\不存在目录\\非密😀.part01.rar";
        let second = "Z:\\不存在目录\\非密😀.part02.rar";
        assert_eq!(archive_cache_key(first), archive_cache_key(second));
        remember_archive_password(second, Some("test-password"));
        assert_eq!(resolve_archive_password(first, None).as_deref(), Some("test-password"));
        assert_ne!(archive_cache_key(first), archive_cache_key("Z:\\不存在目录\\其他😀.part02.rar"));

        let legacy_first = "Z:\\不存在目录\\非密😀.rar";
        let legacy_second = "Z:\\不存在目录\\非密😀.r00";
        assert_eq!(archive_cache_key(legacy_first), archive_cache_key(legacy_second));
        assert_ne!(archive_cache_key(first), archive_cache_key(legacy_first));
        assert_ne!(archive_cache_key(first), archive_cache_key("Z:\\不存在目录\\非密😀.7z"));
    }

    #[test]
    #[ignore = "requires WIND_ARCHIVE_SMOKE_PATH pointing to a local archive"]
    fn smoke_read_archive_from_env() {
        let path = std::env::var("WIND_ARCHIVE_SMOKE_PATH").expect("set WIND_ARCHIVE_SMOKE_PATH");
        match crate::archive::list_entries(&path, "", None) {
            Ok(entries) => println!("archive entries: {}", entries.len()),
            Err(error) => println!("archive reader error: {error}"),
        }
    }
}
