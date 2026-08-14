## Tasks

### Phase 1: Rust — Recursive FTP directory listing

- [ ] **Task 1.1**: Add `list_dir_recursive()` to `src-tauri/src/ftp.rs`
  - Recursively walk a remote path using MLSD (fallback LIST via `parse_list_line`)
  - Return `Vec<(String, u64, bool)>` where tuple = `(full_remote_path, size_bytes, is_dir)`
  - Handle permission errors: abort entire operation with path-specific error message
  - Handle empty directories: return empty list
  - Verify: `cargo check` passes

### Phase 2: Rust — Folder transfer command logic

- [ ] **Task 2.1**: Add `enqueue_ftp_folder_download()` to `src-tauri/src/transfer.rs`
  - Take `(conn_name, remote_dir_path, local_target_dir, move_mode)`
  - Call `list_dir_recursive()` to get full file tree
  - Pre-create all local directories via `fs::create_dir_all()`
  - Build `Vec<EnqueueTask>` for all files (not dirs) with `TransferType::FtpDownload`
  - Call `scheduler.enqueue(tasks)`

- [ ] **Task 2.2**: Add `enqueue_ftp_folder_upload()` to `src-tauri/src/transfer.rs`
  - Take `(conn_name, local_dir_path, remote_target_dir, move_mode)`
  - Recursively scan local directory with `std::fs::read_dir`
  - Pre-create all remote directories via FTP MKD (handle intermediate directory creation)
  - Build `Vec<EnqueueTask>` for all files with `TransferType::FtpUpload`
  - Call `scheduler.enqueue(tasks)`

- [ ] **Task 2.3**: Add `ftp_download_folder` Tauri command to `src-tauri/src/lib.rs`
  - parameters: `(state, conn_name, remote_path, local_path, move_mode)`
  - Acquire scheduler lock, call `enqueue_ftp_folder_download()`, save history for cut-mode deletion

- [ ] **Task 2.4**: Add `ftp_upload_folder` Tauri command to `src-tauri/src/lib.rs`
  - parameters: `(state, conn_name, local_path, remote_path, move_mode)`
  - Acquire scheduler lock, call `enqueue_ftp_folder_upload()`, save history for cut-mode deletion

- [ ] **Task 2.5**: Implement cut-mode cleanup for folder transfers
  - After all tasks in a folder batch complete successfully: delete source (local dir or remote dir tree)
  - If any task failed: skip cleanup entirely
  - Leverage existing `execute_ftp_delete` / `execute_local_delete` for source cleanup
  - Verify: `cargo check` passes

### Phase 3: Frontend — Route directory paste to folder commands

- [ ] **Task 3.1**: Update `DirectoryPanel.svelte` paste handler
  - When clipboard contains a directory entry (`is_dir=true`) and the target panel is on a different filesystem (FTP ↔ local), call `ftp_download_folder` or `ftp_upload_folder` instead of the single-file path
  - The existing single-file paste path for `is_dir=false` remains unchanged
  - Verify: `npx svelte-check` passes

### Phase 4: Integration test

- [ ] **Task 4.1**: Manual end-to-end test plan
  - Connect to FTP test server
  - Download a folder with 3 levels of nesting → verify directory tree created locally, all files present, batch visible in TransferManager
  - Upload a folder to FTP → verify remote directory tree, all files present
  - Cancel mid-transfer → verify partial files cleaned, remaining batch cancelled
  - Cut (move) folder from FTP → verify remote deleted after successful download
  - Empty folder download → verify only the directory itself is created, no errors
