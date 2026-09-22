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
    state
        .terminal
        .spawn(tab_id, &shell, cwd.as_deref(), cols, rows)
}

#[tauri::command]
pub fn terminal_input(tab_id: u32, data: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.terminal.write_input(tab_id, &data)
}

#[tauri::command]
pub fn terminal_resize(
    tab_id: u32,
    cols: u32,
    rows: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.terminal.resize(tab_id, cols, rows)
}

#[tauri::command]
pub fn terminal_kill(tab_id: u32, state: tauri::State<'_, AppState>) {
    state.terminal.kill(tab_id);
}
