## Why

2026-10-16 18:07:03，wind.exe 在解压一个 15.8 GiB 的 RAR 归档时进程被系统终止。Windows 事件日志显示 `Exception code: 0xc0000409`，WER 报告 `P9: 7`（`FAST_FAIL_FATAL_APP_EXIT`）——即进程调用了 `abort()`，而非普通的崩溃或 panic。原因是 `archive/rar.rs` 的解压实现把**每个归档条目整份读进内存的 `Vec<u8>`** 再写盘；该归档内存在 4 GiB（恰好 2³²）、3.6 GiB、1.3 GiB 等超大条目，`Vec` 倍增扩容的瞬时峰值进一步放大到约 6 GiB，在 15.3 GiB 物理内存 + 17 GiB 页面文件的机器上触发分配失败，Rust 的 `handle_alloc_error` 直接 abort。

重启后同一归档再次解压**可以完整成功**，所以这是一个与进程内存状态相关的偶发失败——用户表现为"应用无缘无故自己退出"。同时日志显示解压进行中前端还对同一个 `.rar` 发起了第二次 unrar 调用（`read_archive_directory`），而 `unrar_sys` 静态链接的 UnRAR 源码含进程级全局状态，当前没有任何串行化保护。

## What Changes

- **RAR 解压改为流式落盘**：`extract_all` / `extract_files` 改用 unrar 的 `extract_to(path)` / `extract_with_base()`，由 C++ 侧直接写文件，彻底消除"N 个 GiB 的 `Vec<u8>`"这一内存峰值。取消当前"先读满整个条目再分块写盘"的实现。
- **保留进度与取消能力**：进度来源从"数据回调按字节上报"改为按条目粒度上报（条目完成 + 已处理字节），并保持 `transfer-progress` 事件契约与取消语义不变（用户可见行为不回归）。
- **串行化 unrar 调用**：对所有会打开归档的入口（`list_entries` / `read_file` / `extract_files` / `extract_all` / `total_uncompressed_size`）加互斥，禁止对同一归档并发打开，消除静态 UnRAR 库全局状态被并发踩踏的隐患。
- **解压路径的诊断日志**：每个条目开始前记录条目名与 `unpacked_size`（让日志直接暴露"即将处理一个 4 GiB 条目"），并在解压过程中按固定间隔记录进程内存水位，使同类问题下次可直接定位到条目与内存峰值。
- 非目标：本变更**不含**崩溃兜底与失败隔离（panic hook 落盘、stderr 重定向、WER LocalDumps、解压放独立子进程）；也不含"解压前按条目大小预检并拒绝/降级超大条目"的策略。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `rar-archive-support`：新增 RAR 解压的内存上界要求（MUST NOT 按条目整份驻留内存）与 unrar 调用串行化要求；现有"RAR supports extraction""Extract encrypted RAR"等场景保持行为不变，仅约束实现方式。
- `rust-logging-coverage`：新增归档解压路径的诊断日志要求（条目级 `unpacked_size` 记录、进程内存水位记录），补齐当前解压过程"日志空白"的缺口。

## Impact

- 后端：`src-tauri/src/archive/rar.rs`（主要）、`src-tauri/src/archive/mod.rs`、`src-tauri/src/archive/shared.rs`、`src-tauri/src/transfer/extract.rs`。
- 前端：无预期改动。`transfer-progress` 事件字段契约保持兼容（`bytes_done` / `total_bytes` / `status`），Transfer Manager 进度显示无需修改。
- 依赖：不新增依赖，继续使用 `unrar = "0.5"` 已有的流式 API。
- 行为影响：解压期间对同一归档的并发列表/预览将被阻塞或排队（可接受：解压本身已占用该归档）；进度事件频率由约 100ms 一次变为按条目一次，长条目（如 4 GiB）刷新变稀，需在任务中确认前端进度条体验可接受。
- 用户配置、数据库、文件格式均无变更。
