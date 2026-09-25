## Why

`transfer.rs` 已膨胀至 1455 行，混合了调度器、队列管理、槽位管理、历史记录持久化、FTP 传输执行、本地传输执行、冲突扫描、辅助函数等 10 种职责。可读性和可维护性下降，每次修改传输逻辑都需要在大文件中定位。项目已有 `archive/`、`commands/`、`terminal/` 等目录模块先例，`transfer` 应遵循相同模式。

## What Changes

- 将 `transfer.rs` 拆分为 `transfer/` 目录模块，包含以下子模块：
  - `mod.rs` — 类型定义（TransferType, TaskStatus, TransferTask, EnqueueTask 等）和 re-export
  - `scheduler.rs` — TransferScheduler 结构体、队列管理（enqueue/cancel/reorder/dispatch）、槽位管理、历史记录持久化
  - `ftp.rs` — FTP 传输执行器（execute_ftp_download, execute_ftp_upload, execute_ftp_delete）及相关辅助（ProgressAsyncReader, ensure_remote_dir）
  - `local.rs` — 本地传输执行器（execute_local_copy, execute_local_delete）及相关辅助（copy_file_with_progress, copy_dir_with_progress, delete_with_progress 等）
  - `conflict.rs` — 冲突扫描（scan_dir_conflicts, walk_local_dir）
  - `helpers.rs` — 通用辅助（now_secs, dir_size, extract_ftp_conn, check_cancelled）
- `execute_transfer` 分发函数放在 `mod.rs` 中，因为它桥接 scheduler 和各执行器
- `commands/transfer_cmd.rs` 的 import 路径更新为 `crate::transfer::*`

## Capabilities

### New Capabilities

无新增功能，纯重构。

### Modified Capabilities

无需求变更。

## Impact

- **Affected code**: `src-tauri/src/transfer.rs` → `src-tauri/src/transfer/mod.rs` + 5 个子模块
- **Affected imports**: `commands/transfer_cmd.rs`、`lib.rs` 中的 `use crate::transfer::*` 路径不变（模块名相同）
- **Risk**: 低 — 纯结构重组，不改变任何 public API 或行为
- **Verification**: `cargo check` 通过即可确认无编译错误
