use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as TokioMutex;

use log::{info, error};

use crate::app_paths;
use tokio::task::JoinHandle;

use crate::ftp::FtpManager;

use super::{EnqueueTask, TransferTask, TransferType, TaskStatus, TransferHistoryRecord};
use super::helpers::{now_secs, dir_size, extract_ftp_conn};
use super::conflict::walk_local_dir;
use super::ftp::ensure_remote_dir;
use super::execute_transfer;

struct ActiveTask {
    handle: JoinHandle<()>,
    cancel_flag: Arc<AtomicBool>,
    op_type: TransferType,
    source: String,
    dest: String,
}

pub struct TransferScheduler {
    pub(super) app: AppHandle,
    pub(super) ftp_manager: Arc<TokioMutex<FtpManager>>,
    queue: VecDeque<TransferTask>,
    active: HashMap<u64, ActiveTask>,
    next_id: AtomicU64,
    next_batch_id: AtomicU64,
    history: Vec<TransferHistoryRecord>,
    history_path: PathBuf,

    ftp_conn_slots: HashMap<String, usize>,
    ftp_max_per_conn: usize,
    local_slots_used: usize,
    local_max_slots: usize,
    extract_slots_used: usize,
    extract_max_slots: usize,
}

impl TransferScheduler {
    pub fn new(app: AppHandle, ftp_manager: Arc<TokioMutex<FtpManager>>) -> Self {
        let history_path = app_paths::state_file("transfer-history.json");
        let legacy_history_path = app_paths::legacy_roaming_file("transfer-history.json");
        if let Err(error) = app_paths::migrate_legacy_file(&history_path, &legacy_history_path) {
            error!("[transfer] Failed to migrate history: {}", error);
        }

        let mut scheduler = TransferScheduler {
            app,
            ftp_manager,
            queue: VecDeque::new(),
            active: HashMap::new(),
            next_id: AtomicU64::new(1),
            next_batch_id: AtomicU64::new(1),
            history: Vec::new(),
            history_path,
            ftp_conn_slots: HashMap::new(),
            ftp_max_per_conn: 2, // Each transfer uses create_independent() for its own TCP connection
            local_slots_used: 0,
            local_max_slots: 2,
            extract_slots_used: 0,
            extract_max_slots: 2,
        };

        scheduler.load_history();
        scheduler
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn next_batch_id(&self) -> u64 {
        self.next_batch_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn enqueue(&mut self, tasks: Vec<EnqueueTask>, sched: Arc<TokioMutex<TransferScheduler>>) -> Vec<u64> {
        info!("[transfer] enqueue: {} task(s)", tasks.len());
        let batch_id = self.next_batch_id();
        let mut ids = Vec::new();

        for task in tasks {
            let id = self.next_id();
            ids.push(id);
            // Look up file size for local files if not provided
            let total_bytes = if task.total_bytes > 0 {
                task.total_bytes
            } else {
                match task.op_type {
                    TransferType::FtpDownload => 0, // source is FTP URL, can't stat
                    TransferType::FtpUpload => {
                        // source is local path, can look up
                        let path = Path::new(&task.source);
                        path.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                    _ => {
                        let path = Path::new(&task.source);
                        if path.is_file() {
                            path.metadata().map(|m| m.len()).unwrap_or(0)
                        } else if path.is_dir() {
                            dir_size(path)
                        } else {
                            0
                        }
                    }
                }
            };
            let entry = TransferTask {
                id,
                batch_id,
                op_type: task.op_type,
                status: TaskStatus::Queued,
                source: task.source.clone(),
                destination: task.destination.clone(),
                total_bytes,
                bytes_done: 0,
                conn_name: task.conn_name.clone(),
                cancel_flag: None,
                skip_rel_paths: task.skip_rel_paths.clone(),
                permanent: task.permanent,
                password: task.password.clone(),
                internal_paths: task.internal_paths.clone(),
            };
            // Emit queued event so frontend creates the entry immediately
            let _ = self.app.emit("transfer-progress", serde_json::json!({
                "id": entry.id,
                "batch_id": entry.batch_id,
                "op_type": entry.op_type,
                "status": "queued",
                "bytes_done": 0,
                "total_bytes": entry.total_bytes,
                "speed_bps": 0,
                "source": entry.source,
                "destination": entry.destination,
            }));
            self.queue.push_back(entry);
        }

        self.dispatch_pending(sched);
        ids
    }

    /// Enqueue an extraction task. Emits queued event immediately with total_bytes=0,
    /// then spawns a background scan to compute actual total_bytes and dispatch.
    pub async fn enqueue_extract(
        sched: Arc<TokioMutex<TransferScheduler>>,
        archive_path: String,
        dest_dir: String,
        password: Option<String>,
        internal_paths: Option<Vec<String>>,
        skip_paths: Option<Vec<String>>,
    ) -> u64 {
        // Phase 1: create task with total_bytes=0 and emit queued event
        let id;
        let batch_id;
        let app;
        {
            let mut s = sched.lock().await;
            id = s.next_id();
            batch_id = s.next_batch_id();
            app = s.app.clone();

            let entry = TransferTask {
                id,
                batch_id,
                op_type: TransferType::Extract,
                status: TaskStatus::Queued,
                source: archive_path.clone(),
                destination: dest_dir.clone(),
                total_bytes: 0,
                bytes_done: 0,
                conn_name: None,
                cancel_flag: None,
                skip_rel_paths: skip_paths.clone().unwrap_or_default(),
                permanent: false,
                password: password.clone(),
                internal_paths: internal_paths.clone(),
            };

            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": entry.id,
                "batch_id": entry.batch_id,
                "op_type": entry.op_type,
                "status": "queued",
                "bytes_done": 0,
                "total_bytes": 0u64,
                "speed_bps": 0,
                "source": entry.source,
                "destination": entry.destination,
            }));

            s.queue.push_back(entry);
        }

        // Phase 2: spawn blocking scan for total_bytes
        let sched_clone = sched.clone();
        let app_clone = app.clone();
        tokio::spawn(async move {
            let scan_result = tokio::task::spawn_blocking({
                let archive_path = archive_path.clone();
                let password = password.clone();
                let internal_paths = internal_paths.clone();
                move || -> Result<u64, String> {
                    use crate::archive;
                    let total = if let Some(ref paths) = internal_paths {
                        archive::files_uncompressed_size(&archive_path, paths, password)?
                    } else {
                        archive::total_uncompressed_size(&archive_path, password)?
                    };
                    info!("[transfer] extract scan: total_bytes={}", total);
                    Ok(total)
                }
            }).await;

            let total_bytes = scan_result.unwrap_or_else(|e| {
                log::warn!("[transfer] extract scan join error: {}", e);
                Ok(0u64)
            }).unwrap_or_else(|e| {
                log::warn!("[transfer] extract scan error: {}", e);
                0
            });

            // Update task total_bytes and emit updated progress
            let mut s = sched_clone.lock().await;
            if let Some(task) = s.queue.iter_mut().find(|t| t.id == id) {
                task.total_bytes = total_bytes;
            }
            // Also check if it's already been dispatched to active
            let _ = app_clone.emit("transfer-progress", serde_json::json!({
                "id": id,
                "batch_id": batch_id,
                "op_type": "extract",
                "status": "queued",
                "bytes_done": 0,
                "total_bytes": total_bytes,
                "speed_bps": 0,
                "source": archive_path,
                "destination": dest_dir,
            }));

            // Try to dispatch — the task might be ready for a slot now
            s.dispatch_pending(sched_clone.clone());
        });

        id
    }

    pub fn cancel(&mut self, id: u64, sched: Arc<TokioMutex<TransferScheduler>>) -> bool {
        // Check queue first
        if let Some(i) = self.queue.iter().position(|t| t.id == id) {
            let task = self.queue.remove(i).unwrap();
            let op_type = task.op_type;
            let source = task.source.clone();
            let dest = task.destination.clone();
            self.emit_cancelled(task.id, task.batch_id, &source, &dest, op_type);
            self.save_to_history(&TransferHistoryRecord {
                id: task.id, batch_id: task.batch_id, transfer_type: op_type,
                source, destination: dest,
                total_bytes: task.total_bytes, bytes_transferred: 0,
                status: TaskStatus::Cancelled, error_message: None,
                started_at: 0, completed_at: now_secs(), avg_speed_bps: None,
            });
            self.dispatch_pending(sched);
            return true;
        }

        // Check active — set cancel flag, let the task complete gracefully.
        // The task's callback handles cleanup: removing from active, freeing slot, dispatching next.
        if let Some(active) = self.active.get(&id) {
            active.cancel_flag.store(true, Ordering::Relaxed);
            // Emit cancelled immediately so the UI updates
            self.emit_cancelled(id, 0, &active.source, &active.dest, active.op_type);
            return true;
        }

        false
    }

    pub fn cancel_all(&mut self) -> usize {
        let mut cancelled_ids: Vec<u64> = Vec::new();

        // Cancel queued tasks directly — they never took a slot, so no
        // free_slot_direct and no dispatch (which would otherwise promote
        // the next queued task into running just to be cancelled).
        while let Some(task) = self.queue.pop_front() {
            let op_type = task.op_type;
            let source = task.source.clone();
            let dest = task.destination.clone();
            cancelled_ids.push(task.id);
            self.save_to_history(&TransferHistoryRecord {
                id: task.id, batch_id: task.batch_id, transfer_type: op_type,
                source, destination: dest,
                total_bytes: task.total_bytes, bytes_transferred: 0,
                status: TaskStatus::Cancelled, error_message: None,
                started_at: 0, completed_at: now_secs(), avg_speed_bps: None,
            });
        }

        // Cancel active tasks by setting their flag; workers clean up their own slots.
        for (id, active) in &self.active {
            active.cancel_flag.store(true, Ordering::Relaxed);
            cancelled_ids.push(*id);
        }

        // Emit a single batch event so the frontend re-renders once.
        let _ = self.app.emit("transfer-cancelled-batch", serde_json::json!({
            "ids": cancelled_ids,
        }));

        cancelled_ids.len()
    }

    pub fn reorder(&mut self, ids: Vec<u64>) {
        let mut new_order: VecDeque<TransferTask> = VecDeque::new();
        let mut remaining: Vec<TransferTask> = self.queue.drain(..).collect();

        for &id in &ids {
            if let Some(pos) = remaining.iter().position(|t| t.id == id) {
                new_order.push_back(remaining.remove(pos));
            }
        }
        // Append any remaining queued tasks not in the reorder list
        new_order.extend(remaining);

        self.queue = new_order;
        let queued_ids: Vec<u64> = self.queue.iter().map(|t| t.id).collect();
        let _ = self.app.emit("transfer-queue-updated", serde_json::json!({ "ids": queued_ids }));
    }

    fn dispatch_pending(&mut self, sched: Arc<TokioMutex<TransferScheduler>>) {
        self.cleanup_finished();
        self.dispatch_queued(sched);
    }

    /// Dispatch queued tasks without first cleaning up (caller ensures cleanup).
    fn dispatch_queued(&mut self, sched: Arc<TokioMutex<TransferScheduler>>) {
        while !self.queue.is_empty() {
            let task = self.queue.front().unwrap();
            if !self.can_take_slot(task) {
                break;
            }

            let mut task = self.queue.pop_front().unwrap();
            self.take_slot(&task);
            task.status = TaskStatus::Running;
            task.cancel_flag = Some(Arc::new(AtomicBool::new(false)));
            info!("[transfer] dispatching task {}: total_bytes={}", task.id, task.total_bytes);
            self.emit_progress(&task, 0);

            let app = self.app.clone();
            let ftp_mgr = self.ftp_manager.clone();
            let cancel_flag = task.cancel_flag.clone().unwrap();
            let cancel_flag2 = cancel_flag.clone();
            let task_clone = task.clone();
            let op_type = task.op_type;
            let source = task.source.clone();
            let source2 = source.clone();
            let dest = task.destination.clone();
            let dest2 = dest.clone();
            let id = task.id;
            let sched_clone = sched.clone();

            let handle = tokio::spawn(async move {
                let _result = execute_transfer(&task_clone, &cancel_flag, &app, &ftp_mgr).await;
                let mut s = sched_clone.lock().await;
                s.active.remove(&id);
                s.free_slot_direct(op_type, &source2, &dest2);
                s.cleanup_finished();
                s.dispatch_queued(sched_clone.clone());
            });

            self.active.insert(id, ActiveTask {
                handle,
                cancel_flag: cancel_flag2,
                op_type,
                source,
                dest,
            });
        }
    }

    fn can_take_slot(&self, task: &TransferTask) -> bool {
        match task.op_type {
            TransferType::FtpDownload | TransferType::FtpUpload => {
                let conn = extract_ftp_conn(&task.source).or_else(|| extract_ftp_conn(&task.destination));
                if let Some(conn) = conn {
                    let used = self.ftp_conn_slots.get(&conn).copied().unwrap_or(0);
                    used < self.ftp_max_per_conn
                } else {
                    false
                }
            }
            TransferType::Copy | TransferType::Move | TransferType::Delete => {
                if self.local_slots_used >= self.local_max_slots {
                    return false;
                }
                // Same-drive copies are serial
                if task.op_type == TransferType::Copy || task.op_type == TransferType::Move {
                    let src_drive = task.source.chars().next().map(|c| c.to_ascii_uppercase());
                    let dst_drive = task.destination.chars().next().map(|c| c.to_ascii_uppercase());
                    if src_drive == dst_drive && self.local_slots_used > 0 {
                        return false;
                    }
                }
                true
            }
            TransferType::Extract => {
                self.extract_slots_used < self.extract_max_slots
            }
        }
    }

    fn take_slot(&mut self, task: &TransferTask) {
        match task.op_type {
            TransferType::FtpDownload | TransferType::FtpUpload => {
                let conn = extract_ftp_conn(&task.source)
                    .or_else(|| extract_ftp_conn(&task.destination))
                    .unwrap_or_default();
                *self.ftp_conn_slots.entry(conn).or_insert(0) += 1;
            }
            TransferType::Extract => {
                self.extract_slots_used += 1;
            }
            _ => {
                self.local_slots_used += 1;
            }
        }
    }

    fn free_slot_direct(&mut self, op_type: TransferType, source: &str, dest: &str) {
        match op_type {
            TransferType::FtpDownload | TransferType::FtpUpload => {
                let conn = extract_ftp_conn(source)
                    .or_else(|| extract_ftp_conn(dest))
                    .unwrap_or_default();
                if let Some(count) = self.ftp_conn_slots.get_mut(&conn) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.ftp_conn_slots.remove(&conn);
                    }
                }
            }
            TransferType::Extract => {
                self.extract_slots_used = self.extract_slots_used.saturating_sub(1);
            }
            _ => {
                self.local_slots_used = self.local_slots_used.saturating_sub(1);
            }
        }
    }

    fn emit_progress(&self, task: &TransferTask, speed_bps: u64) {
        let _ = self.app.emit("transfer-progress", serde_json::json!({
            "id": task.id,
            "batch_id": task.batch_id,
            "op_type": task.op_type,
            "status": task.status,
            "bytes_done": task.bytes_done,
            "total_bytes": task.total_bytes,
            "speed_bps": speed_bps,
            "source": task.source,
            "destination": task.destination,
        }));
    }

    fn emit_cancelled(&self, id: u64, batch_id: u64, source: &str, dest: &str,
                      op_type: TransferType) {
        let _ = self.app.emit("transfer-cancelled", serde_json::json!({
            "id": id, "batch_id": batch_id, "op_type": op_type,
            "source": source, "destination": dest,
        }));
    }

    // ── History ──

    fn load_history(&mut self) {
        if !self.history_path.exists() {
            return;
        }
        if let Ok(content) = fs::read_to_string(&self.history_path) {
            if let Ok(records) = serde_json::from_str::<Vec<TransferHistoryRecord>>(&content) {
                self.history = records;
            }
        }
    }

    fn save_to_history(&mut self, record: &TransferHistoryRecord) {
        // Update existing or push new
        if let Some(existing) = self.history.iter_mut().find(|r| r.id == record.id) {
            *existing = record.clone();
        } else {
            self.history.push(record.clone());
        }
        self.cleanup_history();
        self.persist_history();
    }

    fn cleanup_history(&mut self) {
        let now = now_secs();
        let seven_days = 7 * 86400;
        let three_days = 3 * 86400;

        self.history.retain(|r| {
            let age = now.saturating_sub(r.completed_at);
            match r.status {
                TaskStatus::Failed => age <= three_days,
                _ => age <= seven_days,
            }
        });

        while self.history.len() > 500 {
            self.history.remove(0);
        }
    }

    fn persist_history(&self) {
        if let Some(parent) = self.history_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.history) {
            let _ = fs::write(&self.history_path, json);
        }
    }

    pub fn get_history(&self) -> Vec<TransferHistoryRecord> {
        self.history.clone()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.persist_history();
    }

    pub fn set_ftp_max_slots(&mut self, n: usize) {
        self.ftp_max_per_conn = n.max(1).min(8);
    }

    pub fn set_local_max_slots(&mut self, n: usize) {
        self.local_max_slots = n.max(1).min(8);
    }

    pub fn set_extract_max_slots(&mut self, n: usize) {
        self.extract_max_slots = n.max(1).min(8);
    }

    pub fn get_slot_config(&self) -> (usize, usize, usize) {
        (self.ftp_max_per_conn, self.local_max_slots, self.extract_max_slots)
    }

    // ── FTP folder transfer ──

    pub async fn enqueue_ftp_folder_download(
        &mut self,
        sched: Arc<TokioMutex<TransferScheduler>>,
        conn_name: &str,
        remote_dir_path: &str,
        local_target_dir: &str,
        move_mode: bool,
        skip_rel_paths: Vec<String>,
    ) -> Result<(u64, Vec<u64>), String> {
        info!("[transfer] enqueue_ftp_folder_download: conn={conn_name} remote={remote_dir_path} → local={local_target_dir}");

        // 1. Get independent FTP session and list recursively
        let mgr = self.ftp_manager.lock().await;
        let session = mgr.create_independent(conn_name).await
            .map_err(|e| format!("Failed to create FTP session for folder download: {e}"))?;
        drop(mgr);

        let mut ftp = session.lock().await;
        let entries = crate::ftp::list_dir_recursive(&mut ftp.client, remote_dir_path).await
            .map_err(|e| format!("Failed to list remote directory tree: {e}"))?;
        // Close the independent session — each file task will create its own
        let _ = ftp.client.quit().await;
        drop(ftp);

        let entry_count = entries.len();
        info!("[transfer] list_dir_recursive returned {entry_count} entries for {remote_dir_path}");

        // 2. Separate dirs and files; pre-create local directories
        let remote_base = remote_dir_path.trim_end_matches('/');
        let local_base = Path::new(local_target_dir);

        let skip_set: std::collections::HashSet<String> = skip_rel_paths.iter().cloned().collect();
        let mut total_bytes: u64 = 0;
        let mut file_paths: Vec<(String, String, u64)> = Vec::new(); // (remote_path, local_path, size)

        for (remote_full, size, is_dir) in &entries {
            let relative = remote_full.strip_prefix(remote_base)
                .unwrap_or(remote_full)
                .trim_start_matches('/');
            let local_full = local_base.join(relative);

            if *is_dir {
                fs::create_dir_all(&local_full)
                    .map_err(|e| format!("Failed to create local directory '{}': {}", local_full.display(), e))?;
            } else if !skip_set.contains(relative) {
                total_bytes += size;
                file_paths.push((remote_full.clone(), local_full.to_string_lossy().to_string(), *size));
            }
        }

        // Ensure the root target directory exists (handles empty folder case)
        fs::create_dir_all(local_base)
            .map_err(|e| format!("Failed to create target directory '{}': {}", local_base.display(), e))?;

        if file_paths.is_empty() {
            // No files to transfer — just created directory structure
            return Ok((0, Vec::new()));
        }

        // 3. Build and enqueue tasks
        let tasks: Vec<EnqueueTask> = file_paths.iter().map(|(remote, local, size)| {
            EnqueueTask {
                op_type: TransferType::FtpDownload,
                source: format!("ftp://{conn_name}{}", remote),
                destination: local.clone(),
                total_bytes: *size,
                conn_name: Some(conn_name.to_string()),
                skip_rel_paths: Vec::new(),
                permanent: false,
                password: None,
                internal_paths: None,
            }
        }).collect();

        let batch_id = self.next_batch_id();
        let ids = self.enqueue(tasks, sched.clone());

        // Save move_mode for later cleanup (stored in history)
        if move_mode {
            self.save_batch_meta(batch_id, conn_name, remote_dir_path);
        }

        Ok((total_bytes, ids))
    }

    pub async fn enqueue_ftp_folder_upload(
        &mut self,
        sched: Arc<TokioMutex<TransferScheduler>>,
        conn_name: &str,
        local_dir_path: &str,
        remote_target_dir: &str,
        move_mode: bool,
        skip_rel_paths: Vec<String>,
    ) -> Result<(u64, Vec<u64>), String> {
        info!("[transfer] enqueue_ftp_folder_upload: local={local_dir_path} → conn={conn_name} remote={remote_target_dir}");

        let local_base = Path::new(local_dir_path);
        if !local_base.is_dir() {
            return Err(format!("Not a directory: {}", local_dir_path));
        }

        // 1. Recursively scan local directory
        let mut entries: Vec<(String, u64, bool)> = Vec::new(); // (full_path, size, is_dir)
        walk_local_dir(&local_base.to_path_buf(), local_base, &mut entries)
            .map_err(|e| format!("Failed to scan local directory: {e}"))?;

        info!("[transfer] local walk returned {} entries for {local_dir_path}", entries.len());

        // 2. Get independent FTP session and pre-create remote directories
        let mgr = self.ftp_manager.lock().await;
        let session = mgr.create_independent(conn_name).await
            .map_err(|e| format!("Failed to create FTP session for folder upload: {e}"))?;
        drop(mgr);

        let mut ftp = session.lock().await;
        let remote_base = remote_target_dir.trim_end_matches('/');

        // Pre-create all remote dirs (collect & sort to ensure parent before child)
        let mut remote_dirs: Vec<String> = entries.iter()
            .filter(|(_, _, is_dir)| *is_dir)
            .map(|(path, _, _)| {
                let relative = path.strip_prefix(&local_base.to_string_lossy().to_string())
                    .unwrap_or(path)
                    .trim_start_matches('/')
                    .trim_start_matches('\\');
                format!("{}/{}", remote_base, relative.replace('\\', "/"))
            })
            .collect();
        // Ensure transitive parent dirs exist (sort by depth)
        remote_dirs.sort_by_key(|d| d.matches('/').count());

        for remote_dir in &remote_dirs {
            // MKD may fail if parent doesn't exist — try creating intermediate dirs
            if let Err(_e) = ftp.client.mkdir(remote_dir).await {
                // Try creating by segments
                ensure_remote_dir(&mut ftp.client, remote_dir).await
                    .map_err(|e2| format!("Failed to create remote directory '{}': {e2}", remote_dir))?;
            }
        }

        // Ensure root remote directory
        ensure_remote_dir(&mut ftp.client, remote_base).await
            .map_err(|e| format!("Failed to create remote root directory: {e}"))?;

        // Close independent session
        let _ = ftp.client.quit().await;
        drop(ftp);

        // 3. Build and enqueue file tasks
        let skip_set: std::collections::HashSet<String> = skip_rel_paths.iter().cloned().collect();
        let mut total_bytes: u64 = 0;
        let tasks: Vec<EnqueueTask> = entries.iter()
            .filter(|(_, _, is_dir)| !*is_dir)
            .filter(|(local_full, _, _)| {
                let relative = local_full.strip_prefix(&local_base.to_string_lossy().to_string())
                    .unwrap_or(local_full)
                    .trim_start_matches('/')
                    .trim_start_matches('\\')
                    .replace('\\', "/");
                !skip_set.contains(&relative)
            })
            .map(|(local_full, size, _)| {
                total_bytes += size;
                let relative = local_full.strip_prefix(&local_base.to_string_lossy().to_string())
                    .unwrap_or(local_full)
                    .trim_start_matches('/')
                    .trim_start_matches('\\');
                let remote_full = format!("{}/{}", remote_base, relative.replace('\\', "/"));
                EnqueueTask {
                    op_type: TransferType::FtpUpload,
                    source: local_full.clone(),
                    destination: format!("ftp://{conn_name}{remote_full}"),
                    total_bytes: *size,
                    conn_name: Some(conn_name.to_string()),
                    skip_rel_paths: Vec::new(),
                    permanent: false,
                    password: None,
                    internal_paths: None,
                }
            })
            .collect();

        if tasks.is_empty() {
            return Ok((0, Vec::new()));
        }

        let batch_id = self.next_batch_id();
        let ids = self.enqueue(tasks, sched.clone());

        if move_mode {
            self.save_batch_meta(batch_id, conn_name, local_dir_path);
        }

        Ok((total_bytes, ids))
    }

    fn save_batch_meta(&mut self, _batch_id: u64, _conn_name: &str, _source_path: &str) {
        // Store move-mode metadata for deferred cleanup.
        // Cleanup is handled by the Tauri command after verifying all tasks succeeded.
    }

    pub fn cleanup_finished(&mut self) {
        let finished: Vec<(TransferType, String, String)> = self.active.iter()
            .filter(|(_, a)| a.handle.is_finished())
            .map(|(_, a)| (a.op_type, a.source.clone(), a.dest.clone()))
            .collect();
        let finished_ids: Vec<u64> = self.active.iter()
            .filter(|(_, a)| a.handle.is_finished())
            .map(|(id, _)| *id)
            .collect();
        for id in finished_ids {
            self.active.remove(&id);
        }
        for (op_type, source, dest) in finished {
            self.free_slot_direct(op_type, &source, &dest);
        }
    }
}