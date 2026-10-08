## Why

Wind 的日志系统（log + fern）在 release 模式下存在多处静默失败路径：文件日志创建失败时日志丢失、`app.emit()` 失败无记录、线程 panic 被 `handle.join()` 吞掉。这些问题导致生产环境排查困难，需要加固。

## What Changes

- **文件日志 fallback**：`setup_logging()` 中日志目录创建或文件打开失败时，fallback 到 stderr 输出，确保日志不丢失
- **emit 失败记录**：关键路径的 `app.emit()` 调用失败时记录 debug 级别日志（directory-changed, transfer-progress, terminal-output 等影响用户体验的事件）
- **线程 panic 记录**：所有 `handle.join()` 调用改为 match 模式，panic 时记录 error 级别日志

## Capabilities

### New Capabilities

无新增 capability。

### Modified Capabilities

- `logging-system`: 增加文件日志失败 fallback 机制，确保 release 模式下日志不静默丢失
- `rust-logging-coverage`: 扩展日志覆盖范围，将 emit 失败和线程 panic 纳入日志记录

## Impact

- **受影响文件**：`src-tauri/src/lib.rs`（setup_logging）、`src-tauri/src/directory_watcher.rs`、`src-tauri/src/file_watcher.rs`、以及 emit 关键路径文件
- **无新依赖**
- **无 breaking change**
- **不影响日志格式、输出目标和默认 log level**
