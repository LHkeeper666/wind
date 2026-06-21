use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Neovim {
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    msg_id: Arc<Mutex<u64>>,
}

impl Neovim {
    pub fn new() -> Self {
        Neovim {
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            msg_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn spawn(&mut self) -> Result<(), String> {
        // Kill existing process if any
        self.kill();

        let mut child = Command::new("nvim")
            .arg("--embed")
            .arg("--headless")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Neovim: {}", e))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        *self.child.lock().unwrap() = Some(child);

        if let Some(stdin) = stdin {
            *self.stdin.lock().unwrap() = Some(Box::new(stdin));
        }

        // Spawn thread to read stdout (msgpack-rpc responses)
        if let Some(mut stdout) = stdout {
            thread::spawn(move || {
                let mut buffer = vec![0u8; 4096];
                loop {
                    match stdout.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(_n) => {
                            // TODO: Parse msgpack-rpc messages
                            // For now, just log
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Spawn thread to read stderr
        if let Some(mut stderr) = stderr {
            thread::spawn(move || {
                let mut buffer = vec![0u8; 4096];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(_n) => {
                            // Log stderr
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Initialize Neovim UI
        self.nvim_ui_attach()?;

        Ok(())
    }

    fn nvim_ui_attach(&self) -> Result<(), String> {
        // Send nvim_ui_attach with ext_linegrid option
        let msg = self.create_request("nvim_ui_attach", vec![
            rmpv::Value::Integer(80.into()),  // width
            rmpv::Value::Integer(24.into()),  // height
            rmpv::Value::Map(vec![
                (
                    rmpv::Value::String("ext_linegrid".into()),
                    rmpv::Value::Boolean(true),
                ),
                (
                    rmpv::Value::String("ext_multigrid".into()),
                    rmpv::Value::Boolean(false),
                ),
                (
                    rmpv::Value::String("rgb".into()),
                    rmpv::Value::Boolean(true),
                ),
            ]),
        ]);

        self.send_message(msg)
    }

    fn create_request(&self, method: &str, params: Vec<rmpv::Value>) -> rmpv::Value {
        let mut id = self.msg_id.lock().unwrap();
        *id += 1;
        let msg_id = *id;

        rmpv::Value::Array(vec![
            rmpv::Value::Integer(0.into()),  // Request type
            rmpv::Value::Integer(msg_id.into()),
            rmpv::Value::String(method.into()),
            rmpv::Value::Array(params),
        ])
    }

    fn send_message(&self, msg: rmpv::Value) -> Result<(), String> {
        let mut stdin = self.stdin.lock().unwrap();
        if let Some(ref mut stdin) = *stdin {
            let mut buf = Vec::new();
            rmpv::encode::write_value(&mut buf, &msg)
                .map_err(|e| format!("Failed to encode msgpack: {}", e))?;
            stdin
                .write_all(&buf)
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
            Ok(())
        } else {
            Err("No stdin available".to_string())
        }
    }

    pub fn send_input(&self, keys: &str) -> Result<(), String> {
        let msg = self.create_request("nvim_input", vec![
            rmpv::Value::String(keys.into()),
        ]);
        self.send_message(msg)
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

impl Drop for Neovim {
    fn drop(&mut self) {
        self.kill();
    }
}
