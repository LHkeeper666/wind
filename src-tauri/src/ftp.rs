use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use suppaftp::tokio::AsyncRustlsFtpStream;
use suppaftp::types::FileType;
use tokio::sync::Mutex;

use log::{info, warn, error, debug};

use crate::app_paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtpConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FtpConnectionSummary {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub has_password: bool,
}

pub struct FtpSession {
    pub client: AsyncRustlsFtpStream,
}

pub struct FtpManager {
    sessions: HashMap<String, Arc<Mutex<FtpSession>>>,
    configs: HashMap<String, FtpConnectionConfig>,
    config_path: PathBuf,
    connection_gates: HashMap<String, Arc<Mutex<()>>>,
    epochs: HashMap<String, u64>,
}

impl FtpManager {
    pub fn new() -> Self {
        let config_path = app_paths::config_file("ftp-connections.json");
        info!("[FTP] Manager initialized, config path: {}", config_path.display());
        FtpManager {
            sessions: HashMap::new(),
            configs: HashMap::new(),
            config_path,
            connection_gates: HashMap::new(),
            epochs: HashMap::new(),
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

    pub fn add_config(&mut self, name: &str, host: &str, port: u16, user: &str, password: &str) -> Result<(), String> {
        if name.trim().is_empty() || name.contains('/') || host.trim().is_empty() || user.trim().is_empty() || port == 0 {
            return Err("Invalid FTP site name, host, port or user".into());
        }
        if self.configs.contains_key(name) {
            return Err(format!("Connection '{}' already exists", name));
        }
        let config = FtpConnectionConfig { name: name.into(), host: host.trim().into(), port, user: user.trim().into(), password: password.into() };
        let mut candidate = self.configs.clone();
        candidate.insert(name.into(), config);
        self.persist_config_map(&candidate)?;
        self.configs = candidate;
        Ok(())
    }

    pub fn disconnect(&mut self, name: &str) -> Result<(), String> {
        if !self.configs.contains_key(name) { return Err(format!("Connection '{}' not found", name)); }
        self.sessions.remove(name);
        self.bump_epoch(name);
        Ok(())
    }

    pub fn remove_config(&mut self, name: &str) -> Result<(), String> {
        if !self.configs.contains_key(name) { return Err(format!("Connection '{}' not found", name)); }
        let mut candidate = self.configs.clone();
        candidate.remove(name);
        self.persist_config_map(&candidate)?;
        self.configs = candidate;
        self.sessions.remove(name);
        self.bump_epoch(name);
        Ok(())
    }

    fn bump_epoch(&mut self, name: &str) { *self.epochs.entry(name.into()).or_default() += 1; }
    pub fn epoch(&self, name: &str) -> u64 { *self.epochs.get(name).unwrap_or(&0) }
    pub fn connection_gate(&mut self, name: &str) -> Arc<Mutex<()>> {
        self.connection_gates.entry(name.into()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
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

    pub fn read_config_summary(name: &str) -> Result<FtpConnectionSummary, String> {
        let manager = Self::new();
        let config = manager.load_configs_from_file()?
            .into_iter()
            .find(|config| config.name == name)
            .ok_or_else(|| format!("Connection '{}' not found", name))?;
        Ok(FtpConnectionSummary {
            name: config.name,
            host: config.host,
            port: config.port,
            user: config.user,
            has_password: !config.password.is_empty(),
        })
    }

    pub fn update_config(&mut self, name: &str, host: &str, port: u16, user: &str, password: Option<&str>) -> Result<(), String> {
        let old = self.configs.get(name).ok_or_else(|| format!("Connection '{}' not found", name))?;
        let updated = Self::edited_config(old, host, port, user, password)?;
        let mut candidate = self.configs.clone();
        candidate.insert(name.to_string(), updated);
        self.persist_config_map(&candidate)?;
        self.configs = candidate;
        self.bump_epoch(name);
        Ok(())
    }

    fn edited_config(old: &FtpConnectionConfig, host: &str, port: u16, user: &str, password: Option<&str>) -> Result<FtpConnectionConfig, String> {
        let host = host.trim();
        let user = user.trim();
        if host.is_empty() { return Err("Host is required".into()); }
        if user.is_empty() { return Err("User is required".into()); }
        if port == 0 { return Err("Port must be between 1 and 65535".into()); }
        Ok(FtpConnectionConfig {
            name: old.name.clone(), host: host.to_string(), port, user: user.to_string(),
            password: password.filter(|value| !value.is_empty()).unwrap_or(&old.password).to_string(),
        })
    }

    pub async fn open_session(config: &FtpConnectionConfig) -> Result<Arc<Mutex<FtpSession>>, String> {
        let addr = format!("{}:{}", config.host, config.port);
        let mut client = AsyncRustlsFtpStream::connect(&addr).await
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        client.login(&config.user, &config.password).await
            .map_err(|e| format!("Login failed: {}", e))?;
        if let Err(e) = client.opts("UTF8", Some("ON")).await {
            warn!("[FTP] reconnect updated '{}': OPTS UTF8 ON failed: {}", config.name, e);
        }
        client.transfer_type(FileType::Binary).await
            .map_err(|e| format!("Failed to set binary transfer mode: {}", e))?;
        Ok(Arc::new(Mutex::new(FtpSession { client })))
    }

    pub fn install_session_if_config_matches(&mut self, config: &FtpConnectionConfig, epoch: u64, session: Arc<Mutex<FtpSession>>) -> Result<(), String> {
        if self.configs.get(&config.name) != Some(config) || self.epoch(&config.name) != epoch {
            return Err("Connection settings changed while reconnecting; retry".into());
        }
        self.sessions.insert(config.name.clone(), session);
        Ok(())
    }

    /// Create an independent session for background transfers.
    /// Does NOT replace the main session — the caller is responsible for cleanup.
    pub async fn create_independent(&self, name: &str) -> Result<Arc<Mutex<FtpSession>>, String> {
        let config = self.get_config(name).ok_or_else(|| format!("Connection '{}' not found", name))?;
        Self::open_independent(&config).await
    }

    pub async fn open_independent(config: &FtpConnectionConfig) -> Result<Arc<Mutex<FtpSession>>, String> {
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
        info!("[FTP] created independent session for '{}'", config.name);
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
        self.persist_config_map(&self.configs)
    }

    fn persist_config_map(&self, configs_map: &HashMap<String, FtpConnectionConfig>) -> Result<(), String> {
        let configs: Vec<&FtpConnectionConfig> = configs_map.values().collect();
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

pub async fn independent_from_manager(manager: &Arc<Mutex<FtpManager>>, name: &str) -> Result<Arc<Mutex<FtpSession>>, String> {
    let config = manager.lock().await.get_config(name)
        .ok_or_else(|| format!("Connection '{}' not found", name))?;
    FtpManager::open_independent(&config).await
}

#[cfg(test)]
mod edit_tests {
    use super::*;

    fn config() -> FtpConnectionConfig {
        FtpConnectionConfig { name: "saved".into(), host: "old.example".into(), port: 21, user: "old".into(), password: "secret".into() }
    }

    #[test]
    fn editing_keeps_or_replaces_password_and_validates_fields() {
        let original = config();
        let kept = FtpManager::edited_config(&original, " new.example ", 2121, " new ", None).unwrap();
        assert_eq!((kept.name.as_str(), kept.host.as_str(), kept.user.as_str(), kept.password.as_str()), ("saved", "new.example", "new", "secret"));
        assert_eq!(FtpManager::edited_config(&original, "host", 21, "user", Some("")).unwrap().password, "secret");
        assert_eq!(FtpManager::edited_config(&original, "host", 21, "user", Some("changed")).unwrap().password, "changed");
        assert!(FtpManager::edited_config(&original, " ", 21, "user", None).is_err());
        assert!(FtpManager::edited_config(&original, "host", 0, "user", None).is_err());
        assert!(FtpManager::edited_config(&original, "host", 21, " ", None).is_err());
    }

    #[test]
    fn failed_write_does_not_change_memory() {
        let mut mgr = FtpManager::new();
        mgr.config_path = std::env::current_dir().unwrap(); // A directory cannot be overwritten as a file.
        mgr.configs.insert("saved".into(), config());
        assert!(mgr.update_config("saved", "new.example", 21, "new", None).is_err());
        assert_eq!(mgr.get_config("saved").unwrap().host, "old.example");
    }

    #[test]
    fn offline_add_and_disconnect_keep_saved_site() {
        let directory = tempfile::tempdir().unwrap();
        let mut mgr = FtpManager::new();
        mgr.config_path = directory.path().join("ftp-connections.json");
        mgr.add_config("offline", "127.0.0.1", 1, "anonymous", "anonymous").unwrap();
        assert!(mgr.get("offline").is_err());
        assert!(mgr.add_config("offline", "127.0.0.1", 1, "anonymous", "anonymous").is_err());
        assert_eq!(mgr.load_configs_from_file().unwrap().len(), 1);
        mgr.disconnect("offline").unwrap();
        assert!(mgr.get_config("offline").is_some());
        assert_eq!(mgr.load_configs_from_file().unwrap().len(), 1);
        mgr.remove_config("offline").unwrap();
        assert!(mgr.get_config("offline").is_none());
        assert!(mgr.load_configs_from_file().unwrap().is_empty());
    }

    #[test]
    fn failed_add_does_not_change_memory() {
        let mut mgr = FtpManager::new();
        mgr.config_path = std::env::current_dir().unwrap();
        assert!(mgr.add_config("offline", "localhost", 21, "user", "pass").is_err());
        assert!(mgr.get_config("offline").is_none());
    }

    #[test]
    fn independent_connection_wait_does_not_hold_manager_lock() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let directory = tempfile::tempdir().unwrap();
            let mut manager = FtpManager::new();
            manager.config_path = directory.path().join("ftp-connections.json");
            manager.add_config("waiting", "127.0.0.1", port, "user", "pass").unwrap();
            let manager = Arc::new(Mutex::new(manager));
            let task_manager = manager.clone();
            let task = tokio::spawn(async move { independent_from_manager(&task_manager, "waiting").await });
            let (_socket, _) = tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await.unwrap().unwrap();
            let guard = tokio::time::timeout(std::time::Duration::from_millis(100), manager.lock()).await
                .expect("FTP manager must be available during network wait");
            assert!(guard.get_config("waiting").is_some());
            drop(guard);
            task.abort();
        });
    }

    #[test]
    fn edit_and_disconnect_advance_session_epoch() {
        let directory = tempfile::tempdir().unwrap();
        let mut manager = FtpManager::new();
        manager.config_path = directory.path().join("ftp-connections.json");
        manager.add_config("site", "localhost", 21, "user", "pass").unwrap();
        let initial = manager.epoch("site");
        manager.update_config("site", "new-host", 21, "user", None).unwrap();
        assert!(manager.epoch("site") > initial);
        let updated = manager.epoch("site");
        manager.disconnect("site").unwrap();
        assert!(manager.epoch("site") > updated);
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
