# Handoff: Split Rust lib.rs

## 背景

`src-tauri/src/lib.rs` 有 2496 行，是 Wind 项目的"上帝文件"。目标是将其中的 Tauri command 函数按职责拆分为独立模块，使 lib.rs 仅保留模块声明、共享类型和 `run()` 函数。

## 当前状态

- Change 提案已完成：`openspec/changes/split-rust-librs/`
- 尚未开始实施

## 拆分方案概览

将 lib.rs 拆为 **12 个新模块**，全部放在 `src-tauri/src/commands/` 目录下：

```
src-tauri/src/
├── lib.rs                 (~100 行: mod声明, AppState, FileEntry, run())
├── commands/
│   ├── mod.rs             (pub mod 声明)
│   ├── config.rs          (read_config, write_config)
│   ├── misc.rs            (set_ime_enabled, exec_shell_command, detect_bash_path, ShellOutput)
│   ├── file_info.rs       (get_file_info, get_file_metadata, file_exists, get_home_dir, FileInfo, FileMetadata, chrono_like, is_leap)
│   ├── file_io.rs         (read/write/open 文件, 缩略图, decode_text, ImageThumbnail, THUMBNAIL_* 常量)
│   ├── file_ops.rs        (delete_file, rename_file, batch_rename, create_file, RenameEntry, create_batch_rename_temp_file, delete_temp_file)
│   ├── recycle.rs         (list_recycle_bin, restore/purge/empty, TrashItemDto)
│   ├── search.rs          (search_files, cancel_search, check_search_tools, SEARCH_CANCELLED, SearchResult, 搜索辅助函数)
│   ├── directory.rs       (read_directory, list_virtual_root, list_drives, calculate_folder_size, walk_dir, FOLDER_SIZE_CALCS)
│   ├── archive_cmd.rs     (10 个压缩包 commands)
│   ├── terminal_cmd.rs    (terminal_spawn/input/resize/kill)
│   ├── transfer_cmd.rs    (10 个传输调度器 commands)
│   └── ftp_cmd.rs         (13 个 FTP commands + parse_ftp_url, try_list_dir, ftp_copy_move_fallback)
```

## 关键约束

1. **AppState 保留在 lib.rs**：各模块通过 `use crate::{AppState, FileEntry}` 引入
2. **Tauri command 名称不变**：前端零改动，`generate_handler!` 集中在 `run()` 中
3. **命名冲突**：已有 `ftp.rs`/`transfer.rs`/`terminal/`/`archive/` 模块，新命令模块使用 `_cmd` 后缀
4. **静态变量归属**：`SEARCH_CANCELLED` → search.rs，`FOLDER_SIZE_CALCS` → directory.rs，`THUMBNAIL_*` → file_io.rs

## 实施顺序

1. 创建 `commands/` 目录和 `mod.rs`
2. 迁移独立模块（无 AppState 依赖）：config → misc → file_info → file_io → file_ops → recycle
3. 迁移有依赖模块：search → directory → archive_cmd → terminal_cmd → transfer_cmd → ftp_cmd
4. 清理 lib.rs import，验证编译

## 前置条件

- Rust 工具链已配置（USTC mirror）
- `cargo check` 在 `src-tauri/` 下可正常运行

## 验证方式

每迁移一个模块后运行 `cargo check`。全部完成后运行 `cargo build` 和 `npm run tauri dev`。

## 文件行号参考

| 领域 | lib.rs 行范围 | 函数数量 |
|------|--------------|---------|
| 文件夹大小 | 30-158 | 3 + helpers |
| 虚拟根/驱动器 | 160-203 | 2 |
| 文件元数据 | 237-256 | 2 |
| 监视器命令 | 258-287 | 4 |
| 目录读取 | 289-392 | 1 |
| 压缩包命令 | 394-533 | 10 |
| 回收站 | 535-602 | 4 |
| 文件操作 | 604-721 | 6 |
| 传输调度器 | 723-982 | 10 |
| 终端命令 | 984-1016 | 4 |
| 文件信息 | 1018-1140 | 3 + helpers |
| 配置 | 1142-1165 | 2 |
| 文件读写 | 1167-1523 | 8 + helpers |
| 搜索 | 1524-1809 | 6 + helpers |
| FTP 命令 | 1811-2328 | 13 + helpers |
| IME | 2330-2347 | 1 |
| Shell 执行 | 2349-2379 | 2 |
| run() | 2381-2495 | 1 |

## 共享类型（保留在 lib.rs）

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    name: String, path: String, is_dir: bool, size: Option<u64>,
    is_hidden: bool, modified: Option<u64>, created: Option<u64>,
    children: Option<Vec<FileEntry>>,
}

struct AppState {
    terminal: terminal::TerminalManager,
    file_watcher: Mutex<file_watcher::FileWatcher>,
    directory_watcher: Mutex<directory_watcher::DirectoryWatcher>,
    ftp_manager: Arc<TokioMutex<ftp::FtpManager>>,
    transfer_scheduler: Arc<TokioMutex<transfer::TransferScheduler>>,
}
```

## 典型模块模板

每个命令模块的标准结构：

```rust
// src-tauri/src/commands/config.rs
use crate::AppState;
use std::fs;
use tauri::State;

#[tauri::command]
pub fn read_config() -> Result<serde_json::Value, String> {
    // ... 从 lib.rs 原样搬过来
}

#[tauri::command]
pub fn write_config(options: serde_json::Value) -> Result<(), String> {
    // ... 从 lib.rs 原样搬过来
}
```

lib.rs 中注册方式：

```rust
// lib.rs run()
.invoke_handler(tauri::generate_handler![
    // ...
    commands::config::read_config,
    commands::config::write_config,
    // ...
])
```
