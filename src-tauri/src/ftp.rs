use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use suppaftp::tokio::AsyncRustlsFtpStream;
use suppaftp::types::FileType;
use tokio::sync::Mutex;

use log::{info, warn, error, debug};

use crate::app_paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

pub struct FtpSession {
    pub client: AsyncRustlsFtpStream,
}

pub struct FtpManager {
    sessions: HashMap<String, Arc<Mutex<FtpSession>>>,
    configs: HashMap<String, FtpConnectionConfig>,
    config_path: PathBuf,
}

impl FtpManager {
    pub fn new() -> Self {
        let config_path = app_paths::config_file("ftp-connections.json");
        info!("[FTP] Manager initialized, config path: {}", config_path.display());
        FtpManager {
            sessions: HashMap::new(),
            configs: HashMap::new(),
            config_path,
        }
    }

    pub fn load_on_startup(&mut self) {
        let legacy_path = app_paths::legacy_roaming_file("ftp-connections.json");
        if let Err(error) = app_paths::migrate_legacy_file(&self.config_path, &legacy_path) {
            error!("[FTP] Failed to migrate stored connections: {}", error);
        }
        match self.load_configs_from_file() {
            Ok(configs) => {
                let count = configs.len();
                for config in configs {
                    info!("[FTP] Loaded stored connection: {} ({}@{})", config.name, config.user, config.host);
                    self.configs.insert(config.name.clone(), config);
                }
                if count > 0 {
                    info!("[FTP] Restored {} stored connection(s) from {}", count, self.config_path.display());
                }
            }
            Err(e) => {
                error!("[FTP] Failed to load stored connections: {}", e);
            }
        }
    }

    pub async fn connect(&mut self, name: &str, host: &str, port: u16, user: &str, password: &str) -> Result<(), String> {
        if self.sessions.contains_key(name) {
            warn!("[FTP] connect '{}': already connected", name);
            return Err(format!("Connection '{}' already exists", name));
        }

        info!("[FTP] connect '{}': {}:{} as {}", name, host, port, user);
        let addr = format!("{}:{}", host, port);
        let mut client = AsyncRustlsFtpStream::connect(&addr)
            .await
            .map_err(|e| {
                error!("[FTP] connect '{}': TCP connect failed: {}", name, e);
                format!("Failed to connect to {}: {}", addr, e)
            })?;

        client.login(user, password)
            .await
            .map_err(|e| {
                error!("[FTP] connect '{}': login failed: {}", name, e);
                format!("Login failed: {}", e)
            })?;

        // Enable UTF-8 for non-ASCII paths (Chinese, etc.)
        if let Err(e) = client.opts("UTF8", Some("ON")).await {
            warn!("[FTP] connect '{}': OPTS UTF8 ON failed (ignored): {}", name, e);
        }

        // Force binary transfer mode to prevent file corruption
        client.transfer_type(FileType::Binary).await.map_err(|e| {
            error!("[FTP] connect '{}': TYPE I failed: {}", name, e);
            format!("Failed to set binary transfer mode: {}", e)
        })?;

        info!("[FTP] connect '{}': authenticated successfully", name);

        let config = FtpConnectionConfig {
            name: name.to_string(),
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
        };

        let session = FtpSession { client };
        self.sessions.insert(name.to_string(), Arc::new(Mutex::new(session)));
        self.configs.insert(name.to_string(), config);
        self.persist_configs()?;
        info!("[FTP] connect '{}': session stored, config persisted", name);
        Ok(())
    }

    pub async fn disconnect(&mut self, name: &str) -> Result<(), String> {
        info!("[FTP] disconnect '{}': closing session", name);
        if let Some(session) = self.sessions.remove(name) {
            let mut ftp = session.lock().await;
            if let Err(e) = ftp.client.quit().await {
                warn!("[FTP] disconnect '{}': QUIT error (ignored): {}", name, e);
            }
        }
        self.configs.remove(name);
        self.persist_configs()?;
        info!("[FTP] disconnect '{}': removed, config persisted", name);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Arc<Mutex<FtpSession>>, String> {
        self.sessions.get(name)
            .cloned()
            .ok_or_else(|| format!("Connection '{}' not found", name))
    }

    pub fn has(&self, name: &str) -> bool {
        self.sessions.contains_key(name)
    }

    pub fn get_configs(&self) -> Vec<FtpConnectionConfig> {
        self.configs.values().cloned().collect()
    }

    pub fn has_config(&self, name: &str) -> bool {
        self.configs.contains_key(name)
    }

    pub fn get_config(&self, name: &str) -> Option<FtpConnectionConfig> {
        self.configs.get(name).cloned()
    }

    /// Create an independent session for background transfers.
    /// Does NOT replace the main session — the caller is responsible for cleanup.
    pub async fn create_independent(&self, name: &str) -> Result<Arc<Mutex<FtpSession>>, String> {
        let config = self.configs.get(name)
            .ok_or_else(|| format!("No config for connection '{}'", name))?.clone();
        let addr = format!("{}:{}", config.host, config.port);
        let mut client = AsyncRustlsFtpStream::connect(&addr)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        client.login(&config.user, &config.password)
            .await
            .map_err(|e| format!("Login failed: {}", e))?;
        if let Err(e) = client.opts("UTF8", Some("ON")).await {
            warn!("[FTP] independent session: OPTS UTF8 ON failed (ignored): {}", e);
        }
        client.transfer_type(FileType::Binary).await.map_err(|e| {
            error!("[FTP] independent session: TYPE I failed: {}", e);
            format!("Failed to set binary transfer mode: {}", e)
        })?;
        info!("[FTP] created independent session for '{name}'");
        Ok(Arc::new(Mutex::new(FtpSession { client })))
    }

    /// Reconnect using a stored config
    pub async fn reconnect(&mut self, name: &str) -> Result<(), String> {
        let config = self.configs.get(name)
            .ok_or_else(|| format!("No config for connection '{}'", name))?
            .clone();

        info!("[FTP] reconnect '{}': {}:{} as {}", name, config.host, config.port, config.user);
        let addr = format!("{}:{}", config.host, config.port);
        let mut client = AsyncRustlsFtpStream::connect(&addr)
            .await
            .map_err(|e| {
                error!("[FTP] reconnect '{}': TCP connect failed: {}", name, e);
                format!("Failed to reconnect to {}: {}", addr, e)
            })?;

        client.login(&config.user, &config.password)
            .await
            .map_err(|e| {
                error!("[FTP] reconnect '{}': login failed: {}", name, e);
                format!("Re-login failed: {}", e)
            })?;

        client.transfer_type(FileType::Binary).await.map_err(|e| {
            error!("[FTP] reconnect '{}': TYPE I failed: {}", name, e);
            format!("Failed to set binary transfer mode: {}", e)
        })?;

        info!("[FTP] reconnect '{}': re-authenticated", name);
        self.sessions.insert(name.to_string(), Arc::new(Mutex::new(FtpSession { client })));
        Ok(())
    }

    /// Ensure a session is active; reconnect from stored config if disconnected
    pub async fn ensure_connected(&mut self, name: &str) -> Result<(), String> {
        if self.sessions.contains_key(name) {
            let session = self.get(name)?;
            let mut ftp = session.lock().await;
            match ftp.client.noop().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    warn!("[FTP] ensure_connected '{}': NOOP failed (session stale): {}", name, e);
                    drop(ftp);
                    self.sessions.remove(name);
                }
            }
        }
        info!("[FTP] ensure_connected '{}': reconnecting from stored config", name);
        self.reconnect(name).await
    }

    fn persist_configs(&self) -> Result<(), String> {
        let configs: Vec<&FtpConnectionConfig> = self.configs.values().collect();
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&configs)
            .map_err(|e| format!("Failed to serialize configs: {}", e))?;
        std::fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        debug!("[FTP] persist: {} connection(s) saved to {}", configs.len(), self.config_path.display());
        Ok(())
    }

    fn load_configs_from_file(&self) -> Result<Vec<FtpConnectionConfig>, String> {
        if !self.config_path.exists() {
            debug!("[FTP] load_configs: no config file at {}", self.config_path.display());
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        let configs: Vec<FtpConnectionConfig> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        debug!("[FTP] load_configs: read {} connection(s)", configs.len());
        Ok(configs)
    }
}

/// Parse MLSD output into (name, is_dir, size, modified) tuples.
/// MLSD format (RFC 3659): fact=value;fact=value; filename
pub fn parse_mld_line(line: &str) -> Option<(String, bool, Option<u64>, Option<u64>)> {
    // Find the last semicolon; the filename starts after the space following it.
    // Using rfind(' ') breaks on filenames containing spaces.
    let name = if let Some(semi_pos) = line.rfind(';') {
        let after_semi = &line[semi_pos + 1..];
        if let Some(space_pos) = after_semi.find(' ') {
            after_semi[space_pos + 1..].trim().to_string()
        } else {
            // No space after semicolon — the rest might be a fact without trailing semicolon
            // Fall back to rfind(' ') but only if after_semi contains a space
            return parse_mld_line_fallback(line);
        }
    } else {
        // No semicolons — fall back to space-based parsing
        return parse_mld_line_fallback(line);
    };

    if name.is_empty() || name == "." || name == ".." {
        return None;
    }

    // Facts are everything before the filename (including the last semicolon)
    let facts = &line[..line.len() - name.len()];
    let facts_lower = facts.to_lowercase();
    let is_dir = facts_lower.contains("type=dir");
    let mut size: Option<u64> = None;
    let mut modified: Option<u64> = None;

    for fact in facts.split(';') {
        let fact_lower = fact.trim().to_lowercase();
        if let Some(val) = fact_lower.strip_prefix("size=") {
            size = val.parse().ok();
        } else if let Some(val) = fact_lower.strip_prefix("modify=") {
            modified = parse_ftp_timestamp(val);
        }
    }

    Some((name, is_dir, size, modified))
}

fn parse_mld_line_fallback(line: &str) -> Option<(String, bool, Option<u64>, Option<u64>)> {
    let space_pos = line.rfind(' ')?;
    let facts = &line[..space_pos];
    let name = line[space_pos + 1..].trim().to_string();
    if name == "." || name == ".." { return None; }
    let facts_lower = facts.to_lowercase();
    let is_dir = facts_lower.contains("type=dir");
    let mut size: Option<u64> = None;
    let mut modified: Option<u64> = None;
    for fact in facts.split(';') {
        let fact_lower = fact.trim().to_lowercase();
        if let Some(val) = fact_lower.strip_prefix("size=") {
            size = val.parse().ok();
        } else if let Some(val) = fact_lower.strip_prefix("modify=") {
            modified = parse_ftp_timestamp(val);
        }
    }
    Some((name, is_dir, size, modified))
}

/// Parse Unix `ls -l` style LIST output.
pub fn parse_list_line(line: &str) -> Option<(String, bool, Option<u64>, Option<u64>)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("total ") {
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let is_dir = parts[0].starts_with('d');
    let size: Option<u64> = parts[4].parse().ok();

    let name_idx = 8;
    if name_idx >= parts.len() {
        return None;
    }
    let name = parts[name_idx..].join(" ");

    if name == "." || name == ".." {
        return None;
    }

    Some((name, is_dir, size, None))
}

/// Recursively list a remote FTP directory tree using MLSD (LIST fallback).
/// Returns Vec<(full_remote_path, size_bytes, is_dir)> for all entries discovered.
pub async fn list_dir_recursive(
    client: &mut AsyncRustlsFtpStream,
    remote_path: &str,
) -> Result<Vec<(String, u64, bool)>, String> {
    let path = remote_path.trim_end_matches('/');
    let mut results: Vec<(String, u64, bool)> = Vec::new();
    let mut dirs_to_visit: Vec<String> = Vec::new();

    // List current directory
    let lines = match client.mlsd(Some(path)).await {
        Ok(lines) => lines,
        Err(e) => {
            warn!("[FTP] list_dir_recursive: MLSD failed ({}), falling back to LIST", e);
            client.list(Some(path)).await
                .map_err(|e2| format!("Failed to list directory '{}': MLSD: {}, LIST: {}", path, e, e2))?
        }
    };

    for line in &lines {
        if let Some((name, is_dir, size, _modified)) = parse_mld_line(line)
            .or_else(|| parse_list_line(line))
        {
            let full_path = format!("{}/{}", path, name);
            if is_dir {
                results.push((full_path.clone(), 0, true));
                dirs_to_visit.push(full_path);
            } else {
                results.push((full_path, size.unwrap_or(0), false));
            }
        }
    }

    // Recursively visit subdirectories
    for dir_path in dirs_to_visit {
        match Box::pin(list_dir_recursive(client, &dir_path)).await {
            Ok(mut sub_results) => results.append(&mut sub_results),
            Err(e) => return Err(e),
        }
    }

    Ok(results)
}

fn parse_ftp_timestamp(val: &str) -> Option<u64> {
    if val.len() < 14 {
        return None;
    }
    let year: i32 = val[0..4].parse().ok()?;
    let month: u32 = val[4..6].parse().ok()?;
    if !(1..=12).contains(&month) { return None; }
    let day: u32 = val[6..8].parse().ok()?;
    if !(1..=31).contains(&day) { return None; }
    let hour: u32 = val[8..10].parse().ok()?;
    if hour > 23 { return None; }
    let min: u32 = val[10..12].parse().ok()?;
    if min > 59 { return None; }
    let sec: u32 = val[12..14].parse().ok()?;
    if sec > 59 { return None; }

    let days_before_year = |y: i32| -> u64 {
        let y = y - 1;
        (y as u64) * 365 + (y as u64 / 4) - (y as u64 / 100) + (y as u64 / 400)
    };
    let days_in_month = |m: u32, y: i32| -> u64 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 }
            }
            _ => 0,
        }
    };

    let mut days: u64 = days_before_year(year);
    for m in 1..month {
        days += days_in_month(m, year);
    }
    days += (day - 1) as u64;

    let total_secs = days * 86400 + (hour as u64) * 3600 + (min as u64) * 60 + sec as u64;
    let epoch_offset_days: u64 = 719162;
    let epoch_offset_secs = epoch_offset_days * 86400;

    if total_secs < epoch_offset_secs { return None; }
    Some(total_secs - epoch_offset_secs)
}
