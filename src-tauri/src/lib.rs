mod app_paths;
mod archive;
mod commands;
mod directory_watcher;
mod file_watcher;
mod ftp;
mod pdf;
mod python_completion;
mod terminal;
mod tool_cache;
mod transfer;
mod video;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    is_hidden: bool,
    modified: Option<u64>,
    created: Option<u64>,
    children: Option<Vec<FileEntry>>,
}

struct AppState {
    terminal: terminal::TerminalManager,
    file_watcher: Mutex<file_watcher::FileWatcher>,
    directory_watcher: Mutex<directory_watcher::DirectoryWatcher>,
    ftp_manager: Arc<TokioMutex<ftp::FtpManager>>,
    transfer_scheduler: Arc<TokioMutex<transfer::TransferScheduler>>,
}

fn setup_logging(app_handle: &tauri::AppHandle) {
    let is_dev = cfg!(debug_assertions);

    // Determine log level from RUST_LOG env var
    let level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|v| {
            let level_str: Option<String> = v
                .split(',')
                .find(|s| s.starts_with("wind"))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.to_lowercase());
            level_str
        })
        .and_then(|l| match l.as_str() {
            "trace" => Some(log::LevelFilter::Trace),
            "debug" => Some(log::LevelFilter::Debug),
            "info" => Some(log::LevelFilter::Info),
            "warn" => Some(log::LevelFilter::Warn),
            "error" => Some(log::LevelFilter::Error),
            _ => None,
        })
        .unwrap_or(log::LevelFilter::Info);

    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level);

    if is_dev {
        // Dev mode: stdout only
        dispatch = dispatch.chain(std::io::stdout());
    } else {
        // Release mode: file output with date-based rotation
        let mut file_logging_ok = false;
        if let Ok(log_dir) = app_handle.path().app_local_data_dir().map(|p| p.join("logs")) {
            if std::fs::create_dir_all(&log_dir).is_ok() {
                // State for tracking current log file and date
                let state: &'static Mutex<(Option<std::fs::File>, String)> =
                    Box::leak(Box::new(Mutex::new((None, String::new()))));

                // Use a custom log target for date-based file rotation
                let log_dir_clone = log_dir.clone();
                dispatch = dispatch.chain(fern::Output::call(move |record| {
                    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                    let mut guard = state.lock().unwrap();
                    let (ref mut file, ref mut current_date) = *guard;
                    if current_date != &today {
                        let log_path = log_dir_clone.join(format!("wind-{}.log", today));
                        if let Ok(f) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&log_path)
                        {
                            *file = Some(f);
                            *current_date = today;
                        }
                    }
                    if let Some(ref mut f) = file {
                        use std::io::Write;
                        let _ = writeln!(f, "{}", record.args());
                    }
                }));
                file_logging_ok = true;
            }
        }
        if !file_logging_ok {
            // Fallback: log to stderr so messages are not silently lost
            dispatch = dispatch.chain(std::io::stderr());
        }
    }

    dispatch.apply().ok();
}

/// Clean up log files older than 7 days
fn cleanup_old_logs(app_handle: &tauri::AppHandle) {
    if let Ok(log_dir) = app_handle.path().app_local_data_dir().map(|p| p.join("logs")) {
        if !log_dir.exists() {
            return;
        }
        let cutoff = chrono::Local::now() - chrono::Duration::days(7);
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                // Match wind-YYYY-MM-DD.log pattern
                if name_str.starts_with("wind-") && name_str.ends_with(".log") {
                    let date_part = &name_str[5..name_str.len() - 4];
                    if let Ok(file_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                        if file_date < cutoff.naive_local().date() {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

/// Tauri command for frontend to send log messages
#[tauri::command]
fn frontend_log(level: String, message: String) {
    match level.to_lowercase().as_str() {
        "error" => log::error!("[frontend] {}", message),
        "warn" => log::warn!("[frontend] {}", message),
        "debug" => log::debug!("[frontend] {}", message),
        _ => log::info!("[frontend] {}", message),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize logging
            setup_logging(&handle);
            log::info!("Wind application starting");

            // Clean up old log files
            cleanup_old_logs(&handle);

            let mut terminal = terminal::TerminalManager::new();
            terminal.set_app_handle(handle.clone());
            let mut ftp_manager = ftp::FtpManager::new();
            ftp_manager.load_on_startup();
            let ftp_manager = Arc::new(TokioMutex::new(ftp_manager));
            let transfer_scheduler = Arc::new(TokioMutex::new(transfer::TransferScheduler::new(
                handle.clone(),
                ftp_manager.clone(),
            )));
            app.manage(AppState {
                terminal,
                file_watcher: Mutex::new(file_watcher::FileWatcher::new()),
                directory_watcher: Mutex::new(directory_watcher::DirectoryWatcher::new()),
                ftp_manager: ftp_manager.clone(),
                transfer_scheduler: transfer_scheduler.clone(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::file_info::get_home_dir,
            commands::file_info::file_exists,
            commands::file_info::get_file_metadata,
            commands::watchers::start_watch_file,
            commands::watchers::stop_watch_file,
            commands::watchers::start_watch_directory,
            commands::watchers::stop_watch_directory,
            commands::directory::read_directory,
            commands::archive_cmd::read_archive_directory,
            commands::archive_cmd::read_archive_file,
            commands::archive_cmd::extract_archive_files,
            commands::archive_cmd::extract_archive,
            commands::archive_cmd::compress_files,
            commands::archive_cmd::archive_delete_entry,
            commands::archive_cmd::archive_rename_entry,
            commands::archive_cmd::archive_add_files,
            commands::archive_cmd::archive_write_file,
            commands::archive_cmd::archive_create_entry,
            commands::archive_cmd::extract_enqueue,
            commands::directory::list_drives,
            commands::recycle::list_recycle_bin,
            commands::recycle::restore_recycle_items,
            commands::recycle::purge_recycle_items,
            commands::recycle::empty_recycle_bin,
            commands::file_ops::delete_file,
            commands::file_ops::rename_file,
            commands::file_ops::batch_rename,
            commands::file_ops::create_batch_rename_temp_file,
            commands::file_ops::delete_temp_file,
            commands::file_ops::create_file,
            commands::transfer_cmd::transfer_enqueue,
            commands::transfer_cmd::transfer_cancel,
            commands::transfer_cmd::transfer_cancel_all,
            commands::transfer_cmd::scan_transfer_conflicts,
            commands::transfer_cmd::scan_ftp_upload_conflicts,
            commands::transfer_cmd::scan_ftp_download_conflicts,
            commands::transfer_cmd::transfer_reorder,
            commands::transfer_cmd::transfer_get_history,
            commands::transfer_cmd::transfer_clear_history,
            commands::transfer_cmd::transfer_set_ftp_slots,
            commands::transfer_cmd::transfer_set_local_slots,
            commands::transfer_cmd::transfer_set_extract_slots,
            commands::transfer_cmd::transfer_get_slots,
            commands::file_info::get_file_info,
            commands::directory::calculate_folder_size,
            commands::directory::cancel_folder_size,
            commands::file_io::open_file,
            commands::file_io::open_with_dialog,
            commands::file_io::read_file,
            commands::file_io::read_file_partial,
            commands::file_io::read_binary_file,
            commands::file_io::read_binary_file_partial,
            commands::file_io::read_image_thumbnail,
            commands::file_io::write_file,
            commands::terminal_cmd::terminal_spawn,
            commands::terminal_cmd::terminal_input,
            commands::terminal_cmd::terminal_resize,
            commands::terminal_cmd::terminal_kill,
            commands::misc::exec_shell_command,
            commands::search::search_files,
            commands::search::cancel_search,
            commands::search::check_search_tools,
            commands::misc::set_ime_enabled,
            commands::config::read_config,
            commands::config::write_config,
            pdf::get_pdf_info,
            pdf::render_pdf_page,
            pdf::render_pdf_tile,
            pdf::search_pdf_text,
            pdf::get_pdf_outline,
            pdf::get_pdf_page_links,
            pdf::clear_pdf_cache,
            video::get_video_thumbnail,
            video::start_video_server,
            video::stop_video_server,
            commands::ftp_cmd::ftp_connect,
            commands::ftp_cmd::ftp_disconnect,
            commands::ftp_cmd::ftp_read_directory,
            commands::ftp_cmd::ftp_download_folder,
            commands::ftp_cmd::ftp_upload_folder,
            commands::ftp_cmd::ftp_delete,
            commands::ftp_cmd::ftp_rename,
            commands::ftp_cmd::ftp_copy,
            commands::ftp_cmd::ftp_create_file,
            commands::ftp_cmd::ftp_mkdir,
            commands::ftp_cmd::list_ftp_connections,
            commands::ftp_cmd::check_ftp_connection,
            python_completion::get_package_api,
            frontend_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}