use portable_pty::{ChildKiller, CommandBuilder, PtyPair, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct Terminal {
    pty_pair: Arc<Mutex<Option<PtyPair>>>,
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    child_killer: Arc<Mutex<Option<Box<dyn ChildKiller + Send + Sync>>>>,
    app_handle: Option<AppHandle>,
}

impl Terminal {
    pub fn new() -> Self {
        Terminal {
            pty_pair: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            child_killer: Arc::new(Mutex::new(None)),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn spawn(&mut self, shell: &str, cwd: Option<&str>, cols: u16, rows: u16) -> Result<(), String> {
        // Kill existing process if any
        self.kill();

        let pty_system = native_pty_system();

        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        // Build command based on shell type
        let mut cmd = match shell {
            "powershell" => {
                let mut c = CommandBuilder::new("powershell.exe");
                c.arg("-NoLogo");
                c.arg("-NoProfile");
                c
            }
            "cmd" => {
                let c = CommandBuilder::new("cmd.exe");
                c
            }
            "git-bash" => {
                let mut c = CommandBuilder::new("C:\\Program Files\\Git\\bin\\bash.exe");
                c.arg("--login");
                c.arg("-i");
                c
            }
            _ => return Err(format!("Unknown shell: {}", shell)),
        };

        // Set working directory if provided
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        // Spawn child process in PTY
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Store child killer for cleanup
        let killer = child.clone_killer();
        *self.child_killer.lock().unwrap() = Some(killer);

        // Get writer for sending input
        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;
        *self.writer.lock().unwrap() = Some(writer);

        // Get reader for receiving output
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Store PTY pair
        *self.pty_pair.lock().unwrap() = Some(pty_pair);

        // Spawn thread to read output
        let app_handle = self.app_handle.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit("terminal-output", &data);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    pub fn write_input(&self, data: &str) -> Result<(), String> {
        let mut writer = self.writer.lock().unwrap();
        if let Some(ref mut writer) = *writer {
            writer
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to PTY: {}", e))?;
            writer
                .flush()
                .map_err(|e| format!("Failed to flush PTY: {}", e))?;
            Ok(())
        } else {
            Err("No PTY writer available".to_string())
        }
    }

    pub fn resize(&self, cols: u32, rows: u32) -> Result<(), String> {
        let pty_pair = self.pty_pair.lock().unwrap();
        if let Some(ref pty_pair) = *pty_pair {
            pty_pair
                .master
                .resize(PtySize {
                    rows: rows as u16,
                    cols: cols as u16,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| format!("Failed to resize PTY: {}", e))?;
            Ok(())
        } else {
            Err("No PTY available".to_string())
        }
    }

    pub fn kill(&self) {
        // Kill child process
        if let Some(mut killer) = self.child_killer.lock().unwrap().take() {
            let _ = killer.kill();
        }

        // Clear writer
        *self.writer.lock().unwrap() = None;

        // Drop PTY pair (this will close the handles)
        *self.pty_pair.lock().unwrap() = None;
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.kill();
    }
}
