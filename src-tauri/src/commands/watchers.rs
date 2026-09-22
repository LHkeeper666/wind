use crate::AppState;

#[tauri::command]
pub fn start_watch_file(
    path: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .file_watcher
        .lock()
        .map_err(|e| e.to_string())?
        .start(&path, app_handle);
    Ok(())
}

#[tauri::command]
pub fn stop_watch_file(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.file_watcher.lock().map_err(|e| e.to_string())?.stop();
    Ok(())
}

#[tauri::command]
pub fn start_watch_directory(path: String, app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.directory_watcher.lock().map_err(|e| e.to_string())?.start(&path, app_handle)
}

#[tauri::command]
pub fn stop_watch_directory(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.directory_watcher.lock().map_err(|e| e.to_string())?.stop();
    Ok(())
}
