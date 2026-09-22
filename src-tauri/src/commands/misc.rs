use serde::Serialize;
use windows::Win32::UI::Input::Ime::{
    ImmGetContext, ImmGetOpenStatus, ImmReleaseContext, ImmSetOpenStatus,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::tool_cache;

#[tauri::command]
pub fn set_ime_enabled(enabled: bool) {
    unsafe {
        let hwnd = GetForegroundWindow();
        let himc = ImmGetContext(hwnd);
        if himc.is_invalid() {
            eprintln!("[IME] ImmGetContext returned invalid handle");
            return;
        }
        let current = ImmGetOpenStatus(himc).as_bool();
        eprintln!("[IME] current={current}, requested={enabled}");
        if current != enabled {
            let _ = ImmSetOpenStatus(himc, enabled.into());
            eprintln!("[IME] toggled to {enabled}");
        }
        let _ = ImmReleaseContext(hwnd, himc);
    }
}

#[derive(Debug, Serialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

fn detect_bash_path() -> Option<String> {
    tool_cache::bash_path().map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn exec_shell_command(command: String, cwd: Option<String>) -> Result<ShellOutput, String> {
    let bash = detect_bash_path()
        .ok_or_else(|| "bash not found. Install Git for Windows or MSYS2.".to_string())?;

    let output = tool_cache::background_command(&bash)
        .args(["-c", &command])
        .current_dir(cwd.unwrap_or_else(|| ".".to_string()))
        .output()
        .map_err(|e| {
            tool_cache::invalidate("bash");
            format!("Failed to execute: {}", e)
        })?;

    Ok(ShellOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
