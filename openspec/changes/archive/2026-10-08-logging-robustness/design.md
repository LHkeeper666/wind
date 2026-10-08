## Context

Wind 使用 `fern` + `log` 作为 Rust 日志系统。`setup_logging()` 在 release 模式下将日志写入按日期轮转的文件。当前代码中存在三类静默失败：

1. **文件日志路径**：`let _ = std::fs::create_dir_all` 和 `OpenOptions::new().open()` 失败时无 fallback，日志完全丢失
2. **事件发射**：65+ 处 `let _ = app.emit(...)` 调用，失败完全静默
3. **线程 join**：5 处 `let _ = handle.join()`，线程 panic 被吞掉

## Goals / Non-Goals

**Goals:**
- 文件日志创建失败时 fallback 到 stderr，确保日志不丢失
- 关键路径 emit 失败时记录 debug 级别日志
- 线程 panic 通过 `handle.join()` 记录 error 级别日志

**Non-Goals:**
- 不改日志格式和输出目标
- 不改默认 log level
- 不改 `Box::leak` 设计（fern 闭包的已知模式）
- 不加新依赖
- 不改所有 65 处 emit，只改关键路径

## Decisions

### 1. 文件日志 fallback 策略

**决策**：文件日志失败时，fallback 到 stderr 输出。不在 fern dispatch 内部做重试。

**理由**：
- fern dispatch 是一次性配置，闭包捕获后无法动态切换输出目标
- stderr 在 release 模式下会被 Tauri 捕获到日志文件（Windows 上 OutputDebugString）
- 重试逻辑会增加闭包复杂度，且目录权限问题通常不会自行恢复

**实现**：
- `setup_logging()` 中，如果 `create_dir_all` 失败或首次 `OpenOptions` 失败，dispatch 添加 `std::io::stderr()` chain
- 保留现有文件写入逻辑不变，文件可写时自动恢复正常

### 2. emit 失败记录范围

**决策**：只改影响用户体验的关键事件，用 `debug!` 级别。

**关键事件列表**（基于代码审查）：
- `directory-changed` — 目录刷新
- `transfer-progress` / `transfer-complete` / `transfer-error` — 文件传输
- `terminal-output` / `terminal-exit` — 终端输出
- `file-watcher-event` — 文件变更通知

**理由**：
- debug 级别在默认配置下不会输出，避免日志膨胀
- 关键路径的 emit 失败意味着 UI 不更新，值得记录

### 3. handle.join() panic 记录

**决策**：所有 `let _ = handle.join()` 改为 match 模式，panic 记录 error 级别。

**理由**：
- 线程 panic 是严重错误，必须记录
- error 级别确保默认配置下可见

## Risks / Trade-offs

- **[Risk] stderr fallback 可能产生重复日志** → 如果 Tauri 已经将 stderr 重定向到文件，可能会看到重复。但这是比日志丢失更好的权衡。
- **[Risk] emit debug 日志在高频事件中可能过多** → 使用 debug 级别，用户需要显式设置 `RUST_LOG=wind=debug` 才会看到。高频事件（如 terminal-output）已经是 debug 级别，不会额外增加。
- **[Trade-off] 不在 fern 闭包内做重试** → 简单可靠，但如果目录后来变得可写，需要重启应用才能恢复文件日志。
