## 1. 文件日志 fallback

- [x] 1.1 修改 `src-tauri/src/lib.rs` 的 `setup_logging()` 函数：当 `create_dir_all` 失败或 `OpenOptions::new().open()` 失败时，dispatch 添加 `std::io::stderr()` chain，确保日志不丢失
- [x] 1.2 验证 release 模式下文件日志正常写入时行为不变

## 2. emit 失败记录

- [x] 2.1 修改 `src-tauri/src/directory_watcher.rs`：`directory-changed` emit 改为 `if let Err(e)` 模式，失败时 `log::debug!`
- [x] 2.2 修改 `src-tauri/src/transfer/mod.rs`：`transfer-complete`、`transfer-cancelled`、`transfer-failed` emit 改为 match 模式
- [x] 2.3 修改 `src-tauri/src/transfer/scheduler.rs`：`transfer-progress`、`transfer-cancelled-batch`、`transfer-queue-updated` emit 改为 match 模式
- [x] 2.4 修改 `src-tauri/src/transfer/local.rs`：`transfer-progress` emit 改为 match 模式
- [x] 2.5 修改 `src-tauri/src/transfer/ftp.rs`：`transfer-progress` emit 改为 match 模式
- [x] 2.6 修改 `src-tauri/src/transfer/extract.rs`：`transfer-progress` emit 改为 match 模式
- [x] 2.7 修改 `src-tauri/src/transfer/conflict.rs`：`transfer-conflict-found` emit 改为 match 模式
- [x] 2.8 修改 `src-tauri/src/commands/transfer_cmd.rs`：`transfer-conflict-*`、`transfer-progress` emit 改为 match 模式
- [x] 2.9 修改 `src-tauri/src/commands/directory.rs`：`directory-changed` emit 改为 match 模式
- [x] 2.10 修改 `src-tauri/src/commands/archive_cmd.rs`：`directory-changed` emit 改为 match 模式
- [x] 2.11 修改 `src-tauri/src/terminal/mod.rs`：`terminal-output`、`terminal-exit` emit 改为 match 模式

## 3. handle.join() panic 记录

- [x] 3.1 修改 `src-tauri/src/directory_watcher.rs`：2 处 `let _ = handle.join()` 改为 match 模式，panic 时 `log::error!`
- [x] 3.2 修改 `src-tauri/src/video/mod.rs`：2 处 `let _ = handle.join()` 改为 match 模式，panic 时 `log::error!`

## 4. 验证

- [x] 4.1 运行 `cargo check` 确认编译通过
- [x] 4.2 运行 `cargo build` 确认构建成功
