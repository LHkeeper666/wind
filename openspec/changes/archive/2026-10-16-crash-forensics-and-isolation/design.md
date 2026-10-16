## Context

**事故现场（2026-10-16）**：15.8 GiB RAR 解压中，进程在 18:07:03 以 `0xc0000409`（`FAST_FAIL_FATAL_APP_EXIT`，WER `P9: 7`）终止。证据链只能靠外部工具拼凑：Windows 事件日志给出异常码，应用日志给出"最后活跃时间"。应用自身的日志没有任何崩溃线索。

**为什么看不见**：

| 观察 | 位置 | 后果 |
|---|---|---|
| release 为 GUI 子系统，无控制台 | `src-tauri/src/main.rs:2` | stderr 没有接收方 |
| release 分支只挂文件输出 | `lib.rs:56-100` | 日志正常，但崩溃信息不走 `log` 宏 |
| stderr fallback 仅在"文件日志创建失败"时挂载 | `lib.rs:112-115` | 正常运行路径上 fallback 不生效 |
| 无 panic hook | 全局 | panic 默认只写 stderr 并 unwind/abort |
| 无退出/心跳记录 | `lib.rs:171-176` | 无法区分"崩溃"与"用户关窗" |

**当前解压路径（隔离对象）**：

```
commands/archive_cmd.rs::extract_archive / extract_enqueue
  → transfer/scheduler.rs::enqueue_extract（extract_max_slots = 2）
    → transfer/extract.rs::execute_extract
      → tokio::task::spawn_blocking
        → archive::extract_all / archive::extract_files
          → archive/{zip,rar,seven_z,tar}.rs（unrar / zip / sevenz-rust 为链接库，非外部 CLI）
```

`extract.rs` 已有一个内存采样线程（每 5s 记录 `working_set` / `pagefile`），说明内存问题已在观察范围内；但采样线程与解压同进程，abort 时它一起消失。

**约束**：
- 不新增 crate 依赖，不引入宕机上报服务（用户红线 + 非目标）。
- 不修 RAR 解压本身的内存问题（属 `stream-rar-extraction`）。
- 归档使用链接库而非外部 `unrar.exe` / `7z.exe`，因此"改用外部 CLI 解压"会改变支持面，不作为主要路线。

## Goals / Non-Goals

**Goals:**
- 任何一次异常终止都能在应用自己的日志里被判定，无需依赖 Windows 事件日志。
- 归档解压进程崩溃时，UI 进程存活，且日志中有明确归因（是哪个任务、以什么退出码、最后一条输出是什么）。
- Phase 1（可观测性）可独立发布并被验证。

**Non-Goals:**
- 完全消除解压崩溃（目标是"崩了 UI 还在且有归因"）。
- 修复解压的内存占用本身。
- 把归档的读取、列表、密码校验也移出主进程（本变更只隔离解压写入路径）。
- 自动配置 WER LocalDumps 或任何系统级注册表项。

## Decisions

### D1. Phase 1（可观测性）作为独立交付，先于隔离落地

**选择**：先交付崩溃可观测性，再做进程隔离。

**理由**：隔离本身会产生"子进程为何死"的新问题，如果没有 Phase 1，子进程的 abort 依然不可见——只会从"主进程静默消失"变成"任务莫名失败"。Phase 1 也能独立验证（人为 panic / `process::abort()`）。

**替代方案**：只做隔离，不做可观测性 → 拒绝，因为无法验证隔离是否真的生效，也无法归因。

### D2. stderr 重定向用 `SetStdHandle(STD_ERROR_HANDLE, ...)`，在 `run()` 最早期执行

**选择**：release 下，在 `run()` 的第一条语句之前打开 `logs/wind-crash-YYYY-MM-DD.log`（`FILE_APPEND_DATA` + 可继承句柄），调用 `windows` crate 的 `SetStdHandle` 覆盖 `STD_ERROR_HANDLE`；dev 构建跳过。

**理由**：
- Rust std 与 MSVC CRT 都是通过 `GetStdHandle(STD_ERROR_HANDLE)` 解析 stderr，且**首次使用时缓存**句柄。GUI 子系统下该句柄默认为空/无效，缓存后即使再设置也无用。所以必须"在第一次 stderr 写入之前"完成。
- Rust 的 `rtabort`（分配失败路径）与 CRT 的 `_NMSG_WRITE`（`memory allocation of N bytes failed`）走的是低层 `write`，不经缓冲，所以落盘是即时的。

**约束与代价**：句柄在进程生命周期内固定，因此崩溃日志**不在运行中轮转**，跨日长跑会持续追加到启动那天命名的文件。接受此代价（文件名带启动日期即可辨识）。

**替代方案**：
- 只依赖 `log` 宏 + panic hook → 拒绝：abort 路径根本不经过 `log` 宏，正是本次事故的核心遗漏。
- `freopen` / `_dup2` → 等价但需要 C 运行时调用点，`SetStdHandle` 更直接且已有 `windows` crate。

### D3. panic hook：写日志 + 保留默认行为

**选择**：`std::panic::set_hook` 中构造 `std::backtrace::Backtrace::force_capture()`（仅当 `RUST_BACKTRACE` 或 `RUST_LIB_BACKTRACE` 置位，避免默认 release 下无符号表的开销），以 `log::error!(target: "crash", ...)` 写出 panic 消息、文件、行号与回溯；随后调用默认 hook（或在 unwind 后让既有 panic 策略生效）以保持退出码语义不变。

**理由**：hook 只增加"落盘"能力，不改变 panic 的既有行为（unwind/abort 策略、退出码），避免引入行为回归。

**代价**：hook 中的所有 I/O 在 panic 现场执行，若文件写入阻塞会拖慢退出；因此只做单次 `writeln!`，不做格式化重活。

### D4. 心跳与 shutdown 记录用"存在性"判定异常终止，而非时间差阈值

**选择**：
- 心跳：独立线程每 30s 写 `log::info!(target: "crash", "[heartbeat] alive")`（不依赖 tokio runtime，避免 runtime 崩溃时心跳一并停止）。
- 退出记录：把 `.run(generate_context!())` 改为 `.build(...)?.run(|_app, event| ...)`，在 `RunEvent::Exit` 写 `[shutdown] graceful exit`。

**判定规则**：一次运行区间内**没有** `[shutdown]` 记录 ⇒ 异常终止。

**理由**：用"shutdown 记录缺失"这种结构性证据，避免了对心跳间隔做启发式阈值判断（心跳也会因系统休眠/挂起而暂停，时间差不可靠）。

**代价**：日志体积增加（心跳约 2880 行/天）。缓解：心跳为单行短文本，且日志目录已有 7 天清理。

**替代方案**：用"最后一条日志距今 > N 秒"判定 → 拒绝，休眠/长任务/空闲会误报。

### D5. 启动 banner 需要 `windows` crate 的 `Win32_System_SystemInformation` feature

**选择**：用 `GlobalMemoryStatusEx` 取物理内存与页面文件；版本/构建时间用 `env!("CARGO_PKG_VERSION")` 与构建期注入的时间常量；日志级别从与 `setup_logging` 相同的解析结果复用（抽出一个 `resolve_log_level()` 供两处共用）。

**理由**：`GlobalMemoryStatusEx` 在 `Win32_System_SystemInformation` 下，当前 `Cargo.toml` 未启用该 feature。这是既有 crate 的 feature 开关，**不是新增依赖**，符合"不引入全局依赖"的约束。

**替代方案**：读注册表 `HKLM\HARDWARE\RESOURCEMAP`（`Win32_System_Registry` 已启用）→ 拒绝，过于 hacky 且返回值语义不稳定。

### D6. 隔离路线选型：独立子进程（路线 a），复用同一可执行文件

**选择**：`wind.exe --archive-extract-worker`。父进程用 `tool_cache::background_command`（已带 `CREATE_NO_WINDOW`）派生自身，管道连接 stdin/stdout/stderr。

**理由**：
- `0xc0000409` 是进程级致命终止，**在进程内无论怎样加固都不可能让 UI 存活**，路线 (b)（留在主进程但确保 abort 前有落盘证据）只解决"可观测"，不解决"隔离"，因此 (b) 只能作为 Phase 1 的一部分，不能替代隔离。
- 复用同一可执行文件 → 无需 sidecar 打包配置、无额外构建产物、共享同一份 `archive` 模块代码与 CRC/编码处理逻辑。
- 切换到外部 `unrar.exe` / `7z.exe` → 会偏离现有链接库行为（编码检测、密码处理、部分解压），且引入外部二进制可用性问题，拒绝。

**替代方案（记录取舍）**：独立 sidecar 二进制 → 需 `tauri.conf.json` 的 `externalBin` 与构建脚本改动，收益与自派生相同而成本更高。

### D7. 跨进程契约：JSON Lines over stdio

**方向 1 — 父 → 子（stdin，单行 JSON 后关闭写入）**：
```json
{"archive_path":"...","dest_dir":"...","password":"...","internal_paths":null,"skip_rel_paths":[]}
```
密码只走 stdin，**绝不进 argv**（argv 对同机其他进程可见，见 `process-isolation` 要求）。

**方向 2 — 子 → 父（stdout，逐行 JSON）**：
```json
{"event":"entry","name":"...","size":123}
{"event":"progress","bytes_done":123,"total":456}
{"event":"done","bytes":789}
{"event":"error","message":"..."}
```
父进程解析后**复用现有 `transfer-progress` 事件形状**（`id`/`batch_id`/`op_type`/`bytes_done`/`total_bytes`/`entry_name`/`entry_size`），保持 100ms 节流与"条目边界直发"两条既有语义。**前端零改动**。

**方向 3 — 子进程 stderr（管道）**：父进程用独立 reader 任务逐行读取并 `log::warn!(target: "crash", "[extract-worker] {}", line)`。

**代价**：
- 进度要序列化/反序列化，但节流上限 100ms/次，开销相对解压本身可忽略。
- 需要一个常驻 drain 任务：**若父进程不持续读取管道，子进程会在写 stdout 时阻塞而表现为"卡住"**。这是本设计最需要注意的实现陷阱。
- 子进程的"当前条目"改动需要新增一条 `entry` 消息（现有的 `on_entry` 回调语义）。

### D8. 取消：先协商，超时强杀

**选择**：父进程向子进程 stdin 写一行 `{"cmd":"cancel"}`；worker 在既有的 `ExtractProgress` 回调链路上检查取消标志（等价于现有 `cancel_flag: Arc<AtomicBool>` 的语义），返回错误退出。父进程等待 N 秒（建议 5s）后 `kill()`。

**理由**：协商取消能让 worker 完成当前条目的收尾并正常写日志；超时强杀保证"取消"在解压卡死（或正是本次的内存失控场景）时依然生效。

**代价**：需要把进程内的 `Arc<AtomicBool>` 语义映射为"stdin 消息 + 本地标志"两套路径，实现上多一层。

### D9. 子进程崩溃归因：退出码 + stderr 尾行

**选择**：父进程在 `child.wait()` 后检查 `ExitStatus::code()`。Windows 下异常终止的 `code()` 返回原始退出码的 `i32`（`0xC0000409` → `-1073740791`）。据此写两条不同级别的日志：
- 正常非零：`log::warn!("[extract-worker] task {} exited with code {}", ...)`
- 异常终止码（`0xC0000005` / `0xC0000409` / `0xC00000FD` 等）：`log::error!("[extract-worker] task {} terminated abnormally: 0x{:08X}", ...)`

并把该任务标记为失败，UI 通过既有 `transfer-progress` 的失败状态呈现。

**理由**：这一条是"日志里有明确归因"的落点——崩溃码 + 崩溃前的 stderr 尾行足以定位。

### D10. 子进程启动失败时不静默回退

**选择**：若 worker 无法派生（权限、镜像损坏、句柄耗尽），任务**显式失败**并记录 `log::error!`，**不回退到主进程内解压**。

**理由**：静默回退会在用户不知情的时候把 UI 重新暴露在 abort 风险下，也让"隔离是否生效"无法从日志判断。

### D11. 保留进程内路径以便回滚

**选择**：保留 `transfer/extract.rs` 中的进程内实现，用函数级开关（例如 `execute_extract_in_process`）保留下来，默认走 worker。

**理由**：隔离改动面大，出现"所有解压都失败"这类问题时需要一行回滚。且这一开关同时是验证手段（对比两条路径的进度/取消行为是否一致）。

### D12. WER LocalDumps 仅文档化

**选择**：在文档中给出注册表位置、`DumpFolder`/`DumpType` 的含义与取舍，**不自动写入**。

**取舍**：
- 收益：minidump 能给出 abort 时刻的原生调用栈，是唯一能直接回答"哪次分配失败/哪里的 `std::terminate`"的手段。
- 代价：`HKLM` 写入需管理员权限（用户红线）；对所有用户全局生效；dump 文件可能数 GB；属于"改机器配置"而非"改应用"。
- 因此留给用户按需手动启用。

## Risks / Trade-offs

- **[重定向时机]** 若重定向晚于任何一次 stderr 写入（std 句柄缓存），崩溃日志会静默为空 → 缓解：重定向作为 `run()`/`main()` 的最早语句；重定向后立即写一条确认日志；在验证阶段用 `process::abort()` 实测崩溃日志非空。
- **[崩溃日志不轮转]** 句柄固定，跨日长跑持续追加 → 接受；文件名含启动日期。不做 `ReopenFile` 复杂度。
- **[日志体积]** 心跳 30s 一条 → 约 2880 行/天；缓解：单行短文本 + 既有 7 天清理；周期可在实现时定义为常量便于调整。
- **[管道阻塞]** 父进程未持续读 stdout/stderr 会让子进程假死 → 缓解：stderr/stdout 各一个常驻 reader 任务，先于 `wait()` 启动；验证阶段用大归档 + 高频条目确认无卡顿。
- **[并发隔离的内存代价]** 现有 `extract_max_slots = 2`，隔离后是 2 个独立地址空间，系统总内存峰值可能更高（每个进程不再共享堆，但也不再互相拖累）。→ 接受：这正是换取存活性的代价；若实测峰值过高，可下调并发（配置项 `transfer_set_extract_slots` 已存在）。
- **[残留风险：非解压路径仍在主进程]** `list_entries` / `read_file_bytes`（归档内文件预览）仍可能为大条目分配大内存，理论上同样可 abort。→ 本变更不覆盖，记录为后续可选隔离项（见 Open Questions）。
- **[取消语义差异]** 进程内取消是即时打断；跨进程协商取消的响应粒度是"下一个条目/下一个进度回调"。→ 接受；超时强杀兜底。
- **[worker 重复初始化]** 若 worker 入口未在 `tauri::Builder` 之前返回，会尝试建窗/初始化 Tauri 而失败或闪窗 → 缓解：参数分发放在 `main()`，worker 分支直接返回，不进入 `run()`；验证阶段确认 worker 进程无窗口。

## Migration Plan

1. **Phase 1 独立发布**：不改变任何解压行为，只增加日志/崩溃工件。风险低，先落地并观察一次真实运行的日志（确认 banner / 心跳 / shutdown 记录形态）。
2. **Phase 2 发布**：解压切到 worker；`transfer-progress` 事件形状不变，前端无感。
3. **回滚**：Phase 2 出问题时切回进程内实现（D11 的开关），无需回滚 Phase 1。
4. **无数据迁移**：不涉及配置格式、数据库或用户数据；仅日志目录新增 `wind-crash-*.log`。
5. **验证口径**：`cargo check` + `npx svelte-check`（前端未改，作为回归确认）+ 手动场景（见 tasks.md 验证组）。

## Open Questions

- 心跳周期是否要暴露为配置（当前计划为常量 30s）？
- 是否需要在后续把归档**读取/列表**也移入 worker（当前残留风险项）？本变更按非目标处理，不作为交付内容。
- WER LocalDumps 是否只在文档中给出，还是同时提供一个"打印当前注册表状态"的只读诊断命令？（只读命令可行，但会新增 Tauri 命令面，倾向不做。）
