## Tasks

### Phase 1: 基础设施

- [x] 1.1 创建 `src-tauri/src/commands/mod.rs`，声明所有子模块
- [x] 1.2 在 `lib.rs` 添加 `mod commands;` 声明

### Phase 2: 独立模块迁移（无 AppState 依赖）

- [x] 2.1 迁移 `config.rs`：read_config, write_config
- [x] 2.2 迁移 `misc.rs`：set_ime_enabled, exec_shell_command, detect_bash_path, ShellOutput
- [x] 2.3 迁移 `file_info.rs`：get_file_info, get_file_metadata, file_exists, get_home_dir, FileInfo, FileMetadata, chrono_like, is_leap
- [x] 2.4 迁移 `file_io.rs`：read_file, read_file_partial, read_binary_file, read_binary_file_partial, read_image_thumbnail, write_file, open_file, open_with_dialog, decode_text, ImageThumbnail, THUMBNAIL_* 常量
- [x] 2.5 迁移 `file_ops.rs`：delete_file, rename_file, batch_rename, create_batch_rename_temp_file, delete_temp_file, create_file, RenameEntry
- [x] 2.6 迁移 `recycle.rs`：list_recycle_bin, restore_recycle_items, purge_recycle_items, empty_recycle_bin, TrashItemDto

### Phase 3: 有依赖的模块迁移

- [x] 3.1 迁移 `search.rs`：search_files, cancel_search, check_search_tools, 搜索辅助函数, SEARCH_CANCELLED, SearchResult
- [x] 3.2 迁移 `directory.rs`：read_directory, list_virtual_root, list_drives, calculate_folder_size, cancel_folder_size, walk_dir, FOLDER_SIZE_CALCS
- [x] 3.3 迁移 `archive_cmd.rs`：10 个压缩包 Tauri commands
- [x] 3.4 迁移 `terminal_cmd.rs`：terminal_spawn, terminal_input, terminal_resize, terminal_kill
- [x] 3.5 迁移 `transfer_cmd.rs`：10 个传输调度器 Tauri commands
- [x] 3.6 迁移 `ftp_cmd.rs`：13 个 FTP Tauri commands + 辅助函数

### Phase 4: 清理与验证

- [x] 4.1 清理 lib.rs 中未使用的 import
- [x] 4.2 确认 lib.rs 仅保留 mod 声明、FileEntry、AppState、run()
- [x] 4.3 `cargo check` 通过
- [x] 4.4 `cargo build` 通过
- [x] 4.5 前端 `npm run tauri dev` 验证功能正常
