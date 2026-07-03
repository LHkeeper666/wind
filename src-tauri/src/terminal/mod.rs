use portable_pty::{ChildKiller, CommandBuilder, PtyPair, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

struct TerminalInstance {
    pty_pair: Arc<Mutex<Option<PtyPair>>>,
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    child_killer: Arc<Mutex<Option<Box<dyn ChildKiller + Send + Sync>>>>,
}

pub struct TerminalManager {
    instances: Mutex<HashMap<u32, TerminalInstance>>,
    app_handle: Option<AppHandle>,
}

impl TerminalManager {
    pub fn new() -> Self {
        TerminalManager {
            instances: Mutex::new(HashMap::new()),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn spawn(&self, tab_id: u32, shell: &str, cwd: Option<&str>, cols: u16, rows: u16) -> Result<(), String> {
        // Kill existing instance for this tab if any
        self.kill(tab_id);

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

        // Get writer for sending input
        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        // Get reader for receiving output
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Store instance
        let instance = TerminalInstance {
            pty_pair: Arc::new(Mutex::new(Some(pty_pair))),
            writer: Arc::new(Mutex::new(Some(writer))),
            child_killer: Arc::new(Mutex::new(Some(killer))),
        };

        {
            let mut instances = self.instances.lock().unwrap();
            instances.insert(tab_id, instance);
        }

        // Spawn thread to read output
        let app_handle = self.app_handle.clone();
        let event_name = format!("terminal-output-{}", tab_id);
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit(&event_name, &data);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    pub fn write_input(&self, tab_id: u32, data: &str) -> Result<(), String> {
        let instances = self.instances.lock().unwrap();
        if let Some(instance) = instances.get(&tab_id) {
            let mut writer = instance.writer.lock().unwrap();
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
        } else {
            Err(format!("No terminal instance for tab {}", tab_id))
        }
    }

    pub fn resize(&self, tab_id: u32, cols: u32, rows: u32) -> Result<(), String> {
        let instances = self.instances.lock().unwrap();
        if let Some(instance) = instances.get(&tab_id) {
            let pty_pair = instance.pty_pair.lock().unwrap();
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
        } else {
            Err(format!("No terminal instance for tab {}", tab_id))
        }
    }

    pub fn kill(&self, tab_id: u32) {
        let mut instances = self.instances.lock().unwrap();
        if let Some(instance) = instances.remove(&tab_id) {
            // Kill child process
            if let Some(mut killer) = instance.child_killer.lock().unwrap().take() {
                let _ = killer.kill();
            }
            // Clear writer
            *instance.writer.lock().unwrap() = None;
            // Drop PTY pair
            *instance.pty_pair.lock().unwrap() = None;
        }
    }

    pub fn kill_all(&self) {
        let mut instances = self.instances.lock().unwrap();
        for (_, instance) in instances.drain() {
            if let Some(mut killer) = instance.child_killer.lock().unwrap().take() {
                let _ = killer.kill();
            }
            *instance.writer.lock().unwrap() = None;
            *instance.pty_pair.lock().unwrap() = None;
        }
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        self.kill_all();
    }
}
