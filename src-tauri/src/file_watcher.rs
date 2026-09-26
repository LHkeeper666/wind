use log::{info, error, debug};

use notify::{EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tauri::Emitter;

pub struct FileWatcher {
    _watcher: Option<notify::RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self { _watcher: None }
    }

    pub fn start(&mut self, path: &str, app_handle: tauri::AppHandle) {
        self.stop();

        let file_path = path.to_string();

        // Skip directories — we only watch files
        if Path::new(&file_path).is_dir() {
            info!("[file_watcher] Skipping directory: {}", file_path);
            return;
        }

        // Normalize path separators for reliable comparison
        let normalized = file_path.replace('/', "\\");

        let (tx, rx) = mpsc::channel();

        // Use polling on Windows for reliability (ReadDirectoryChangesW is flaky with files)
        let config = notify::Config::default()
            .with_poll_interval(Duration::from_secs(1));

        let mut watcher = match notify::RecommendedWatcher::new(tx, config) {
            Ok(w) => w,
            Err(e) => {
                error!("[file_watcher] Failed to create watcher: {}", e);
                return;
            }
        };

        let parent = Path::new(&normalized)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new(".").to_path_buf());

        info!("[file_watcher] Watching {} (parent: {})", normalized, parent.display());

        if let Err(e) = watcher.watch(&parent, RecursiveMode::NonRecursive) {
            error!("[file_watcher] Failed to watch {}: {}", parent.display(), e);
            return;
        }

        std::thread::spawn(move || {
            for event in rx {
                let evt = match event {
                    Ok(e) => e,
                    Err(e) => {
                        error!("[file_watcher] Watch error: {}", e);
                        continue;
                    }
                };

                debug!("[file_watcher] Raw event: kind={:?} paths={:?}", evt.kind, evt.paths);

                let is_modify = matches!(
                    &evt.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Any
                );
                if !is_modify {
                    continue;
                }

                let matched = evt.paths.iter().any(|p| {
                    let p_str = p.to_string_lossy().replace('/', "\\");
                    p_str == normalized
                        || p_str.to_lowercase() == normalized.to_lowercase()
                });
                if matched {
                    info!("[file_watcher] Emitting file-changed for {}", normalized);
                    if let Err(e) = app_handle.emit("file-changed", normalized.clone()) {
                        error!("[file_watcher] Emit error: {}", e);
                    }
                }
            }
            debug!("[file_watcher] Thread exiting");
        });

        self._watcher = Some(watcher);
    }

    pub fn stop(&mut self) {
        if self._watcher.is_some() {
            info!("[file_watcher] Stopping watcher");
        }
        self._watcher = None;
    }
}
