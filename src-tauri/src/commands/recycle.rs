use serde::Serialize;

#[derive(Serialize)]
pub struct TrashItemDto {
    pub id: String,
    pub name: String,
    pub original_path: String,
    pub date_deleted: i64,
    pub size: Option<u64>,
}

#[tauri::command]
pub fn list_recycle_bin() -> Result<Vec<TrashItemDto>, String> {
    use trash::os_limited;
    let items = os_limited::list().map_err(|e| format!("Failed to list recycle bin: {}", e))?;
    let mut result = Vec::with_capacity(items.len());
    for item in &items {
        let size = os_limited::metadata(item)
            .ok()
            .and_then(|m| m.size.size());
        result.push(TrashItemDto {
            id: item.id.to_string_lossy().to_string(),
            name: item.name.clone(),
            original_path: item.original_path().to_string_lossy().to_string(),
            date_deleted: item.time_deleted,
            size,
        });
    }
    Ok(result)
}

#[tauri::command]
pub fn restore_recycle_items(ids: Vec<String>) -> Result<(), String> {
    use trash::os_limited;
    let all_items = os_limited::list().map_err(|e| format!("Failed to list recycle bin: {}", e))?;
    let to_restore: Vec<_> = all_items
        .into_iter()
        .filter(|item| ids.contains(&item.id.to_string_lossy().to_string()))
        .collect();
    if to_restore.is_empty() {
        return Err("No matching items found in recycle bin".to_string());
    }
    os_limited::restore_all(to_restore).map_err(|e| format!("Failed to restore: {}", e))
}

#[tauri::command]
pub fn purge_recycle_items(ids: Vec<String>) -> Result<(), String> {
    use trash::os_limited;
    let all_items = os_limited::list().map_err(|e| format!("Failed to list recycle bin: {}", e))?;
    let to_purge: Vec<_> = all_items
        .into_iter()
        .filter(|item| ids.contains(&item.id.to_string_lossy().to_string()))
        .collect();
    if to_purge.is_empty() {
        return Err("No matching items found in recycle bin".to_string());
    }
    os_limited::purge_all(&to_purge).map_err(|e| format!("Failed to purge: {}", e))
}

#[tauri::command]
pub fn empty_recycle_bin() -> Result<(), String> {
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Shell::SHEmptyRecycleBinW;
        let flags = 0x0001u32; // SHERB_NOCONFIRMATION
        SHEmptyRecycleBinW(Some(HWND::default()), None, flags)
            .map_err(|e| format!("Failed to empty recycle bin: {}", e))?;
    }
    Ok(())
}
