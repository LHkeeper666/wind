mod conflict;
mod extract;
mod ftp;
mod helpers;
mod local;
mod scheduler;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as TokioMutex;

use crate::ftp::FtpManager;

pub use conflict::{scan_dir_conflicts, walk_local_dir};
pub use scheduler::TransferScheduler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferType {
    Copy,
    Move,
    Delete,
    FtpDownload,
    FtpUpload,
    Extract,
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
    #[serde(skip)]
    pub permanent: bool,
    /// Archive password for Extract tasks.
    #[serde(skip)]
    pub password: Option<String>,
    /// Internal paths for partial extraction (y+p from archive browser).
    #[serde(skip)]
    pub internal_paths: Option<Vec<String>>,
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
    #[serde(default)]
    pub permanent: bool,
    /// Archive password for Extract tasks.
    #[serde(default)]
    pub password: Option<String>,
    /// Internal paths for partial extraction (y+p from archive browser).
    #[serde(default)]
    pub internal_paths: Option<Vec<String>>,
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
            ftp::execute_ftp_download(task, cancel_flag, app, ftp_manager).await
        }
        TransferType::FtpUpload => {
            ftp::execute_ftp_upload(task, cancel_flag, app, ftp_manager).await
        }
        TransferType::Copy | TransferType::Move => {
            local::execute_local_copy(task, cancel_flag, app).await
        }
        TransferType::Delete => {
            if task.conn_name.is_some() || task.source.starts_with("ftp://") {
                ftp::execute_ftp_delete(task, cancel_flag, app, ftp_manager).await
            } else {
                local::execute_local_delete(task, cancel_flag, app).await
            }
        }
        TransferType::Extract => {
            extract::execute_extract(task, cancel_flag, app).await
        }
    };

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis() as u64;

    match result {
        Ok(bytes_done) => {
            let avg_speed = if elapsed_ms > 0 {
                (bytes_done as f64 / (elapsed_ms as f64 / 1000.0)) as u64
            } else {
                0
            };
            if let Err(e) = app.emit("transfer-complete", serde_json::json!({
                "id": task.id, "batch_id": task.batch_id,
                "op_type": task.op_type, "bytes_done": bytes_done,
                "elapsed_ms": elapsed_ms, "avg_speed_bps": avg_speed,
                "source": task.source, "destination": task.destination,
            })) {
                log::debug!("[emit] transfer-complete failed: {}", e);
            }
        }
        Err(e) => {
            if cancel_flag.load(Ordering::Relaxed) {
                if let Err(emit_err) = app.emit("transfer-cancelled", serde_json::json!({
                    "id": task.id, "batch_id": task.batch_id,
                    "op_type": task.op_type,
                    "source": task.source, "destination": task.destination,
                })) {
                    log::debug!("[emit] transfer-cancelled failed: {}", emit_err);
                }
            } else {
                if let Err(emit_err) = app.emit("transfer-failed", serde_json::json!({
                    "id": task.id, "batch_id": task.batch_id,
                    "op_type": task.op_type, "error": e,
                    "source": task.source, "destination": task.destination,
                })) {
                    log::debug!("[emit] transfer-failed failed: {}", emit_err);
                }
            }
        }
    }
}