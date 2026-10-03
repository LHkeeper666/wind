use tauri::Emitter;

use crate::AppState;

#[tauri::command]
pub async fn transfer_enqueue(
    tasks: Vec<crate::transfer::EnqueueTask>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<u64>, String> {
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    Ok(scheduler.enqueue(tasks, sched.clone()))
}

#[tauri::command]
pub async fn transfer_cancel(id: u64, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let sched = state.transfer_scheduler.clone();
    let mut scheduler = sched.lock().await;
    Ok(scheduler.cancel(id, sched.clone()))
}

#[tauri::command]
pub async fn transfer_cancel_all(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    Ok(scheduler.cancel_all())
}

#[tauri::command]
pub async fn scan_transfer_conflicts(
    tasks: Vec<crate::transfer::EnqueueTask>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        for task in &tasks {
            if task.op_type != crate::transfer::TransferType::Copy
                && task.op_type != crate::transfer::TransferType::Move
            {
                continue;
            }
            let src = std::path::Path::new(&task.source);
            let dst = std::path::Path::new(&task.destination);
            if src.is_dir() {
                crate::transfer::scan_dir_conflicts(src, dst, src, &app, &task.skip_rel_paths);
            }
        }
        let _ = app.emit("transfer-conflict-scan-done", serde_json::json!({}));
    });
    Ok(())
}

#[tauri::command]
pub async fn scan_ftp_upload_conflicts(
    conn_name: String,
    sources: Vec<String>,
    remote_dir: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.ftp_manager.lock().await;
    let session = mgr
        .create_independent(&conn_name)
        .await
        .map_err(|e| format!("Failed to create FTP session for conflict scan: {e}"))?;
    drop(mgr);

    let mut ftp = session.lock().await;
    let remote_entries = crate::ftp::list_dir_recursive(&mut ftp.client, &remote_dir)
        .await
        .map_err(|e| format!("Failed to list remote directory: {e}"))?;
    let _ = ftp.client.quit().await;
    drop(ftp);

    use std::collections::HashSet;
    let remote_base = remote_dir.trim_end_matches('/').to_string();
    let mut remote_names: HashSet<String> = HashSet::new();
    for (remote_full, _size, _is_dir) in &remote_entries {
        let rel = remote_full
            .strip_prefix(&remote_base)
            .unwrap_or(remote_full)
            .trim_start_matches('/');
        remote_names.insert(rel.to_string());
    }

    tokio::task::spawn_blocking(move || {
        for src in &sources {
            let path = std::path::Path::new(src);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if !path.is_dir() {
                if remote_names.contains(&name) {
                    let _ = app.emit(
                        "transfer-conflict-found",
                        serde_json::json!({
                            "kind": "file",
                            "dir_source": "",
                            "rel_path": name,
                            "source": src,
                            "destination": format!("ftp://{conn_name}{remote_base}/{name}"),
                        }),
                    );
                }
            } else {
                let mut local: Vec<(String, u64, bool)> = Vec::new();
                let _ = crate::transfer::walk_local_dir(&path.to_path_buf(), path, &mut local);
                for (full, _size, is_dir) in &local {
                    if *is_dir {
                        continue;
                    }
                    let rel = full
                        .strip_prefix(&path.to_string_lossy().to_string())
                        .map(|p| {
                            p.trim_start_matches('\\')
                                .trim_start_matches('/')
                                .to_string()
                        })
                        .unwrap_or_default();
                    if rel.is_empty() {
                        continue;
                    }
                    let remote_rel = format!("{}/{}", name, rel.replace('\\', "/"));
                    if remote_names.contains(&remote_rel) {
                        let _ = app.emit("transfer-conflict-found", serde_json::json!({
                            "kind": "dir",
                            "dir_source": src,
                            "rel_path": rel.replace('\\', "/"),
                            "source": full,
                            "destination": format!("ftp://{conn_name}{remote_base}/{remote_rel}"),
                        }));
                    }
                }
            }
        }
        let _ = app.emit("transfer-conflict-scan-done", serde_json::json!({}));
    });
    Ok(())
}

#[tauri::command]
pub async fn scan_ftp_download_conflicts(
    conn_name: String,
    files: Vec<String>,
    dirs: Vec<String>,
    local_dir: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.ftp_manager.lock().await;
    let session = mgr
        .create_independent(&conn_name)
        .await
        .map_err(|e| format!("Failed to create FTP session for conflict scan: {e}"))?;
    drop(mgr);

    let mut ftp = session.lock().await;
    let mut dir_trees: Vec<(String, Vec<(String, u64, bool)>)> = Vec::new();
    for dir in &dirs {
        let remote_path = crate::commands::ftp_cmd::parse_ftp_url(dir)
            .map(|(_, p)| p.to_string())
            .unwrap_or_default();
        let entries = crate::ftp::list_dir_recursive(&mut ftp.client, &remote_path)
            .await
            .map_err(|e| format!("Failed to list remote directory {dir}: {e}"))?;
        dir_trees.push((dir.clone(), entries));
    }
    let _ = ftp.client.quit().await;
    drop(ftp);

    tokio::task::spawn_blocking(move || {
        for f in &files {
            let name = f.rsplit('/').next().unwrap_or(f.as_str());
            let local = std::path::Path::new(&local_dir).join(name);
            if local.exists() {
                let _ = app.emit(
                    "transfer-conflict-found",
                    serde_json::json!({
                        "kind": "file",
                        "dir_source": "",
                        "rel_path": name,
                        "source": f,
                        "destination": local.to_string_lossy(),
                    }),
                );
            }
        }
        for (dir, entries) in &dir_trees {
            let dir_name = dir.rsplit('/').next().unwrap_or(dir.as_str());
            let remote_base = crate::commands::ftp_cmd::parse_ftp_url(dir)
                .map(|(_, p)| p.to_string())
                .unwrap_or_default();
            for (remote_full, _size, is_dir) in entries {
                if *is_dir {
                    continue;
                }
                let rel = remote_full
                    .strip_prefix(&remote_base)
                    .unwrap_or(remote_full)
                    .trim_start_matches('/');
                let local = std::path::Path::new(&local_dir)
                    .join(dir_name)
                    .join(rel.replace('/', "\\"));
                if local.exists() {
                    let _ = app.emit(
                        "transfer-conflict-found",
                        serde_json::json!({
                            "kind": "dir",
                            "dir_source": dir,
                            "rel_path": rel,
                            "source": remote_full,
                            "destination": local.to_string_lossy(),
                        }),
                    );
                }
            }
        }
        let _ = app.emit("transfer-conflict-scan-done", serde_json::json!({}));
    });
    Ok(())
}

#[tauri::command]
pub async fn transfer_reorder(ids: Vec<u64>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.reorder(ids);
    Ok(())
}

#[tauri::command]
pub async fn transfer_get_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::transfer::TransferHistoryRecord>, String> {
    let scheduler = state.transfer_scheduler.lock().await;
    Ok(scheduler.get_history())
}

#[tauri::command]
pub async fn transfer_clear_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.clear_history();
    Ok(())
}

#[tauri::command]
pub async fn transfer_set_ftp_slots(n: usize, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.set_ftp_max_slots(n);
    Ok(())
}

#[tauri::command]
pub async fn transfer_set_local_slots(n: usize, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.set_local_max_slots(n);
    Ok(())
}

#[tauri::command]
pub async fn transfer_set_extract_slots(n: usize, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut scheduler = state.transfer_scheduler.lock().await;
    scheduler.set_extract_max_slots(n);
    Ok(())
}

#[tauri::command]
pub async fn transfer_get_slots(state: tauri::State<'_, AppState>) -> Result<(usize, usize, usize), String> {
    let scheduler = state.transfer_scheduler.lock().await;
    Ok(scheduler.get_slot_config())
}
