## Context

`src-tauri/src/lib.rs` 有 2496 行，包含以下领域的 Tauri command 函数：

| 领域 | 行范围 | 函数数量 |
|------|--------|---------|
| 文件夹大小计算 | 30-158 | 3 (calculate_folder_size, cancel_folder_size, walk_dir + helpers) |
| 虚拟根/驱动器列表 | 160-203 | 2 (list_virtual_root, list_drives) |
| 文件元数据 | 237-256 | 2 (get_file_metadata, FileMetadata) |
| 监视器命令 | 258-287 | 4 (start/stop_watch_file, start/stop_watch_directory) |
| 目录读取 | 289-392 | 1 (read_directory) |
| 压缩包命令 | 394-533 | 8 (read_archive_directory, read_archive_file, extract_archive_files, extract_archive, compress_files, archive_delete_entry, archive_rename_entry, archive_add_files, archive_write_file, archive_create_entry) |
| 回收站 | 535-602 | 4 (list_recycle_bin, restore_recycle_items, purge_recycle_items, empty_recycle_bin) |
| 文件操作 | 604-721 | 6 (delete_file, rename_file, batch_rename, create_batch_rename_temp_file, delete_temp_file, create_file) |
| 传输调度器命令 | 723-982 | 10 (transfer_enqueue, transfer_cancel, transfer_cancel_all, scan_transfer_conflicts, scan_ftp_upload/download_conflicts, transfer_reorder, transfer_get/clear_history, transfer_set_ftp/local_slots, transfer_get_slots) |
| 终端命令 | 984-1016 | 4 (terminal_spawn, terminal_input, terminal_resize, terminal_kill) |
| 文件信息 | 1018-1140 | 3 (get_file_info, FileInfo, chrono_like, is_leap) |
| 配置 | 1142-1165 | 2 (read_config, write_config) |
| 文件读写 | 1167-1523 | 7 (open_file, open_with_dialog, read_file, read_file_partial, read_binary_file, read_binary_file_partial, read_image_thumbnail, write_file, decode_text) |
| 搜索 | 1524-1809 | 6 (search_files, cancel_search, check_search_tools, search_with_fd, search_with_rust, is_hidden_search_path) |
| FTP 命令 | 1811-2328 | 13 (ftp_connect, ftp_disconnect, ftp_read_directory, ftp_delete, ftp_rename, ftp_copy, ftp_create_file, ftp_mkdir, ftp_download_folder, ftp_upload_folder, list_ftp_connections, check_ftp_connection + helpers) |
| IME | 2330-2347 | 1 (set_ime_enabled) |
| Shell 执行 | 2349-2379 | 2 (exec_shell_command, detect_bash_path) |

已拆出的独立模块：`ftp.rs`（FtpManager）、`transfer.rs`（TransferScheduler）、`archive/mod.rs`、`terminal/mod.rs`、`file_watcher.rs`、`directory_watcher.rs`、`pdf/`、`video/`、`python_completion.rs`、`app_paths.rs`、`tool_cache.rs`。

## Goals / Non-Goals

**Goals:**
- lib.rs 缩减到 ~100 行：模块声明、共享类型、AppState、run()。
- 每个新模块独立编译，通过 `State<AppState>` 访问共享状态。
- Tauri command 名称和签名完全不变，前端零改动。
- `cargo check` 通过即为正确。

**Non-Goals:**
- 不修改任何 Tauri command 的行为或签名。
- 不引入新的抽象层（如 trait、泛型 command handler）。
- 不重组已拆出的模块（ftp.rs、transfer.rs 等）。
- 不改动共享类型（FileEntry、AppState）的定义位置。

## Decisions

### 1. 模块划分方案

将 lib.rs 中的函数拆分为以下 8 个模块：

| 模块文件 | 职责 | 包含函数 |
|----------|------|---------|
| `commands/directory.rs` | 目录浏览与文件夹大小 | read_directory, list_virtual_root, list_drives, calculate_folder_size, cancel_folder_size, walk_dir, register/unregister/cancel_folder_size_calc, FOLDER_SIZE_CALCS |
| `commands/file_ops.rs` | 文件 CRUD 与重命名 | delete_file, rename_file, batch_rename, create_batch_rename_temp_file, delete_temp_file, create_file, RenameEntry |
| `commands/recycle.rs` | 回收站操作 | list_recycle_bin, restore_recycle_items, purge_recycle_items, empty_recycle_bin, TrashItemDto |
| `commands/search.rs` | 文件搜索 | search_files, cancel_search, check_search_tools, search_with_fd, search_with_rust, is_hidden_search_path, is_rg_available, get_fd_path, SEARCH_CANCELLED, SearchResult |
| `commands/transfer.rs` | 传输调度器命令 | transfer_enqueue, transfer_cancel, transfer_cancel_all, scan_transfer_conflicts, scan_ftp_upload_conflicts, scan_ftp_download_conflicts, transfer_reorder, transfer_get_history, transfer_clear_history, transfer_set_ftp_slots, transfer_set_local_slots, transfer_get_slots |
| `commands/file_io.rs` | 文件读写与缩略图 | read_file, read_file_partial, read_binary_file, read_binary_file_partial, read_image_thumbnail, write_file, open_file, open_with_dialog, decode_text, ImageThumbnail, THUMBNAIL_* 常量 |
| `commands/file_info.rs` | 文件元数据与信息 | get_file_info, get_file_metadata, file_exists, get_home_dir, FileInfo, FileMetadata, chrono_like, is_leap |
| `commands/ftp.rs` | FTP 命令 | ftp_connect, ftp_disconnect, ftp_read_directory, ftp_delete, ftp_rename, ftp_copy, ftp_create_file, ftp_mkdir, ftp_download_folder, ftp_upload_folder, list_ftp_connections, check_ftp_connection, parse_ftp_url, try_list_dir, ftp_copy_move_fallback |

注意：`terminal` 命令（terminal_spawn 等 4 个）和 `archive` 命令（8 个）已经是薄包装层，可以留在 lib.rs 或也拆出去。建议一并拆分以达到 lib.rs ~100 行的目标。

| 模块文件 | 职责 | 包含函数 |
|----------|------|---------|
| `commands/terminal.rs` | 终端命令 | terminal_spawn, terminal_input, terminal_resize, terminal_kill |
| `commands/archive.rs` | 压缩包命令 | read_archive_directory, read_archive_file, extract_archive_files, extract_archive, compress_files, archive_delete_entry, archive_rename_entry, archive_add_files, archive_write_file, archive_create_entry |
| `commands/misc.rs` | IME 与 Shell 执行 | set_ime_enabled, exec_shell_command, detect_bash_path, ShellOutput |
| `commands/config.rs` | 配置读写 | read_config, write_config |

### 2. 目录结构

```
src-tauri/src/
├── lib.rs              (~100 行: mod 声明, AppState, FileEntry, run())
├── commands/
│   ├── mod.rs          (pub mod 声明)
│   ├── directory.rs
│   ├── file_ops.rs
│   ├── file_info.rs
│   ├── file_io.rs
│   ├── recycle.rs
│   ├── search.rs
│   ├── transfer_cmd.rs  (避免与 transfer.rs 模块名冲突)
│   ├── ftp_cmd.rs       (避免与 ftp.rs 模块名冲突)
│   ├── terminal_cmd.rs  (避免与 terminal/mod.rs 冲突)
│   ├── archive_cmd.rs   (避免与 archive/mod.rs 冲突)
│   ├── config.rs
│   └── misc.rs
├── archive/            (已有，不动)
├── ftp.rs              (已有，不动)
├── transfer.rs         (已有，不动)
├── terminal/           (已有，不动)
├── ...                 (其他已有模块不动)
```

### 3. 命名冲突处理

已有模块 `ftp.rs`、`transfer.rs`、`terminal/`、`archive/` 占用了自然名称。新命令模块使用 `_cmd` 后缀或不同名称：
- `commands/ftp_cmd.rs` — FTP Tauri commands（vs `ftp.rs` 的 FtpManager）
- `commands/transfer_cmd.rs` — Transfer Tauri commands（vs `transfer.rs` 的 TransferScheduler）
- `commands/terminal_cmd.rs` — Terminal Tauri commands（vs `terminal/` 的 TerminalManager）
- `commands/archive_cmd.rs` — Archive Tauri commands（vs `archive/` 的核心逻辑）

### 4. 共享类型保留在 lib.rs

`FileEntry` 和 `AppState` 保留在 lib.rs。各模块通过 `use crate::{FileEntry, AppState}` 引入。这避免了循环依赖，因为所有命令模块都依赖 AppState，而 AppState 依赖各子模块的类型。

### 5. 静态变量的归属

- `SEARCH_CANCELLED` → 移入 `commands/search.rs`
- `FOLDER_SIZE_CALCS` → 移入 `commands/directory.rs`
- `THUMBNAIL_*` 常量 → 移入 `commands/file_io.rs`

### 6. generate_handler! 注册

`run()` 函数中的 `generate_handler!` 宏保持集中。命令函数通过 `commands::module_name::function_name` 路径引用，或在 lib.rs 顶部用 `use` 导入。

## Implementation Order

1. 创建 `commands/` 目录和 `mod.rs`
2. 逐个模块迁移：从独立性最高的开始（config、misc、file_info），逐步到有交叉依赖的（directory、ftp_cmd、transfer_cmd）
3. 每迁移一个模块后运行 `cargo check` 验证
4. 最终清理 lib.rs 中未使用的 import
5. `cargo build` 全量验证
