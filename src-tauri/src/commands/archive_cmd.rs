use tauri::Emitter;

use crate::FileEntry;
use crate::AppState;
use crate::transfer::TransferScheduler;

#[tauri::command]
pub async fn read_archive_directory(
    archive_path: String,
    internal_path: String,
    password: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        crate::archive::list_entries(&archive_path, &internal_path, password)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn read_archive_file(
    archive_path: String,
    internal_path: String,
    password: Option<String>,
) -> Result<Vec<u8>, String> {
    tokio::task::spawn_blocking(move || {
        crate::archive::read_file_bytes(&archive_path, &internal_path, password)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub fn extract_archive_files(
    app: tauri::AppHandle,
    archive_path: String,
    internal_paths: Vec<String>,
    dest_dir: String,
    password: Option<String>,
) -> Result<(), String> {
    crate::archive::extract_files(&archive_path, &internal_paths, &dest_dir, password, &crate::archive::no_progress())?;
    let _ = app.emit("directory-changed", vec![dest_dir]);
    Ok(())
}

#[tauri::command]
pub async fn extract_archive(
    app: tauri::AppHandle,
    archive_path: String,
    dest_dir: String,
    password: Option<String>,
    skip_paths: Option<Vec<String>>,
) -> Result<(), String> {
    let archive_path_clone = archive_path.clone();
    let dest_dir_clone = dest_dir.clone();
    let password_clone = password.clone();
    let skip_set: Option<std::collections::HashSet<String>> =
        skip_paths.map(|v| v.into_iter().collect());
    let total_bytes = tokio::task::spawn_blocking(move || {
        crate::archive::extract_all(&archive_path_clone, &dest_dir_clone, password_clone, skip_set.as_ref(), &crate::archive::no_progress())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("{}", e))?;

    let archive_name = std::path::Path::new(&archive_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| archive_path.clone());
    app.emit(
        "extract-complete",
        serde_json::json!({ "archive": archive_name, "dest": dest_dir, "total_bytes": total_bytes }),
    )
    .ok();
    let _ = app.emit("directory-changed", vec![dest_dir]);
    Ok(())
}

#[tauri::command]
pub async fn compress_files(
    app: tauri::AppHandle,
    sources: Vec<String>,
    dest_path: String,
) -> Result<(), String> {
    let dest_path_clone = dest_path.clone();
    let total_bytes = tokio::task::spawn_blocking(move || {
        crate::archive::compress_files(&sources, &dest_path_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("{}", e))?;

    let archive_name = std::path::Path::new(&dest_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| dest_path.clone());
    app.emit(
        "compress-complete",
        serde_json::json!({ "archive": archive_name, "path": dest_path, "total_bytes": total_bytes }),
    )
    .ok();
    Ok(())
}

#[tauri::command]
pub fn archive_delete_entry(
    archive_path: String,
    internal_paths: Vec<String>,
) -> Result<(), String> {
    crate::archive::delete_entries(&archive_path, &internal_paths)
}

#[tauri::command]
pub fn archive_rename_entry(
    archive_path: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    crate::archive::rename_entry(&archive_path, &old_path, &new_path)
}

#[tauri::command]
pub fn archive_add_files(
    archive_path: String,
    source_paths: Vec<String>,
    internal_path: String,
) -> Result<(), String> {
    crate::archive::add_files(&archive_path, &source_paths, &internal_path)
}

#[tauri::command]
pub fn archive_write_file(
    archive_path: String,
    internal_path: String,
    content: Vec<u8>,
) -> Result<(), String> {
    crate::archive::write_file_bytes(&archive_path, &internal_path, &content)
}

#[tauri::command]
pub fn archive_create_entry(
    archive_path: String,
    internal_path: String,
    is_dir: bool,
) -> Result<(), String> {
    crate::archive::create_entry(&archive_path, &internal_path, is_dir)
}

/// Enqueue an archive extraction task through the TransferScheduler.
/// Returns the task ID immediately. The extraction runs in the background
/// with progress reported via transfer-progress events.
#[tauri::command]
pub async fn extract_enqueue(
    state: tauri::State<'_, AppState>,
    archive_path: String,
    dest_dir: String,
    password: Option<String>,
    internal_paths: Option<Vec<String>>,
    skip_paths: Option<Vec<String>>,
) -> Result<u64, String> {
    let sched = state.transfer_scheduler.clone();
    let id = TransferScheduler::enqueue_extract(
        sched,
        archive_path,
        dest_dir,
        password,
        internal_paths,
        skip_paths,
    )
    .await;
    Ok(id)
}
