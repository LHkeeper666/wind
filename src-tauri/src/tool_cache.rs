use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use crate::app_paths;

#[derive(Default, Deserialize, Serialize)]
struct DiskToolCache {
    tools: HashMap<String, String>,
}

struct ToolCacheState {
    loaded: bool,
    tools: HashMap<String, Option<PathBuf>>,
}

impl Default for ToolCacheState {
    fn default() -> Self {
        Self {
            loaded: false,
            tools: HashMap::new(),
        }
    }
}

static TOOL_CACHE: OnceLock<Mutex<ToolCacheState>> = OnceLock::new();

fn cache_state() -> &'static Mutex<ToolCacheState> {
    TOOL_CACHE.get_or_init(|| Mutex::new(ToolCacheState::default()))
}

fn cache_path() -> PathBuf {
    app_paths::cache_file("tool-paths.json")
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn load_disk_cache(state: &mut ToolCacheState) {
    if state.loaded {
        return;
    }
    state.loaded = true;

    let path = cache_path();
    let legacy_path = app_paths::legacy_local_file("tool-paths.json");
    let _ = app_paths::migrate_legacy_file(&path, &legacy_path);

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(cache) = serde_json::from_str::<DiskToolCache>(&contents) else {
        return;
    };

    for (name, path) in cache.tools {
        let path = PathBuf::from(path);
        if is_executable_file(&path) {
            state.tools.insert(name, Some(path));
        }
    }
}

fn save_disk_cache(state: &ToolCacheState) {
    let tools = state
        .tools
        .iter()
        .filter_map(|(name, path)| {
            path.as_ref()
                .filter(|path| is_executable_file(path))
                .map(|path| (name.clone(), path.to_string_lossy().to_string()))
        })
        .collect();
    let cache = DiskToolCache { tools };

    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&cache) {
        let _ = fs::write(path, json);
    }
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.is_absolute() {
        return is_executable_file(path).then(|| path.to_path_buf());
    }

    let path_var = std::env::var_os("PATH")?;
    let has_extension = path.extension().is_some();
    let extensions: Vec<String> = if has_extension {
        vec![String::new()]
    } else {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .filter(|ext| !ext.is_empty())
            .map(|ext| ext.to_string())
            .collect()
    };

    for directory in std::env::split_paths(&path_var) {
        if has_extension {
            let candidate = directory.join(command);
            if is_executable_file(&candidate) {
                return Some(candidate);
            }
            continue;
        }

        for extension in &extensions {
            let candidate = directory.join(format!("{command}{extension}"));
            if is_executable_file(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn resolve_tool(name: &str, env_var: &str, candidates: &[&str]) -> Option<PathBuf> {
    let mut state = cache_state().lock().unwrap();
    load_disk_cache(&mut state);

    // An explicit environment override must take precedence over a stale
    // path persisted from an earlier application run.
    if let Some(path) = std::env::var_os(env_var)
        .map(PathBuf::from)
        .filter(|path| is_executable_file(path))
    {
        state.tools.insert(name.to_string(), Some(path.clone()));
        save_disk_cache(&state);
        return Some(path);
    }

    if let Some(path) = state.tools.get(name) {
        if let Some(path) = path.as_ref().filter(|path| is_executable_file(path)) {
            return Some(path.clone());
        }
    }

    let discovered = candidates
        .iter()
        .find_map(|candidate| find_on_path(candidate));

    state.tools.insert(name.to_string(), discovered.clone());
    save_disk_cache(&state);
    discovered
}

pub fn invalidate(name: &str) {
    let mut state = cache_state().lock().unwrap();
    load_disk_cache(&mut state);
    state.tools.remove(name);
    save_disk_cache(&state);
}

pub fn ffmpeg_path() -> Option<PathBuf> {
    resolve_tool("ffmpeg", "WIND_FFMPEG_PATH", &["ffmpeg"])
}

pub fn fd_path() -> Option<PathBuf> {
    resolve_tool(
        "fd",
        "WIND_FD_PATH",
        &[
            "fd",
            r"D:\Application\fd\fd-v10.2.0-x86_64-pc-windows-msvc\fd.exe",
        ],
    )
}

pub fn rg_path() -> Option<PathBuf> {
    resolve_tool("rg", "WIND_RG_PATH", &["rg"])
}

pub fn bash_path() -> Option<PathBuf> {
    resolve_tool(
        "bash",
        "WIND_BASH_PATH",
        &[
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\msys64\usr\bin\bash.exe",
            r"C:\cygwin64\bin\bash.exe",
            "bash",
        ],
    )
}

pub fn python_path() -> Option<PathBuf> {
    resolve_tool("python", "WIND_PYTHON_PATH", &["python", "python3", "py"])
}

pub fn nvim_path() -> Option<PathBuf> {
    resolve_tool("nvim", "WIND_NVIM_PATH", &["nvim"])
}

/// Creates a console program without a visible Windows console window.
pub fn background_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}
