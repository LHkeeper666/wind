use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use crate::{app_paths, tool_cache};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiMember {
    pub name: String,
    pub signature: Option<String>,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageApi {
    pub name: String,
    pub version: String,
    pub members: Vec<ApiMember>,
}

fn cache_dir() -> PathBuf {
    app_paths::cache_dir().join("python-completions")
}

fn python_exe_hash(python: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    python.hash(&mut h);
    format!("{:x}", h.finish())
}

fn find_python(python_exe: Option<&str>) -> Result<String, String> {
    if let Some(exe) = python_exe {
        if PathBuf::from(exe).is_file() {
            return Ok(exe.to_string());
        }
    }
    tool_cache::python_path()
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "Python not found in PATH".into())
}

#[tauri::command]
pub async fn scan_python_packages(python_exe: Option<String>) -> Result<Vec<PackageInfo>, String> {
    let python = find_python(python_exe.as_deref())?;

    let output = tool_cache::background_command(&python)
        .arg("-m")
        .arg("pip")
        .arg("list")
        .arg("--format=json")
        .output()
        .map_err(|e| {
            tool_cache::invalidate("python");
            format!("Failed to run pip: {}", e)
        })?;

    if !output.status.success() {
        return Err(format!(
            "pip list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut packages: Vec<PackageInfo> =
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse pip output: {}", e))?;

    // Filter out pip itself and distro packages
    packages.retain(|p| p.name != "pip" && p.name != "setuptools" && p.name != "wheel");
    Ok(packages)
}

#[tauri::command]
pub async fn get_package_api(
    python_exe: Option<String>,
    package_name: String,
    force_refresh: Option<bool>,
) -> Result<PackageApi, String> {
    let python = find_python(python_exe.as_deref())?;
    let version = get_package_version(&python, &package_name).unwrap_or_default();
    let exe_hash = python_exe_hash(&python);
    let cache_key = format!("{}_{}_{}.json", exe_hash, package_name, version);
    let cache_path = cache_dir().join(&cache_key);

    // Read from cache if not forced to refresh
    if !force_refresh.unwrap_or(false) {
        if let Ok(mut file) = fs::File::open(&cache_path) {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_ok() {
                if let Ok(api) = serde_json::from_str::<PackageApi>(&buf) {
                    return Ok(api);
                }
            }
        }
    }

    // Extract API via Python subprocess
    let api = extract_package_api(&python, &package_name, &version)?;

    // Save to cache
    let _ = fs::create_dir_all(cache_dir());
    if let Ok(json) = serde_json::to_string_pretty(&api) {
        let _ = fs::write(&cache_path, json);
    }

    Ok(api)
}

fn get_package_version(python: &str, package_name: &str) -> Result<String, String> {
    let output = tool_cache::background_command(python)
        .arg("-m")
        .arg("pip")
        .arg("show")
        .arg(package_name)
        .output()
        .map_err(|e| {
            tool_cache::invalidate("python");
            format!("Failed to run pip show: {}", e)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(ver) = line.strip_prefix("Version: ") {
            return Ok(ver.trim().to_string());
        }
    }
    Ok("unknown".into())
}

fn extract_package_api(
    python: &str,
    package_name: &str,
    version: &str,
) -> Result<PackageApi, String> {
    let import_name = package_name.replace('-', "_");
    let script = format!(
        r#"
import json, sys, inspect, importlib
try:
    pkg = importlib.import_module("{}")
except Exception as e:
    print(json.dumps({{"error": str(e)}}))
    sys.exit(0)

members = []
for name in sorted(dir(pkg)):
    if name.startswith('_') and name != '__init__' and name != '__all__':
        continue
    try:
        obj = getattr(pkg, name)
        kind = type(obj).__name__
        sig = None
        if callable(obj):
            try:
                sig = str(inspect.signature(obj))
            except (ValueError, TypeError):
                pass
        members.append({{"name": name, "signature": sig, "kind": kind}})
    except Exception:
        pass

print(json.dumps(members))
"#,
        import_name
    );

    let mut child = tool_cache::background_command(python)
        .arg("-c")
        .arg(&script)
        .env("PYTHONIOENCODING", "utf-8")
        .env("MPLBACKEND", "Agg") // prevent matplotlib from opening windows
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            tool_cache::invalidate("python");
            format!("Failed to spawn Python: {}", e)
        })?;

    // Wait with timeout (some packages are slow to import)
    let timeout = std::time::Duration::from_secs(15);
    let start = std::time::Instant::now();
    let mut status = None;
    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(s)) => {
                status = Some(s);
                break;
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }
    if status.is_none() {
        let _ = child.kill();
        status = child.wait().ok();
    }
    let exit_status = status.unwrap_or_default();

    if !exit_status.success() {
        let mut stderr = String::new();
        if let Some(mut e) = child.stderr.take() {
            let _ = e.read_to_string(&mut stderr);
        }
        return Err(format!(
            "Failed to import {}: {}",
            package_name,
            stderr.trim().lines().last().unwrap_or("unknown error")
        ));
    }

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_string(&mut stdout)
            .map_err(|e| format!("Failed to read Python output: {}", e))?;
    }

    // Check for error in JSON output
    if let Ok(err_map) = serde_json::from_str::<HashMap<String, String>>(&stdout) {
        if let Some(err_msg) = err_map.get("error") {
            return Err(format!("Import error for {}: {}", package_name, err_msg));
        }
    }

    let members: Vec<ApiMember> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse API: {}", e))?;

    Ok(PackageApi {
        name: package_name.to_string(),
        version: version.to_string(),
        members,
    })
}
