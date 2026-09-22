use crate::app_paths;
use std::fs;

#[tauri::command]
pub fn read_config() -> Result<serde_json::Value, String> {
    let path = app_paths::config_file("windrc.json");
    let legacy_path = app_paths::legacy_roaming_file("windrc.json");
    app_paths::migrate_legacy_file(&path, &legacy_path)
        .map_err(|e| format!("Failed to migrate config: {}", e))?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

#[tauri::command]
pub fn write_config(options: serde_json::Value) -> Result<(), String> {
    let path = app_paths::config_file("windrc.json");
    let dir = path
        .parent()
        .ok_or_else(|| "Failed to determine config directory".to_string())?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let content = serde_json::to_string_pretty(&options)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))
}
