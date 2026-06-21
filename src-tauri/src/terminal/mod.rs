use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct Terminal {
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    app_handle: Option<AppHandle>,
}

impl Terminal {
    pub fn new() -> Self {
        Terminal {
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn spawn(&mut self, shell: &str) -> Result<(), String> {
        // Kill existing process if any
        self.kill();

        let mut cmd = match shell {
            "powershell" => {
                let mut c = Command::new("powershell.exe");
                c.arg("-NoLogo");
                c.arg("-NoProfile");
                c
            }
            "cmd" => {
                let mut c = Command::new("cmd.exe");
                c.arg("/K");
                c.arg("echo."); // Print empty line to initialize
                c
            }
            "git-bash" => {
                let mut c = Command::new("C:\\Program Files\\Git\\bin\\bash.exe");
                c.arg("--login");
                c.arg("-i");
                c
            }
            _ => return Err(format!("Unknown shell: {}", shell)),
        };

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        *self.child.lock().unwrap() = Some(child);

        if let Some(stdin) = stdin {
            *self.stdin.lock().unwrap() = Some(Box::new(stdin));
        }

        let app_handle = self.app_handle.clone();

        // Spawn thread to read stdout
        if let Some(mut stdout) = stdout {
            let handle = app_handle.clone();
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stdout.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                            if let Some(ref handle) = handle {
                                let _ = handle.emit("terminal-output", &data);
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Spawn thread to read stderr
        if let Some(mut stderr) = stderr {
            let handle = app_handle.clone();
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                            if let Some(ref handle) = handle {
                                let _ = handle.emit("terminal-output", &data);
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Ok(())
    }

    pub fn write_input(&self, data: &str) -> Result<(), String> {
        let mut stdin = self.stdin.lock().unwrap();
        if let Some(ref mut stdin) = *stdin {
            stdin
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
            Ok(())
        } else {
            Err("No stdin available".to_string())
        }
    }

    pub fn kill(&self) {
        let mut child = self.child.lock().unwrap();
        if let Some(ref mut child) = *child {
            let _ = child.kill();
        }
        *child = None;
        *self.stdin.lock().unwrap() = None;
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.kill();
    }
}
