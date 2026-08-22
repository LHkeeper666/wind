use notify::{EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use tauri::Emitter;

pub struct DirectoryWatcher {
    watcher: Option<notify::RecommendedWatcher>,
}

impl DirectoryWatcher {
    pub fn new() -> Self { Self { watcher: None } }

    pub fn start(&mut self, root: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
        self.stop();
        let root_path = Path::new(root);
        if !root_path.is_dir() { return Err(format!("Not a directory: {}", root)); }
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())
            .map_err(|error| error.to_string())?;
        watcher.watch(root_path, RecursiveMode::Recursive).map_err(|error| error.to_string())?;
        std::thread::spawn(move || {
            for event in rx {
                let Ok(event) = event else { continue; };
                if !matches!(event.kind, EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) | EventKind::Any) { continue; }
                let paths: Vec<String> = event.paths.into_iter()
                    .filter(|path| !path.components().any(|part| matches!(part.as_os_str().to_string_lossy().to_ascii_lowercase().as_str(), ".git" | "target" | "node_modules")))
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();
                if !paths.is_empty() { let _ = app_handle.emit("directory-changed", paths); }
            }
        });
        self.watcher = Some(watcher);
        Ok(())
    }

    pub fn stop(&mut self) { self.watcher = None; }
}
