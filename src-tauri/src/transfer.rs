use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::sync::Mutex as TokioMutex;

use crate::app_paths;
use tokio::task::JoinHandle;

use crate::ftp::FtpManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferType {
    Copy,
    Move,
    Delete,
    FtpDownload,
    FtpUpload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    pub id: u64,
    pub batch_id: u64,
    pub op_type: TransferType,
    pub status: TaskStatus,
    pub source: String,
    pub destination: String,
    pub total_bytes: u64,
    pub bytes_done: u64,
    pub conn_name: Option<String>,
    #[serde(skip)]
    pub cancel_flag: Option<Arc<AtomicBool>>,
    #[serde(skip)]
    pub skip_rel_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryRecord {
    pub id: u64,
    pub batch_id: u64,
    pub transfer_type: TransferType,
    pub source: String,
    pub destination: String,
    pub total_bytes: u64,
    pub bytes_transferred: u64,
    pub status: TaskStatus,
    pub error_message: Option<String>,
    pub started_at: u64,
    pub completed_at: u64,
    pub avg_speed_bps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueTask {
    pub op_type: TransferType,
    pub source: String,
    pub destination: String,
    pub total_bytes: u64,
    pub conn_name: Option<String>,
    #[serde(default)]
    pub skip_rel_paths: Vec<String>,
}

struct ActiveTask {
    handle: JoinHandle<()>,
    cancel_flag: Arc<AtomicBool>,
    op_type: TransferType,
    source: String,
    dest: String,
}

pub struct TransferScheduler {
    app: AppHandle,
    ftp_manager: Arc<TokioMutex<FtpManager>>,
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
}

impl TransferScheduler {
    pub fn new(app: AppHandle, ftp_manager: Arc<TokioMutex<FtpManager>>) -> Self {
        let history_path = app_paths::state_file("transfer-history.json");
        let legacy_history_path = app_paths::legacy_roaming_file("transfer-history.json");
        if let Err(error) = app_paths::migrate_legacy_file(&history_path, &legacy_history_path) {
            eprintln!("[transfer] Failed to migrate history: {}", error);
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
        eprintln!("[transfer] enqueue: {} task(s)", tasks.len());
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

    pub fn get_slot_config(&self) -> (usize, usize) {
        (self.ftp_max_per_conn, self.local_max_slots)
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
        eprintln!("[transfer] enqueue_ftp_folder_download: conn={conn_name} remote={remote_dir_path} → local={local_target_dir}");

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
        eprintln!("[transfer] list_dir_recursive returned {entry_count} entries for {remote_dir_path}");

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
        eprintln!("[transfer] enqueue_ftp_folder_upload: local={local_dir_path} → conn={conn_name} remote={remote_target_dir}");

        let local_base = Path::new(local_dir_path);
        if !local_base.is_dir() {
            return Err(format!("Not a directory: {}", local_dir_path));
        }

        // 1. Recursively scan local directory
        let mut entries: Vec<(String, u64, bool)> = Vec::new(); // (full_path, size, is_dir)
        walk_local_dir(&local_base.to_path_buf(), local_base, &mut entries)
            .map_err(|e| format!("Failed to scan local directory: {e}"))?;

        eprintln!("[transfer] local walk returned {} entries for {local_dir_path}", entries.len());

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

// ── Helpers ──

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    collect_dir_size(path, &mut total);
    total
}

fn collect_dir_size(dir: &Path, total: &mut u64) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_dir_size(&path, total);
            } else if let Ok(meta) = path.metadata() {
                *total += meta.len();
            }
        }
    }
}

/// Recursively collect conflicts between a source directory tree and its
/// destination. Returns relative paths (relative to `root`) of entries that
/// already exist at `dst`.
fn collect_dir_conflicts(src: &Path, dst: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(src) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        let rel = path.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| name.to_string_lossy().to_string());
        if path.is_dir() {
            collect_dir_conflicts(&path, &dst_path, root, out);
        } else if dst_path.exists() {
            out.push(rel);
        }
    }
}

/// Recursively scan a source directory tree and emit a
/// `transfer-conflict-found` event for each conflicting file.
pub fn scan_dir_conflicts(src: &Path, dst: &Path, root: &Path, app: &AppHandle) {
    let Ok(entries) = fs::read_dir(src) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        let rel = path.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| name.to_string_lossy().to_string());
        if path.is_dir() {
            scan_dir_conflicts(&path, &dst_path, root, app);
        } else if dst_path.exists() {
            let _ = app.emit("transfer-conflict-found", serde_json::json!({
                "kind": "dir",
                "dir_source": root.to_string_lossy().to_string(),
                "source": path.to_string_lossy(),
                "destination": dst_path.to_string_lossy(),
                "rel_path": rel,
            }));
        }
    }
}

/// Check conflicts for a batch of transfer tasks (local copy/move directories).
pub fn check_transfer_conflicts(tasks: &[EnqueueTask]) -> Vec<String> {
    let mut conflicts = Vec::new();
    for task in tasks {
        if task.op_type != TransferType::Copy && task.op_type != TransferType::Move {
            continue;
        }
        let src = Path::new(&task.source);
        let dst = Path::new(&task.destination);
        if src.is_dir() {
            collect_dir_conflicts(src, dst, src, &mut conflicts);
        }
    }
    conflicts
}

fn extract_ftp_conn(path: &str) -> Option<String> {
    if path.starts_with("ftp://") {
        let rest = &path[6..];
        rest.split('/').next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Recursively walk a local directory, collecting (full_path, size, is_dir).
pub fn walk_local_dir(
    current: &PathBuf,
    base: &Path,
    out: &mut Vec<(String, u64, bool)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        if path.is_dir() {
            out.push((path_str.clone(), 0, true));
            walk_local_dir(&path.to_path_buf(), base, out)?;
        } else {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            out.push((path_str, size, false));
        }
    }
    Ok(())
}

/// Ensure a remote directory path exists on the FTP server, creating
/// intermediate directories as needed.
async fn ensure_remote_dir(
    client: &mut suppaftp::tokio::AsyncRustlsFtpStream,
    remote_path: &str,
) -> Result<(), String> {
    let path = remote_path.trim_end_matches('/');
    // Try creating the full path directly
    match client.mkdir(path).await {
        Ok(_) => return Ok(()),
        Err(e) => eprintln!("[transfer] MKD '{}' direct failed, building by segments: {}", path, e),
    }

    // Build directory path segment by segment
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut built = String::new();
    for seg in segments {
        built.push('/');
        built.push_str(seg);
        match client.mkdir(&built).await {
            Ok(_) => continue,
            Err(e) => {
                // File exists / dir already exists → ok
                let err_str = e.to_string();
                if err_str.contains("550") || err_str.contains("521") || err_str.contains("exist") {
                    continue;
                }
                return Err(format!("MKD '{}' failed: {}", built, e));
            }
        }
    }
    Ok(())
}

// ── Transfer execution ──

async fn execute_transfer(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) {
    let start = Instant::now();
    let result = match task.op_type {
        TransferType::FtpDownload => {
            execute_ftp_download(task, cancel_flag, app, ftp_manager).await
        }
        TransferType::FtpUpload => {
            execute_ftp_upload(task, cancel_flag, app, ftp_manager).await
        }
        TransferType::Copy | TransferType::Move => {
            execute_local_copy(task, cancel_flag, app).await
        }
        TransferType::Delete => {
            if task.conn_name.is_some() || task.source.starts_with("ftp://") {
                execute_ftp_delete(task, cancel_flag, app, ftp_manager).await
            } else {
                execute_local_delete(task, cancel_flag, app).await
            }
        }
    };

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis() as u64;
    let _bytes = task.total_bytes;

    // We can't access the scheduler from here without the Arc.
    // Scheduling cleanup happens via the Tauri commands that wrap this.
    // For now, emit complete/failed directly and let the scheduler's callers handle cleanup.
    match result {
        Ok(bytes_done) => {
            let avg_speed = if elapsed_ms > 0 {
                (bytes_done as f64 / (elapsed_ms as f64 / 1000.0)) as u64
            } else {
                0
            };
            let _ = app.emit("transfer-complete", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "bytes_done": bytes_done,
                "elapsed_ms": elapsed_ms, "avg_speed_bps": avg_speed,
                "source": task.source, "destination": task.destination,
            }));
        }
        Err(e) => {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = app.emit("transfer-cancelled", serde_json::json!({
                    "id": task.id, "batch_id": task.batch_id,
                    "op_type": task.op_type,
                    "source": task.source, "destination": task.destination,
                }));
            } else {
                let _ = app.emit("transfer-failed", serde_json::json!({
                    "id": task.id, "batch_id": task.batch_id,
                    "op_type": task.op_type, "error": e,
                    "source": task.source, "destination": task.destination,
                }));
            }
        }
    }
}

fn check_cancelled(cancel_flag: &Arc<AtomicBool>) -> bool {
    cancel_flag.load(Ordering::Relaxed)
}

// ── FTP transfer workers ──

async fn execute_ftp_download(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.source))
        .ok_or_else(|| "No FTP connection for download".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.source[6..].find('/') {
        &task.source[6 + pos..]
    } else {
        "/"
    };
    eprintln!("[transfer] FTP download: conn={conn_name} remote={remote_path} → local={}", task.destination);

    let mgr = ftp_manager.lock().await;
    let session = mgr.create_independent(conn_name).await?;
    drop(mgr);

    let mut ftp = session.lock().await;

    // Try to get file size for progress display
    let file_size = match ftp.client.size(remote_path).await {
        Ok(s) => { eprintln!("[transfer] FTP SIZE: {s} bytes"); s }
        Err(e) => { eprintln!("[transfer] FTP SIZE failed (ignored): {e}"); 0 }
    };

    let mut stream = ftp.client.retr_as_stream(remote_path)
        .await
        .map_err(|e| {
            let msg = format!("FTP download failed: {e}");
            eprintln!("[transfer] {msg}");
            msg
        })?;

    let local = Path::new(&task.destination);
    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(local)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut buf = vec![0u8; 65536];
    let mut done: u64 = 0;
    let total = if file_size > 0 { file_size as u64 } else { task.total_bytes };
    // Immediately update frontend with the correct file size
    if total > 0 {
        let _ = app.emit("transfer-progress", serde_json::json!({
            "id": task.id, "batch_id": task.batch_id,
            "op_type": task.op_type, "status": "running",
            "bytes_done": 0, "total_bytes": total, "speed_bps": 0,
            "source": task.source, "destination": task.destination,
        }));
    }
    let mut last_emit = Instant::now();
    let start = Instant::now();

    loop {
        if check_cancelled(cancel_flag) {
            drop(file);
            let _ = tokio::fs::remove_file(local).await;
            return Err("Cancelled".into());
        }

        use tokio::io::AsyncReadExt;
        let n = stream.read(&mut buf)
            .await
            .map_err(|e| format!("Download read error: {}", e))?;
        if n == 0 { break; }

        file.write_all(&buf[..n])
            .await
            .map_err(|e| format!("Download write error: {}", e))?;

        done += n as u64;
        let now = Instant::now();
        if now.duration_since(last_emit).as_millis() >= 100 {
            let elapsed = start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (done as f64 / elapsed) as u64 } else { 0 };
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": done, "total_bytes": total,
                "speed_bps": speed,
                "source": task.source, "destination": task.destination,
            }));
            last_emit = now;
        }
    }

    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    Ok(done)
}

async fn execute_ftp_upload(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.destination))
        .ok_or_else(|| "No FTP connection for upload".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.destination[6..].find('/') {
        &task.destination[6 + pos..]
    } else {
        "/"
    };
    eprintln!("[transfer] FTP upload: local={} → conn={conn_name} remote={remote_path}", task.source);

    let mgr = ftp_manager.lock().await;
    let session = mgr.create_independent(conn_name).await?;
    drop(mgr);

    let mut ftp = session.lock().await;
    let local = Path::new(&task.source);
    let file = tokio::fs::File::open(local)
        .await
        .map_err(|e| format!("Failed to open local file: {}", e))?;
    let total = file.metadata()
        .await
        .map(|m| m.len())
        .unwrap_or(task.total_bytes);

    let mut reader = ProgressAsyncReader {
        inner: file,
        done: 0,
        total,
        task_id: task.id,
        batch_id: task.batch_id,
        op_type: task.op_type,
        source: task.source.clone(),
        dest: task.destination.clone(),
        last_emit: Instant::now(),
        start: Instant::now(),
        cancel_flag: cancel_flag.clone(),
        app: app.clone(),
    };

    let put_result = ftp.client.put_file(remote_path, &mut reader).await;

    match put_result {
        Ok(_) => Ok(reader.done),
        Err(e) => {
            if reader.cancel_flag.load(Ordering::Relaxed) {
                // Try to delete partial file from server
                let _ = ftp.client.rm(remote_path).await;
                eprintln!("[transfer] FTP upload cancelled, cleaned up remote file: {remote_path}");
                Err("Cancelled".into())
            } else {
                Err(format!("FTP upload failed: {}", e))
            }
        }
    }
}

struct ProgressAsyncReader {
    inner: tokio::fs::File,
    done: u64,
    total: u64,
    task_id: u64,
    batch_id: u64,
    op_type: TransferType,
    source: String,
    dest: String,
    last_emit: Instant,
    start: Instant,
    cancel_flag: Arc<AtomicBool>,
    app: AppHandle,
}

impl AsyncRead for ProgressAsyncReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        if this.cancel_flag.load(Ordering::Relaxed) {
            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Cancelled")));
        }
        let filled_before = buf.filled().len();
        let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        let result = AsyncRead::poll_read(inner, cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            let n = (buf.filled().len() - filled_before) as u64;
            if n > 0 {
                this.done += n;
                let now = Instant::now();
                if now.duration_since(this.last_emit).as_millis() >= 100 {
                    let elapsed = this.start.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 { (this.done as f64 / elapsed) as u64 } else { 0 };
                    let _ = this.app.emit("transfer-progress", serde_json::json!({
                        "id": this.task_id, "batch_id": this.batch_id,
                        "op_type": this.op_type, "status": "running",
                        "bytes_done": this.done, "total_bytes": this.total,
                        "speed_bps": speed,
                        "source": this.source, "destination": this.dest,
                    }));
                    this.last_emit = now;
                }
            }
        }
        result
    }
}

// ── FTP delete worker ──

async fn execute_ftp_delete(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    ftp_manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<u64, String> {
    let conn_owned: String = task.conn_name.clone()
        .or_else(|| extract_ftp_conn(&task.source))
        .ok_or_else(|| "No FTP connection for delete".to_string())?;
    let conn_name = conn_owned.as_str();
    let remote_path = if let Some(pos) = task.source[6..].find('/') {
        &task.source[6 + pos..]
    } else {
        "/"
    };
    eprintln!("[transfer] FTP delete: conn={conn_name} remote={remote_path}");

    let mgr = ftp_manager.lock().await;
    let session = mgr.create_independent(conn_name).await?;
    drop(mgr);

    let mut ftp = session.lock().await;

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }

    // Try rm (file), fall back to rmdir (directory)
    match ftp.client.rm(remote_path).await {
        Ok(_) => {
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": 1, "total_bytes": 1,
                "speed_bps": 0,
                "source": task.source, "destination": task.destination,
            }));
            Ok(1)
        }
        Err(rm_err) => {
            eprintln!("[transfer] FTP delete rm failed, trying rmdir: {rm_err}");
            ftp.client.rmdir(remote_path)
                .await
                .map_err(|e| format!("FTP delete failed (both rm and rmdir): {rm_err} / {e}"))?;
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": 1, "total_bytes": 1,
                "speed_bps": 0,
                "source": task.source, "destination": task.destination,
            }));
            Ok(1)
        }
    }
}

// ── Local file operation workers ──

async fn execute_local_copy(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let src = Path::new(&task.source);
    let dst = Path::new(&task.destination);

    if !src.exists() {
        return Err(format!("Source not found: {}", task.source));
    }

    // Ensure parent dir
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tokio::task::spawn_blocking({
        let cancel = cancel_flag.clone();
        let app = app.clone();
        let task_clone = task.clone();
        let src = src.to_path_buf();
        let dst = dst.to_path_buf();
        move || {
            let mut task_corrected = task_clone;
            // Runtime fallback: if enqueue didn't compute the directory size,
            // compute it here (on the blocking thread) and emit the correct total.
            if src.is_dir() && task_corrected.total_bytes == 0 {
                let actual_total = dir_size(&src);
                if actual_total > 0 {
                    task_corrected.total_bytes = actual_total;
                    let _ = app.emit("transfer-progress", serde_json::json!({
                        "id": task_corrected.id,
                        "batch_id": task_corrected.batch_id,
                        "op_type": task_corrected.op_type,
                        "status": "running",
                        "bytes_done": 0,
                        "total_bytes": actual_total,
                        "speed_bps": 0,
                        "source": task_corrected.source,
                        "destination": task_corrected.destination,
                    }));
                }
            }
            local_copy_blocking(&src, &dst, &cancel, &app, &task_corrected)
        }
    }).await
    .map_err(|e| format!("Join error: {}", e))?
}

fn local_copy_blocking(
    src: &Path,
    dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    task: &TransferTask,
) -> Result<u64, String> {
    let mut total: u64 = 0;
    let mut last_emit = Instant::now();
    let dst_existed = dst.exists();
    let mut created: Vec<PathBuf> = Vec::new();
    let skip: std::collections::HashSet<String> = task.skip_rel_paths.iter().cloned().collect();

    let copy_result = if src.is_dir() {
        copy_dir_with_progress(src, dst, cancel_flag, app, task, &mut total, &mut last_emit, &mut created, &skip, src)
    } else {
        copy_file_with_progress(src, dst, cancel_flag, app, task, &mut total, &mut last_emit)
    };

    if let Err(e) = copy_result {
        if e == "Cancelled" && src.is_dir() {
            cleanup_cancelled_dir(dst, dst_existed, &created);
        }
        return Err(e);
    }

    // If this is a move, delete source after copy
    if task.op_type == TransferType::Move {
        if src.is_dir() {
            let _ = fs::remove_dir_all(src);
        } else {
            let _ = fs::remove_file(src);
        }
    }

    Ok(total)
}

fn cleanup_cancelled_dir(dst: &Path, dst_existed: bool, created: &[PathBuf]) {
    if !dst_existed {
        let _ = fs::remove_dir_all(dst);
    } else {
        for f in created {
            let _ = fs::remove_file(f);
        }
    }
}

fn copy_file_with_progress(
    src: &Path, dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle, task: &TransferTask,
    total: &mut u64, last_emit: &mut Instant,
) -> Result<(), String> {
    let mut reader = fs::File::open(src)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let mut writer = fs::File::create(dst)
        .map_err(|e| format!("Failed to create: {}", e))?;
    let mut buf = [0u8; 65536];
    let source_str = task.source.clone();
    let dest_str = task.destination.clone();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            drop(writer);
            let _ = fs::remove_file(dst);
            return Err("Cancelled".into());
        }
        let n = reader.read(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        std::io::Write::write_all(&mut writer, &buf[..n])
            .map_err(|e| format!("Write error: {}", e))?;
        *total += n as u64;

        let now = Instant::now();
        if now.duration_since(*last_emit).as_millis() >= 100 {
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": *total, "total_bytes": task.total_bytes,
                "speed_bps": 0,
                "source": source_str, "destination": dest_str,
            }));
            *last_emit = now;
        }
    }

    writer.flush().map_err(|e| format!("Flush error: {}", e))?;
    Ok(())
}

fn copy_dir_with_progress(
    src: &Path, dst: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle, task: &TransferTask,
    total: &mut u64, last_emit: &mut Instant,
    created: &mut Vec<PathBuf>,
    skip: &std::collections::HashSet<String>,
    root: &Path,
) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Read dir error: {}", e))? {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let rel = path.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if skip.contains(&rel) {
            continue;
        }

        if path.is_dir() {
            copy_dir_with_progress(&path, &dst_path, cancel_flag, app, task, total, last_emit, created, skip, root)?;
        } else {
            copy_file_with_progress(&path, &dst_path, cancel_flag, app, task, total, last_emit)?;
            created.push(dst_path);
        }
    }
    Ok(())
}

async fn execute_local_delete(
    task: &TransferTask,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<u64, String> {
    let path = Path::new(&task.source);
    if !path.exists() {
        return Ok(0);
    }

    tokio::task::spawn_blocking({
        let cancel = cancel_flag.clone();
        let app = app.clone();
        let task_clone = task.clone();
        let path = path.to_path_buf();
        move || {
            let mut total: u64 = 0;
            let mut last_emit = Instant::now();
            delete_with_progress(&path, &cancel, &app, &task_clone, &mut total, &mut last_emit)
                .map(|_| total)
        }
    }).await
    .map_err(|e| format!("Join error: {}", e))?
}

fn delete_with_progress(
    path: &Path,
    cancel_flag: &Arc<AtomicBool>,
    app: &AppHandle,
    task: &TransferTask,
    total: &mut u64,
    last_emit: &mut Instant,
) -> Result<(), String> {
    if cancel_flag.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }

    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("Read dir error: {}", e))? {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            delete_with_progress(&entry.path(), cancel_flag, app, task, total, last_emit)?;
        }
        fs::remove_dir(path).map_err(|e| format!("Remove dir error: {}", e))?;
    } else {
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        fs::remove_file(path).map_err(|e| format!("Remove file error: {}", e))?;
        *total += size;

        let now = Instant::now();
        if now.duration_since(*last_emit).as_millis() >= 100 {
            let _ = app.emit("transfer-progress", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "status": "running",
                "bytes_done": *total, "total_bytes": task.total_bytes,
                "speed_bps": 0,
                "source": task.source.clone(), "destination": task.destination.clone(),
            }));
            *last_emit = now;
        }
    }
    Ok(())
}
