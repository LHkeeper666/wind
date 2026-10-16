## Context

RAR 解压当前有两条实现（`src-tauri/src/archive/rar.rs`）：

- `extract_all`（第 186-251 行）：`open_for_processing()` → 循环 `read_header()` → 对每个条目调用 `header.read()`
- `extract_files`（第 117-184 行）：同上，但只处理 `internal_paths` 命中的条目

两者都用 `header.read()` 拿到 `(Vec<u8>, OpenArchive)`，然后按 65536 字节分块写盘。

`header.read()` 在 unrar crate 0.5.8 里走 `ReadToVec` ProcessMode（`open_archive.rs:478-485`），其实现是 `my.extend_from_slice(other)`——**条目的全部解压数据先堆进一个 `Vec<u8>`，再写盘**。这是本次崩溃的内存峰值来源：归档内含 4 GiB（`start0` / `start1` 恰好 2³²）、3.6 GiB（`en.lang`）、1.3 GiB（`Master.pck`）等条目，`Vec` 倍增扩容时从 2 GiB 扩到 4 GiB 需同时持有新旧两块，瞬时峰值约 6 GiB。

崩溃现场：`0xc0000409` + WER `P9=7`（`FAST_FAIL_FATAL_APP_EXIT`）= `abort()`。Rust 的 `handle_alloc_error` 在分配失败时正是走这条路。因为 release 构建是 GUI 子系统（`src-tauri/src/main.rs:2`）且 `setup_logging()` 在 release 下只写文件不写 stderr，abort 时往 stderr 打印的失败信息被丢弃——表现为"应用静默退出、日志无任何错误"。

`unrar_sys` 把官方 UnRAR DLL 源码（`dll.cpp` 等）静态编译进 wind.exe，其中存在进程级全局状态（`ErrHandler` 单例、`crc.cpp` 的 `crc_tables`、`rijndael.cpp` 的 `T1..U4` 懒初始化表、`blake2s_sse.cpp` 注释明说 SSE 静态初始化非线程安全），而当前所有入口都可被并发调用，无任何串行化。

约束：
- 不能换归档库（`unrar` 是项目既定依赖，且 RAR 分卷/加密/Unicode 文件名等行为已被 `rar-archive-support` 规范锁定）
- `transfer-progress` 事件契约被前端 Transfer Manager 消费，必须保持兼容
- 用户红线：不新增全局依赖

## Goals / Non-Goals

**Goals:**

- RAR 解压的常驻内存与**单个归档条目的大小解耦**——解压 4 GiB 条目时进程内存不得随条目大小线性增长
- 消除"对同一归档并发调用 unrar"的隐患
- 让解压过程在日志中可观测：哪个条目、多大、内存水位多少
- 保持现有用户可见行为：进度显示、取消、加密归档密码流程、分卷、目标目录结构

**Non-Goals:**

- 崩溃兜底与失败隔离（panic hook 落盘、stderr 重定向、WER LocalDumps、解压放独立子进程）——另开变更
- 解压前的条目大小预检 / 超大条目拒绝策略
- 把 ZIP / 7z / tar 的解压路径一并改造（它们不使用 `header.read()`，无同等峰值）
- 提升 RAR 解压吞吐

## Decisions

### 决策 1：用 `extract_to` / `extract_with_base` 流式落盘，替换 `header.read()`

```rust
// 现在
let (data, archive_after_read) = header.read()...;
let mut out = File::create(&dest)...;
for chunk in data.chunks(65536) { out.write_all(chunk)?; ... }

// 改为
open_archive = header.extract_with_base(dest_dir)...;   // C++ 侧直接写文件
```

`Extract` ProcessMode（`open_archive.rs:486-491`）把 `RARProcessFileW` 的工作完全交给 C++，数据不进 Rust 堆；Rust 侧只保留条目元信息。内存占用与条目大小无关。

**为什么不选其他方案：**
- *保留 `read()` 但调低并发/加内存预检*：治标。峰值由单条目大小决定，预检只能在解压前拒绝，不能解压大数据。
- *直接 spawn `unrar.exe` 子进程*：需要随包分发 unrar 二进制，引入外部可执行文件与新的杀软/签名问题；且复用不到现有 `unrar` crate 的分卷与密码处理。属于失败隔离变更的备选，不在此处。
- *自己实现分块解压*：`unrar` crate 的 `ProcessMode::process_data` 签名返回 `()`，公开 API 不提供"逐块交给调用方"的模式；自实现需要 fork crate 或改 vendor，成本远高于收益。

### 决策 2：进度改为按条目上报，长条目用磁盘大小轮询兜底

`Extract` 模式的 `process_data` 是空实现，拿不到字节级回调，因此字节级进度在现有 crate API 下不可得。方案：

1. 每个条目开始前发一次 `transfer-progress`（`status: running`，附条目名与 `unpacked_size`）
2. 条目完成后把 `unpacked_size` 累加进 `bytes_done`，再发一次
3. 对 `unpacked_size` 超过阈值（建议 256 MiB）的条目，另起一个轻量定时器（200-500ms）`fs::metadata` 目标文件已写字节数，据此发中间进度事件

> **实测否决（2026-10-16）**：unrar 解压前会把目标文件预留到完整解压大小，`fs::metadata().len()` 在条目开始 ~100ms 内即等于最终大小（`Master.pck` 1.3 GiB 在 100ms 报满、实际耗时 10.9s），该轮询只会让进度条瞬间跳到 100% 再卡住。改用进程级 `GetProcessIoCounters().WriteTransferCount` 可行（实测平滑增长），但为进程级共享计数，会被并发的其它传输污染。是否采用见 Open Questions。

**落地方案（2026-10-16，用户选"选项 3"）：不追字节级进度，改为大条目不确定态。** 不再尝试伪造百分比，而是：

1. RAR 每个条目开始时通过新增的 `EntryStartProgress` 回调（`Arc<dyn Fn(&str, u64)>`）上报条目名与 `unpacked_size`；`transfer-progress` 新增**附加**字段 `entry_name` / `entry_size`。
2. 前端对 `entry_size >= 256 MiB` 的 extract 条目，把百分比数字与确定态进度条替换为 `Extracting <name>` 文案 + 不确定动画条；`bytesDone/totalBytes` 仍按条目边界累加显示位置。下一个条目自动回到确定态。

这样大条目不再表现为"卡在 100%"或"停住不动"，而是明确告知"正在解压哪个文件、多大"。`EntryStartProgress` 只在 RAR 路径触发；ZIP/7z/tar 走字节流，进度本就准确，不受影响。

**取消语义的变化（需在任务中确认可接受）**：`ProcessMode::process_data` 无返回值，unrar crate 不支持从回调中途中止。所以**取消只能在条目之间生效**。当前实现在写盘循环里额外检查取消标志，属于旧实现附带的检查点；改造后取消当前条目会等它写完再停（文件完整，而非半个文件）。这不劣于现状（现状中途取消会留下截断文件）。

### 决策 3：unrar 调用全局串行化，只读入口用 `try_lock` 而非阻塞

引入一个进程级 `Mutex`（建议放在 `archive` 模块内，例如 `static UNRAR_GUARD: Mutex<()>`）：

- **写/解压类**（`extract_all` / `extract_files`）：阻塞 `lock()`
- **只读类**（`list_entries` / `read_file` / `total_uncompressed_size` / `read_file_bytes`）：`try_lock()`，失败则返回一个可识别的"归档正忙"错误

**为什么只读入口不阻塞**：解压一个 15 GiB 归档要几分钟，若 `read_archive_directory` 阻塞等锁，用户在解压期间点开这个 `.rar` 会让预览面板卡住几分钟。返回可识别的错误让前端能提示"归档正在解压中"，是更好的降级。

**为什么不做按路径加锁**：并发隐患来自静态 UnRAR 库的进程级全局状态，与是不是同一个归档无关。不同归档并发同样不安全，所以用全局单锁而非 keyed lock。

**副作用**：`extract_max_slots`（默认 2）对 RAR 实际退化为 1。这是安全上必须的，需在实现时确认调度器不会因此死锁（解压任务持锁期间，队列里的其他 RAR 解压任务只是排队）。

### 决策 4：诊断日志——条目级大小 + 进程内存水位

- 每个条目开始前 `log::debug!("[archive] extract entry: {name} size={unpacked_size}")`
- 超过阈值（如 256 MiB）的条目额外 `log::info!`，保证默认 Info 级别下可见
- 解压任务期间按固定间隔（如 5s）记录进程 working set / private bytes

内存水位读取：使用已在依赖里的 `windows` crate，需在 `src-tauri/Cargo.toml` 的 `windows` features 中加入 `Win32_System_ProcessStatus`，调用 `GetProcessMemoryInfo`。不引入新 crate。

**为什么不只记条目大小**：本次崩溃的最后一条日志停在 18:06:26，之后 37 秒空白——这正是内存抖动导致 UI 线程停摆的表现。有了内存水位日志，"静默 37 秒"会变成一串递增的内存数字，下次一眼就能看出是内存问题而不是"应用莫名卡死"。

## Risks / Trade-offs

- **`extract_with_base` 与现有目标路径拼接语义不一致** → `extract_files` 当前用 `Path::new(dest_dir).join(entry.filename)`，`extract_with_base` 走 `pathed::preprocess_extract`。需在实现时对含中文/空格/反斜杠条目名的归档做一次实际解压验证，确认落盘路径与现在完全一致（含分卷场景）。
- **进度事件频率下降导致前端进度条观感变差** → 见决策 2 的磁盘轮询兜底；若不做轮询，需确认 4 GiB 条目下 UI 表现可接受。
- **只读入口返回"归档正忙"是新错误分支** → 前端需处理该错误（至少不能静默失败）；实现时要确认前端已有 toast 通道可用。
- **全局锁引入死锁风险** → 现有调度器在解压任务的 `JoinHandle` 里再回去拿调度器锁；新增的 unrar 锁必须只包住 unrar 调用本身，不得跨 `.await` 或跨调度器锁持有。
- **改造后仍可能 OOM，但成因不同** → 若某个条目本身超过可用内存（例如 12 GiB 的条目），流式落盘不会崩，但 unrar 自身的解压窗口仍会占用；这属于 follow-up（预检/降级策略），不在本变更。
- **回归面**：解压是核心功能，加密/分卷/选中条目/整包解压四条路径都要重测。

## Migration Plan

无数据迁移、无配置迁移、无 API 破坏性变更。`transfer-progress` 字段不变。

上线方式：直接发版。若出问题，回滚点是恢复 `header.read()` 实现——建议实现时把两条路径的差异控制在 `rar.rs` 内部（不改 `mod.rs` 的函数签名），使回滚只需还原单个文件。

验证顺序：
1. 用本次崩溃的归档（`[暴走VR]刺客信条…v76.rar`，15.8 GiB，含 4 GiB 条目）整包解压，观察日志中的条目级大小与内存水位，确认内存不随条目大小增长
2. 选中条目解压（`extract_files` 路径）
3. 解压进行中，在面板里点开该 `.rar`，确认返回"归档正忙"而不是卡死
4. 加密归档 + 分卷归档回归
5. 与崩溃前产物做逐字节对比（`du -sb` 对比 `total_bytes`）

## Open Questions

- ~~长条目进度兜底（决策 2 第 3 点）是否本期实现？~~ **已定（2026-10-16，用户选"选项 3"）并已实现**：不做字节级进度，改为大条目"不确定态 + 条目名/大小文案"（见决策 2 落地方案）。`GetProcessIoCounters().WriteTransferCount` 方案未被采用（进程级计数会被并发传输污染）。
- "归档正忙"的文案与呈现方式（toast / 预览面板内提示）需要定；倾向于沿用现有 toast 通道。
- 内存水位日志的间隔与级别（建议 5s / Info，仅在解压任务运行期间开启）需要在实现时确认不会让日志文件膨胀太快。
