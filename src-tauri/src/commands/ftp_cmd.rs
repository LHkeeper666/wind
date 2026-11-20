use crate::{AppState, FileEntry};
use log::{info, warn, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn ensure_session(state: &AppState, name: &str) -> Result<Arc<Mutex<crate::ftp::FtpSession>>, String> {
    let gate = {
        let mut mgr = state.ftp_manager.lock().await;
        mgr.connection_gate(name)
    };
    let _guard = gate.lock().await;
    let (existing, config, epoch) = {
        let mgr = state.ftp_manager.lock().await;
        let config = mgr.get_config(name).ok_or_else(|| format!("Connection '{}' not found", name))?;
        (mgr.get(name).ok(), config, mgr.epoch(name))
    };
    if let Some(session) = existing {
        if session.lock().await.client.noop().await.is_ok() { return Ok(session); }
        let mut mgr = state.ftp_manager.lock().await;
        if mgr.get(name).ok().as_ref().is_some_and(|current| Arc::ptr_eq(current, &session)) {
            mgr.disconnect(name)?;
        }
        drop(mgr);
        let (config, epoch) = {
            let mgr = state.ftp_manager.lock().await;
            (mgr.get_config(name).ok_or_else(|| format!("Connection '{}' not found", name))?, mgr.epoch(name))
        };
        let replacement = crate::ftp::FtpManager::open_session(&config).await?;
        state.ftp_manager.lock().await.install_session_if_config_matches(&config, epoch, replacement.clone())?;
        return Ok(replacement);
    }
    let session = crate::ftp::FtpManager::open_session(&config).await?;
    let mut mgr = state.ftp_manager.lock().await;
    mgr.install_session_if_config_matches(&config, epoch, session.clone())?;
    Ok(session)
}

async fn replace_session(state: &AppState, name: &str) -> Result<Arc<Mutex<crate::ftp::FtpSession>>, String> {
    let gate = { state.ftp_manager.lock().await.connection_gate(name) };
    let _guard = gate.lock().await;
    let (config, epoch) = {
        let mgr = state.ftp_manager.lock().await;
        (mgr.get_config(name).ok_or_else(|| format!("Connection '{}' not found", name))?, mgr.epoch(name))
    };
    let session = crate::ftp::FtpManager::open_session(&config).await?;
    state.ftp_manager.lock().await.install_session_if_config_matches(&config, epoch, session.clone())?;
    Ok(session)
}

/// Parse `ftp://<name>/<path>` into (connection_name, remote_path).
pub fn parse_ftp_url(url: &str) -> Result<(&str, &str), String> {
    let url = url
        .strip_prefix("ftp://")
        .ok_or_else(|| format!("Not an FTP URL: {}", url))?;
    let slash_pos = url.find('/');
    let (name, path) = match slash_pos {
        Some(pos) => {
            let (n, p) = url.split_at(pos);
            (n, if p.is_empty() { "/" } else { p })
        }
        None => (url, "/"),
    };
    Ok((name, path))
}

fn invalid_move_target(source: &str, target: &str, is_dir: bool) -> bool {
    let source = source.trim_end_matches('/');
    let target = target.trim_end_matches('/');
    source == target || (is_dir && target.starts_with(&format!("{source}/")))
}

fn rename_rejected(error: &str) -> String {
    format!("FTP_RENAME_REJECTED: {error}")
}

pub async fn try_list_dir(
    session: std::sync::Arc<tokio::sync::Mutex<crate::ftp::FtpSession>>,
    conn_name: &str,
    remote_path: &str,
) -> Result<Vec<FileEntry>, String> {
    let mut ftp = session.lock().await;

    // Try MLSD first, fall back to LIST
    let mut entries = match ftp.client.mlsd(Some(remote_path)).await {
        Ok(lines) => {
            let mut entries = Vec::new();
            for line in &lines {
                if let Some((name, is_dir, size, modified)) = crate::ftp::parse_mld_line(line) {
                    let full_path = format!(
                        "ftp://{}{}/{}",
                        conn_name,
                        remote_path.trim_end_matches('/'),
                        name
                    );
                    entries.push(FileEntry {
                        name,
                        path: full_path,
                        is_dir,
                        size,
                        is_hidden: false,
                        modified,
                        created: None,
                        children: None,
                    });
                }
            }
            info!(
                "[FTP] read_directory: MLSD returned {} entries",
                entries.len()
            );
            entries
        }
        Err(e) => {
            warn!(
                "[FTP] read_directory: MLSD failed ({}), falling back to LIST",
                e
            );
            let lines = ftp
                .client
                .list(Some(remote_path))
                .await
                .map_err(|e| format!("Failed to list directory: {}", e))?;
            let mut entries = Vec::new();
            for line in &lines {
                if let Some((name, is_dir, size, _modified)) = crate::ftp::parse_list_line(line) {
                    let full_path = format!(
                        "ftp://{}{}/{}",
                        conn_name,
                        remote_path.trim_end_matches('/'),
                        name
                    );
                    entries.push(FileEntry {
                        name,
                        path: full_path,
                        is_dir,
                        size,
                        is_hidden: false,
                        modified: None,
                        created: None,
                        children: None,
                    });
                }
            }
            info!(
                "[FTP] read_directory: LIST returned {} entries",
                entries.len()
            );
            entries
        }
    };

    // Deduplicate by path
    {
        let before = entries.len();
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| seen.insert(e.path.clone()));
        let removed = before - entries.len();
        if removed > 0 {
            debug!(
                "[FTP] read_directory: dedup removed {} duplicate entries, {} remaining",
                removed,
                entries.len()
            );
        }
    }

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

#[tauri::command]
pub async fn ftp_connect(
    name: String,
    host: String,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let port = port.unwrap_or(21);
    let user = user.unwrap_or_else(|| "anonymous".to_string());
    let password = password.unwrap_or_else(|| "anonymous".to_string());
    info!(
        "[FTP] command ftp_connect: name={} host={}:{} user={}",
        name, host, port, user
    );
    state.ftp_manager.lock().await.add_config(&name, &host, port, &user, &password)?;
    replace_session(&state, &name).await?;
    Ok(format!("Connected to {}", name))
}

#[tauri::command]
pub async fn ftp_add(name: String, host: String, port: Option<u16>, user: Option<String>, password: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.ftp_manager.lock().await.add_config(&name, &host, port.unwrap_or(21), &user.unwrap_or_else(|| "anonymous".into()), &password.unwrap_or_else(|| "anonymous".into()))?;
    Ok(format!("Saved FTP site {}", name))
}

#[tauri::command]
pub async fn ftp_disconnect(name: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("[FTP] command ftp_disconnect: name={}", name);
    let mut mgr = state.ftp_manager.lock().await;
    mgr.disconnect(&name)?;
    Ok(format!("Disconnected from {}", name))
}

#[tauri::command]
pub async fn ftp_remove(name: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.ftp_manager.lock().await.remove_config(&name)?;
    Ok(format!("Removed FTP site {}", name))
}

#[tauri::command]
pub async fn ftp_read_directory(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    info!(
        "[FTP] command ftp_read_directory: conn={} path={}",
        conn_name, remote_path
    );

    // Never hold the manager lock while awaiting the FTP server.
    let session = ensure_session(&state, conn_name).await?;
    match try_list_dir(session.clone(), conn_name, remote_path).await {
        Ok(entries) => Ok(entries),
        Err(original) => {
            // A permissions/path error leaves the control connection alive.
            let healthy = session.lock().await.client.noop().await.is_ok();
            if healthy { return Err(original); }
            {
                let mut mgr = state.ftp_manager.lock().await;
                if mgr.get(conn_name).ok().as_ref().is_some_and(|current| Arc::ptr_eq(current, &session)) {
                    mgr.disconnect(conn_name)?;
                }
            }
            let replacement = ensure_session(&state, conn_name).await?;
            try_list_dir(replacement, conn_name, remote_path).await
        }
    }
}

#[tauri::command]
pub async fn ftp_delete(
    path: String,
    _permanent: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    info!(
        "[FTP] command ftp_delete: conn={} path={}",
        conn_name, remote_path
    );

    let session = ensure_session(&state, conn_name).await?;

    let mut ftp = session.lock().await;
    // Try file delete first, fall back to directory delete
    match ftp.client.rm(&remote_path).await {
        Ok(()) => {
            info!("[FTP] delete: removed file {}", remote_path);
            return Ok(());
        }
        Err(e) => {
            warn!("[FTP] delete: rm failed ({}), trying rmdir...", e);
        }
    }
    ftp.client
        .rmdir(&remote_path)
        .await
        .map_err(|e| format!("Failed to delete: {}", e))?;
    info!("[FTP] delete: removed directory {}", remote_path);
    Ok(())
}

#[tauri::command]
pub async fn ftp_rename(
    old_path: String,
    new_path: String,
    is_dir: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let (conn_name, remote_path) = parse_ftp_url(&old_path)?;
    let (new_conn_name, new_remote) = parse_ftp_url(&new_path)?;
    if conn_name != new_conn_name {
        return Err("Cannot rename across FTP connections".into());
    }
    if invalid_move_target(remote_path, new_remote, is_dir == Some(true)) {
        return Err("Cannot move a directory into itself or its descendant".into());
    }
    info!(
        "[FTP] command ftp_rename: conn={} from={} to={}",
        conn_name, remote_path, new_remote
    );

    // Try server-side rename first (fast path)
    {
        let session = ensure_session(&state, conn_name).await?;
        let mut ftp = session.lock().await;
        match ftp.client.rename(remote_path, new_remote).await {
            Ok(()) => {
                info!("[FTP] rename: server-side rename OK");
                return Ok(new_path.to_string());
            }
            Err(e) => {
                warn!(
                    "[FTP] rename: server-side rename failed ({}); caller may queue a tracked copy",
                    e
                );
                return Err(rename_rejected(&e.to_string()));
            }
        }
    }

}

#[cfg(test)]
mod move_tests {
    use super::{invalid_move_target, rename_rejected};

    #[test]
    fn rejected_server_rename_is_routed_to_tracked_fallback() {
        assert!(rename_rejected("550 Error during rename operation").starts_with("FTP_RENAME_REJECTED:"));
        assert!(invalid_move_target("/src", "/src/nested", true));
        assert!(invalid_move_target("/src/file", "/src/file", false));
        assert!(!invalid_move_target("/src", "/other", true));
    }
}

#[tauri::command]
pub async fn ftp_copy(
    conn_name: String,
    src_path: String,
    dst_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!(
        "[FTP] command ftp_copy: conn={} src={} dst={}",
        conn_name, src_path, dst_path
    );

    // Download to temp file, re-upload (no native server-side copy in FTP)
    let tmp = std::env::temp_dir().join(format!(
        "wind_ftp_copy_{}_{}",
        std::process::id(),
        src_path.rsplit('/').next().unwrap_or("file")
    ));

    // Download
    debug!("[FTP] copy: downloading to temp {}", tmp.display());
    let bytes_dl: u64;
    {
        let session = ensure_session(&state, &conn_name).await?;
        let mut ftp = session.lock().await;

        let mut stream = ftp
            .client
            .retr_as_stream(&src_path)
            .await
            .map_err(|e| format!("Failed to open download stream: {}", e))?;
        let mut file = tokio::fs::File::create(&tmp)
            .await
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        bytes_dl = tokio::io::copy(&mut stream, &mut file)
            .await
            .map_err(|e| format!("Failed to download: {}", e))?;
    }
    debug!("[FTP] copy: downloaded {} bytes", bytes_dl);

    // Upload
    debug!("[FTP] copy: uploading from temp");
    let bytes_ul: u64;
    {
        let session = ensure_session(&state, &conn_name).await?;
        let mut ftp = session.lock().await;

        let mut file = tokio::fs::File::open(&tmp)
            .await
            .map_err(|e| format!("Failed to read temp file: {}", e))?;
        bytes_ul = ftp
            .client
            .put_file(&dst_path, &mut file)
            .await
            .map_err(|e| format!("Failed to upload: {}", e))?;
    }
    debug!("[FTP] copy: uploaded {} bytes", bytes_ul);

    // Cleanup
    let _ = tokio::fs::remove_file(&tmp).await;
    info!("[FTP] copy: completed {} → {}", src_path, dst_path);
    Ok(())
}

#[tauri::command]
pub async fn ftp_create_file(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    info!(
        "[FTP] command ftp_create_file: conn={} path={}",
        conn_name, remote_path
    );

    let session = ensure_session(&state, conn_name).await?;

    let mut ftp = session.lock().await;
    // Upload empty content
    match ftp.client.put_with_stream(&remote_path).await {
        Ok(stream) => {
            drop(stream);
            info!("[FTP] create_file: created {}", remote_path);
        }
        Err(e) => {
            error!("[FTP] create_file: FAILED {} — {}", remote_path, e);
            return Err(format!("Failed to create file: {}", e));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn ftp_mkdir(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (conn_name, remote_path) = parse_ftp_url(&path)?;
    info!(
        "[FTP] command ftp_mkdir: conn={} path={}",
        conn_name, remote_path
    );

    let session = ensure_session(&state, conn_name).await?;

    let mut ftp = session.lock().await;
    // Accept 250/257/200 — Android FTP servers return 250 for MKD
    let mkd_ok = &[
        suppaftp::Status::RequestedFileActionOk,
        suppaftp::Status::PathCreated,
        suppaftp::Status::CommandOk,
    ];
    // Try absolute MKD first; if it fails, CWD to parent and use relative MKD
    // (Android FTP servers often reject absolute MKD on protected paths)
    match ftp.client.custom_command(format!("MKD {}", remote_path), mkd_ok).await {
        Ok(_) => info!("[FTP] mkdir: created {}", remote_path),
        Err(e) => {
            warn!("[FTP] mkdir absolute failed ({}), trying CWD + relative MKD", e);
            // Extract parent and dir name
            let (parent, dirname) = match remote_path.rfind('/') {
                Some(pos) if pos > 0 => (&remote_path[..pos], &remote_path[pos + 1..]),
                Some(_) => ("/", &remote_path[1..]),
                None => return Err(format!("Failed to create directory: {}", e)),
            };
            if dirname.is_empty() {
                return Err(format!("Failed to create directory: {}", e));
            }
            ftp.client.cwd(parent).await
                .map_err(|e2| format!("CWD '{}' failed: {}", parent, e2))?;
            match ftp.client.custom_command(format!("MKD {}", dirname), mkd_ok).await {
                Ok(_) => info!("[FTP] mkdir: created {} via relative path (parent={})", dirname, parent),
                Err(e2) => {
                    error!("[FTP] mkdir: FAILED {} (absolute: {}, relative: {})", remote_path, e, e2);
                    return Err(format!("Failed to create directory '{}': {}", remote_path, e2));
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn ftp_download_folder(
    conn_name: String,
    remote_path: String,
    local_path: String,
    move_mode: bool,
    skip_rel_paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("[FTP] command ftp_download_folder: conn={conn_name} remote={remote_path} → local={local_path} move={move_mode} skip={}", skip_rel_paths.len());
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    let (total_bytes, ids) = scheduler
        .enqueue_ftp_folder_download(
            sched.clone(),
            &conn_name,
            &remote_path,
            &local_path,
            move_mode,
            skip_rel_paths,
        )
        .await?;
    Ok(serde_json::json!({ "total_bytes": total_bytes, "task_ids": ids }).to_string())
}

#[tauri::command]
pub async fn ftp_upload_folder(
    conn_name: String,
    local_path: String,
    remote_path: String,
    move_mode: bool,
    skip_rel_paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("[FTP] command ftp_upload_folder: local={local_path} → conn={conn_name} remote={remote_path} move={move_mode} skip={}", skip_rel_paths.len());
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    let (total_bytes, ids) = scheduler
        .enqueue_ftp_folder_upload(
            sched.clone(),
            &conn_name,
            &local_path,
            &remote_path,
            move_mode,
            skip_rel_paths,
        )
        .await?;
    Ok(serde_json::json!({ "total_bytes": total_bytes, "task_ids": ids }).to_string())
}

#[tauri::command]
pub async fn ftp_copy_folder(
    conn_name: String,
    source_dir: String,
    destination_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<u64>, String> {
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    scheduler.enqueue_ftp_folder_copy(sched.clone(), &conn_name, &source_dir, &destination_dir).await
}

#[tauri::command]
pub async fn list_ftp_connections(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::ftp::FtpConnectionConfig>, String> {
    let mgr = state.ftp_manager.lock().await;
    let configs = mgr.get_configs();
    info!(
        "[FTP] command list_ftp_connections: {} connection(s)",
        configs.len()
    );
    Ok(configs)
}

#[tauri::command]
pub async fn get_ftp_connection_for_edit(
    name: String,
) -> Result<crate::ftp::FtpConnectionSummary, String> {
    tokio::task::spawn_blocking(move || crate::ftp::FtpManager::read_config_summary(&name))
        .await
        .map_err(|e| format!("Failed to load connection: {}", e))?
}

#[tauri::command]
pub async fn update_ftp_connection(
    name: String,
    host: String,
    port: u16,
    user: String,
    password: Option<String>,
    reconnect: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let (config, epoch) = {
        let mut mgr = state.ftp_manager.lock().await;
        mgr.update_config(&name, &host, port, &user, password.as_deref())?;
        if !reconnect { return Ok(()); }
        (mgr.get_config(&name).ok_or_else(|| format!("Connection '{}' not found", name))?, mgr.epoch(&name))
    };
    let session = crate::ftp::FtpManager::open_session(&config).await?;
    let mut mgr = state.ftp_manager.lock().await;
    mgr.install_session_if_config_matches(&config, epoch, session)?;
    Ok(())
}

#[tauri::command]
pub async fn check_ftp_connection(name: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    info!("[FTP] command check_ftp_connection: name={}", name);
    let session = ensure_session(&state, &name).await?;
    if session.lock().await.client.noop().await.is_err() { replace_session(&state, &name).await?; }
    Ok(true)
}
