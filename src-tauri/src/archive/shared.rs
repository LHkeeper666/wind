use std::collections::HashSet;
use std::path::Path;

use crate::FileEntry;

pub(crate) fn normalize_internal(path: &str) -> String {
    let trimmed = path.trim_matches('/').trim_matches('\\');
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.replace('\\', "/")
    }
}

pub(crate) fn matches_internal_path(entry_path: &str, internal: &str) -> bool {
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

pub(crate) fn is_dir_in_entries(all_paths: &[String], path: &str, internal: &str) -> bool {
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

pub(crate) fn collect_entries_at_path(
    all_paths: &[String],
    all_dirs: &HashSet<String>,
    internal: &str,
) -> Vec<FileEntry> {
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut seen = HashSet::new();

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
