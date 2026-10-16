## Why

2026-10-16 18:07:03，wind.exe 解压一个 15.8 GiB RAR 归档时进程直接消失：Windows 事件日志为 `Application Error 1000` / `0xc0000409` / WER `P9: 7`（FAST_FAIL_FATAL_APP_EXIT，即进程调用了 abort），而应用自己的日志停在 18:06:26.663 之后 37 秒空白、没有任何错误记录。原因是 release 构建为 GUI 子系统（`main.rs:2`）没有控制台，abort 时 Rust/CRT 只往 stderr 打印，而 `setup_logging` 的 stderr fallback（`lib.rs:112-115`）仅在文件日志创建失败时才挂上，正常情况等于没有接收方。要拿到"是 abort"这个结论，只能事后去查 Windows 事件日志。因此本次先把"看不见"变成"看得见"，再把风险最高的解压移出主进程。

## What Changes

**崩溃可观测性（Phase 1，前置且可独立发布）**
- 注册全局 panic hook，把 panic 消息、位置与回溯写入日志文件，不再只打到无人接收的 stderr。
- release 启动最早期把进程 stderr 重定向到 `logs/wind-crash-YYYY-MM-DD.log`，保证 abort 时 CRT/Rust 往 stderr 写的 `memory allocation of N bytes failed` 之类信息被保留。
- 每 30 秒写一条心跳；在退出事件时写一条明确的 shutdown 记录。于是"日志在 X 时间点后停止且无 shutdown 记录"即可直接判定为异常终止，无需再查事件日志。
- 启动日志补上构建与环境信息（版本、构建时间、日志级别、物理内存、页面文件），并加一条醒目的运行分隔行，使同一日志文件中的多次运行一眼可辨（当前只有 `Wind application starting` 一句话）。

**失败隔离（Phase 2）**
- From: 归档解压在 UI 进程内以 `spawn_blocking` 调用 `archive::extract_all`，一次 abort 即整进程死亡。
- To: 归档解压在独立 worker 进程中执行（复用同一可执行文件 `wind.exe --archive-extract-worker`），父进程捕获其 stdout 进度、stderr 与退出码。
- Reason: `0xc0000409` 是进程级致命终止，只有跨进程边界才能让 UI 存活。
- Impact: 解压路径与进度/取消机制改写，Tauri 命令签名不变（非破坏性）。

**可选配置（仅文档化，不自动执行）**
- 列出通过 WER LocalDumps 注册表（`HKLM\...\Windows Error Reporting\LocalDumps\wind.exe`）产出 minidump 的可行性与代价；因属管理员级系统配置且触及用户红线，本变更不写入注册表。

## Capabilities

### New Capabilities

- `crash-forensics`: 崩溃现场的可观测性——panic 捕获、stderr 落地、崩溃日志工件命名与保留、异常终止判定依据，以及本机 minidump 选项的说明约束。
- `process-isolation`: 归档解压在可监管子进程边界内执行的契约——隔离程度、进度与取消的跨进程传递、子进程崩溃证据保留、密码不经命令行传递、启动失败不静默降级。

### Modified Capabilities

- `logging-system`: 新增 release 下 stderr 必须落地、优雅退出 shutdown 记录与周期性心跳、启动环境信息与运行分隔三项要求；并澄清文件日志失败时的 stderr fallback 现指向已重定向的崩溃日志文件。
- `global-error-capture`: 从"仅前端未捕获异常"扩展为"前后端全链路错误进入同一日志文件"，补齐后端 panic 与致命输出的捕获要求。

## Impact

- Rust：`src-tauri/src/lib.rs`（`setup_logging` / `cleanup_old_logs` / `run`）、`src-tauri/src/main.rs`（入口与 worker 参数分发）、新增崩溃日志与 worker 相关模块、`src-tauri/src/transfer/extract.rs`、`src-tauri/src/transfer/scheduler.rs`、`src-tauri/src/commands/archive_cmd.rs`。
- 依赖：不新增 crate；仅在已存在的 `windows` crate 上启用 `Win32_System_SystemInformation` feature（读取物理内存/页面文件）。
- 日志目录新增 `wind-crash-*.log` 工件，需纳入既有 7 天清理逻辑。
- 前端无 API 变更：`extract_archive` / `extract_enqueue` 与 `transfer-progress` 事件语义保持，前端无感。
- 文档：需记录日志文件布局与异常终止判定方法，以及可选的 WER LocalDumps 配置及其代价。
