use crate::AppState;

#[tauri::command]
pub fn terminal_spawn(
    tab_id: u32,
    shell: String,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::debug!("[terminal_cmd] terminal_spawn: tab_id={}, shell={}", tab_id, shell);
    state
        .terminal
        .spawn(tab_id, &shell, cwd.as_deref(), cols, rows)
        .map_err(|e| {
            log::error!("[terminal_cmd] terminal_spawn failed: {}", e);
            e
        })
}

#[tauri::command]
pub fn terminal_input(tab_id: u32, data: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    log::debug!("[terminal_cmd] terminal_input: tab_id={}, len={}", tab_id, data.len());
    state.terminal.write_input(tab_id, &data)
        .map_err(|e| {
            log::error!("[terminal_cmd] terminal_input failed: {}", e);
            e
        })
}

#[tauri::command]
pub fn terminal_resize(
    tab_id: u32,
    cols: u32,
    rows: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::debug!("[terminal_cmd] terminal_resize: tab_id={}, cols={}, rows={}", tab_id, cols, rows);
    state.terminal.resize(tab_id, cols, rows)
        .map_err(|e| {
            log::error!("[terminal_cmd] terminal_resize failed: {}", e);
            e
        })
}

#[tauri::command]
pub fn terminal_kill(tab_id: u32, state: tauri::State<'_, AppState>) {
    log::debug!("[terminal_cmd] terminal_kill: tab_id={}", tab_id);
    state.terminal.kill(tab_id);
}
