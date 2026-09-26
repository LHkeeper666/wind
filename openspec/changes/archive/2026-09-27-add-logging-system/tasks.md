## 1. Rust 依赖与日志初始化

- [x] 1.1 在 `src-tauri/Cargo.toml` 中添加 `log`、`fern`、`chrono` 依赖
- [x] 1.2 在 `src-tauri/src/lib.rs` 中实现 fern 日志初始化（setup_logging 函数，开发模式 stdout，release 模式按日期写文件）
- [x] 1.3 实现 `frontend_log` Tauri 命令，前端通过 IPC 发送日志到 Rust 后端

## 2. 前端日志封装

- [x] 2.1 创建 `src/lib/utils/log.ts`，封装 `logInfo`/`logWarn`/`logError`/`logDebug`，通过 invoke 调用后端 `frontend_log`
- [x] 2.2 在现有前端代码中选 1-2 处 `console.*` 替换为 `logInfo`/`logError` 验证前端日志写入

## 3. 启动时日志清理

- [x] 3.1 在 `src-tauri/src/lib.rs` 的 `setup` 阶段实现日志目录清理逻辑（删除 7 天前的 `wind-YYYY-MM-DD.log` 文件）
- [x] 3.2 处理日志目录不存在的情况（静默跳过）

## 4. Rust 后端 eprintln! 全量迁移

- [x] 4.1 ftp.rs — 30 处 eprintln! → log 宏
- [x] 4.2 commands/ftp_cmd.rs — 35 处 eprintln! → log 宏
- [x] 4.3 commands/file_io.rs — 15 处 eprintln! → log 宏
- [x] 4.4 commands/misc.rs — 3 处 eprintln! → log 宏
- [x] 4.5 commands/directory.rs — 3 处 eprintln! → log 宏
- [x] 4.6 video/mod.rs — 5 处 eprintln! → log 宏
- [x] 4.7 transfer/ftp.rs — 9 处 eprintln! → log 宏
- [x] 4.8 transfer/scheduler.rs — 6 处 eprintln! → log 宏
- [x] 4.9 pdf/mod.rs — 6 处 eprintln! → log 宏
- [x] 4.10 file_watcher.rs — 10 处 eprintln! → log 宏

## 5. 前端 console.* 全量迁移

- [x] 5.1 terminal-manager.ts — 4 处 console.* → log 函数
- [x] 5.2 stores/transfer.ts — 5 处 console.* → log 函数
- [x] 5.3 previewers/MarkdownPreviewer.ts — 9 处 console.* → log 函数
- [x] 5.4 components/PanelLayout.svelte — 9 处 console.* → log 函数
- [x] 5.5 components/DirectoryPanel.svelte — 5 处 console.* → log 函数
- [x] 5.6 其余 15 个文件 — 各 1-3 处 console.* → log 函数

## 6. 验证

- [x] 6.1 `cargo check` 确认 Rust 编译通过（0 错误）
- [x] 6.2 `svelte-check` 确认前端编译通过（0 错误）
- [x] 6.3 确认 eprintln! 残留为 0
- [x] 6.4 确认 console.* 残留为 0
- [x] 6.5 `npm run tauri dev` 验证开发模式日志仅输出到终端
- [x] 6.6 `npm run tauri build` 后运行 release 包，验证日志文件写入 `%LOCALAPPDATA%/Wind/logs/`
- [x] 6.7 验证前端 `logInfo` 调用在日志文件中出现（带 `[frontend]` 标签）
- [x] 6.8 手动创建一个 8 天前日期的假日志文件，重启应用验证自动清理