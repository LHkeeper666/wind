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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
