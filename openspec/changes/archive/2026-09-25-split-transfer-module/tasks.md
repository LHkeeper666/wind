## 1. 创建模块结构

- [x] 1.1 创建 `src-tauri/src/transfer/` 目录，将 `transfer.rs` 移动为 `transfer/mod.rs`
- [x] 1.2 创建空的子模块文件：`scheduler.rs`、`ftp.rs`、`local.rs`、`conflict.rs`、`helpers.rs`
- [x] 1.3 在 `mod.rs` 中添加 `mod scheduler; mod ftp; mod local; mod conflict; mod helpers;` 声明

## 2. 迁移类型定义到 mod.rs

- [x] 2.1 将 `TransferType`、`TaskStatus`、`TransferTask`、`TransferHistoryRecord`、`EnqueueTask`、`ActiveTask` 定义保留在 `mod.rs`
- [x] 2.2 将 `execute_transfer` 分发函数保留在 `mod.rs`
- [x] 2.3 在 `mod.rs` 中添加 `pub use` re-export 确保外部 API 不变

## 3. 迁移调度器到 scheduler.rs

- [x] 3.1 将 `TransferScheduler` 结构体定义和 `new()` 迁移到 `scheduler.rs`
- [x] 3.2 迁移队列管理方法：`enqueue`、`cancel`、`cancel_all`、`reorder`
- [x] 3.3 迁移调度方法：`dispatch_pending`、`dispatch_queued`、`cleanup_finished`
- [x] 3.4 迁移槽位管理方法：`can_take_slot`、`take_slot`、`free_slot_direct`
- [x] 3.5 迁移历史记录方法：`load_history`、`save_to_history`、`cleanup_history`、`persist_history`、`get_history`、`clear_history`
- [x] 3.6 迁移配置方法：`set_ftp_max_slots`、`set_local_max_slots`、`get_slot_config`
- [x] 3.7 迁移 FTP folder 入队方法：`enqueue_ftp_folder_download`、`enqueue_ftp_folder_upload`、`save_batch_meta`
- [x] 3.8 迁移事件发射方法：`emit_progress`、`emit_cancelled`
- [x] 3.9 迁移 ID 生成方法：`next_id`、`next_batch_id`
- [x] 3.10 在 `scheduler.rs` 中添加必要的 `use` 和 `super::` 引用

## 4. 迁移 FTP 执行器到 ftp.rs

- [x] 4.1 迁移 `execute_ftp_download` 函数
- [x] 4.2 迁移 `execute_ftp_upload` 函数
- [x] 4.3 迁移 `execute_ftp_delete` 函数
- [x] 4.4 迁移 `ProgressAsyncReader` 结构体和 `AsyncRead` 实现
- [x] 4.5 迁移 `ensure_remote_dir` 辅助函数
- [x] 4.6 在 `ftp.rs` 中添加必要的 `use` 和 `super::` 引用

## 5. 迁移本地执行器到 local.rs

- [x] 5.1 迁移 `execute_local_copy` 函数
- [x] 5.2 迁移 `execute_local_delete` 函数
- [x] 5.3 迁移 `local_copy_blocking` 函数
- [x] 5.4 迁移 `copy_file_with_progress`、`copy_dir_with_progress` 函数
- [x] 5.5 迁移 `delete_with_progress`、`delete_with_progress_filtered`、`cleanup_cancelled_dir` 函数
- [x] 5.6 在 `local.rs` 中添加必要的 `use` 和 `super::` 引用

## 6. 迁移冲突扫描到 conflict.rs

- [x] 6.1 迁移 `scan_dir_conflicts` 函数
- [x] 6.2 迁移 `walk_local_dir` 函数
- [x] 6.3 在 `conflict.rs` 中添加必要的 `use` 引用

## 7. 迁移辅助函数到 helpers.rs

- [x] 7.1 迁移 `now_secs`、`dir_size`、`collect_dir_size` 函数
- [x] 7.2 迁移 `extract_ftp_conn`、`check_cancelled` 函数
- [x] 7.3 在 `helpers.rs` 中添加必要的 `use` 引用

## 8. 更新 mod.rs 的 re-export

- [x] 8.1 在 `mod.rs` 中添加 `pub use` 导出所有 public 类型和函数
- [x] 8.2 确保 `scheduler` 模块的 public 方法通过 `TransferScheduler` 可访问

## 9. 验证

- [x] 9.1 运行 `cargo check` 确认无编译错误
- [x] 9.2 运行 `cargo build` 确认构建成功
- [x] 9.3 验证 `commands/transfer_cmd.rs` 无需修改 import 路径