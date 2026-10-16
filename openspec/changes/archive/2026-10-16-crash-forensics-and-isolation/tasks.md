## 1. Phase 1 — 崩溃可观测性基础设施

- [x] 1.1 从 `setup_logging` 抽出 `resolve_log_level()`，供日志初始化与启动 banner 共用同一解析结果
- [x] 1.2 新增崩溃日志路径 helper：日志目录（复用 `app_handle.path().app_local_data_dir().join("logs")`）+ `wind-crash-YYYY-MM-DD.log` 命名
- [x] 1.3 在 `run()`/`main()` 最早期实现 release 下的 stderr 重定向：以 `FILE_APPEND_DATA` 打开崩溃日志并 `SetStdHandle(STD_ERROR_HANDLE, handle)`；dev 构建跳过；重定向后写一条确认日志（含路径），失败时记录原因但不阻断启动
- [x] 1.4 注册全局 panic hook：写 panic 消息 + 文件 + 行号到 `log::error!(target: "crash", ...)`；仅在 `RUST_BACKTRACE`/`RUST_LIB_BACKTRACE` 置位时 `Backtrace::force_capture()`；完成后不改变原 panic 退出语义
- [x] 1.5 在 `Cargo.toml` 的 `windows` crate 启用 `Win32_System_SystemInformation` feature（非新增依赖）
- [x] 1.6 启动 banner：版本（`env!("CARGO_PKG_VERSION")`）、构建时间、生效日志级别、物理内存与页面文件（`GlobalMemoryStatusEx`），并以醒目分隔行开始本次运行区段，替换现有单行 `Wind application starting`
- [x] 1.7 心跳线程：独立线程每 30s 写一条 `[heartbeat] alive`（不依赖 tokio runtime），周期定义为常量
- [x] 1.8 优雅退出记录：把 `.run(generate_context!())` 改为 `.build(...)?.run(|_app, event| ...)`，在 `RunEvent::Exit` 写 `[shutdown] graceful exit`
- [x] 1.9 扩展 `cleanup_old_logs` 覆盖 `wind-crash-YYYY-MM-DD.log`（与 `wind-*.log` 相同的 7 天保留策略）

## 2. Phase 1 — 验证

- [x] 2.1 `cargo check` 通过（`src-tauri/`）
- [ ] 2.2 人为触发 panic（临时调用点），确认日志文件出现含位置与回溯的错误条目，且进程退出码语义未变
- [ ] 2.3 临时调用 `std::process::abort()` 验证崩溃日志文件非空（证明重定向时机正确、未被 std 句柄缓存吞掉）
- [ ] 2.4 release 构建启动一次，确认启动 banner 字段齐全、心跳按 30s 出现、正常关窗后有 shutdown 记录
- [ ] 2.5 同一天启动两次，确认日志文件中两次运行可由分隔行区分

## 3. Phase 2 — 解压隔离边界

- [x] 3.1 `main.rs` 解析 `--archive-extract-worker` 参数并分发到 worker 入口；worker 分支 MUST 在进入 `tauri::Builder` 之前返回，不初始化 UI
- [x] 3.2 worker 入口：从 stdin 读取单行 JSON 任务（`archive_path`/`dest_dir`/`password`/`internal_paths`/`skip_rel_paths`），调用 `archive::extract_all` 或 `archive::extract_files`
- [x] 3.3 worker 通过 stdout 输出逐行 JSON 事件（`entry`/`progress`/`done`/`error`），复用现有 100ms 节流与"条目边界直发"语义
- [x] 3.4 worker 侧注册 panic hook，并把诊断输出落到带 `[extract-worker]` 标记的日志
- [x] 3.5 父进程派生 worker：复用 `tool_cache::background_command`（`CREATE_NO_WINDOW`），管道连接 stdin/stdout/stderr
- [x] 3.6 父进程常驻 reader 任务：解析 stdout 进度 → 复用现有 `transfer-progress` 事件形状发往 UI；逐行转发 stderr → 日志（带 worker 标记）；reader MUST 先于 `wait()` 启动
- [x] 3.7 取消契约：向 stdin 写 `{"cmd":"cancel"}`；worker 在进度回调链路上响应；父进程等待 5s 未退出则强制终止
- [x] 3.8 退出码归因：`child.wait()` 后按 `ExitStatus::code()` 区分普通非零（warn）与异常终止码（error，如 `0xC0000409`）
- [x] 3.9 密码只经 stdin 传递，确认子进程命令行不含密码明文
- [x] 3.10 子进程派生失败时任务显式失败并记录 error，不回退到主进程内解压
- [x] 3.11 保留进程内实现（如 `execute_extract_in_process`）作为一行回滚开关，默认走 worker
- [x] 3.12 将 `transfer/extract.rs` 的内存采样线程迁入 worker（其 stderr 输出经父进程转发）

## 4. Phase 2 — 验证

- [x] 4.1 `cargo check` 与 `npx svelte-check` 通过（前端无改动，做回归确认）
- [ ] 4.2 解压一个大归档，确认进度条、当前条目、完成状态与隔离前一致
- [ ] 4.3 解压过程中取消，确认协商取消生效；再造"取消后卡死"场景确认超时强杀生效
- [ ] 4.4 在 worker 中临时注入 `abort()`，确认 UI 存活、任务被标记失败、日志含退出码与 stderr 尾行
- [ ] 4.5 解压带密码归档，确认子进程命令行无密码明文
- [ ] 4.6 确认 worker 进程无可见窗口，且主进程日志能判定该次解压运行在隔离边界内

## 5. 文档

- [x] 5.1 记录日志文件布局：`wind-*.log` 与 `wind-crash-*.log` 的用途与差异
- [x] 5.2 记录异常终止判定方法：运行区段 + 心跳 + shutdown 记录缺失的组合判据
- [x] 5.3 记录可选 WER LocalDumps 配置：注册表位置、`DumpFolder`/`DumpType` 含义，以及管理员权限/全机生效/磁盘占用三项代价；明确应用不自动写入注册表
- [x] 5.4 说明解压隔离边界与回滚开关，便于后续排查与临时回退
