// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Second mode of the same executable: run archive extraction in an isolated
    // process so a fail-fast abort kills only the worker, not the UI. This must
    // return before any Tauri/UI initialization.
    if std::env::args().any(|arg| arg == "--archive-extract-worker") {
        wind_lib::archive_worker::run();
        return;
    }

    wind_lib::run()
}
