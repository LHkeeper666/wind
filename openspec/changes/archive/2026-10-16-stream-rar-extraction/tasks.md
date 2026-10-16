## 1. unrar 调用串行化

- [x] 1.1 在 `archive` 模块新增进程级 `Mutex` 守卫，暴露 `lock_blocking()`（解压用）与 `try_lock_busy()`（只读用）两个入口
- [x] 1.2 `archive/rar.rs` 的 `list_entries` / `read_file` / `total_uncompressed_size` 改为在 `try_lock_busy()` 保护下调用 unrar，失败时返回可识别的 "archive busy" 错误串
- [x] 1.3 `archive/rar.rs` 的 `extract_all` / `extract_files` 改为在 `lock_blocking()` 保护下调用 unrar，确认锁不跨越 `.await` 也不嵌套在调度器锁内
- [x] 1.4 确认 `archive/mod.rs` 出口不重复加锁，ZIP / 7z / tar 路径行为不变
- [x] 1.5 验证：解压进行中对同一归档调用 `list_entries` 返回 busy 错误而非阻塞

## 2. RAR 流式解压

- [x] 2.1 `extract_all` 用 `header.extract_with_base(dest_dir)` 替换 `header.read()` + 手动分块写盘
- [x] 2.2 `extract_files` 同样替换，保留"只解压目标条目"的行为
- [x] 2.3 保留目录条目分支（`create_dir_all` + `skip()`）与 `skip_paths` 跳过逻辑
- [x] 2.4 保留 `remember_archive_password` 在成功路径上的调用时机
- [x] 2.5 验证含中文 / 空格 / 反斜杠条目名的归档落盘路径与改造前一致

## 3. 进度与取消

- [x] 3.1 条目开始与结束时发 `transfer-progress`：`bytes_done` 按条目 `unpacked_size` 累加，`total_bytes` 保持归档总解压大小
- [x] 3.2 取消标志改为在条目边界检查，不再在写盘循环内检查
- [x] 3.3 `extract_files` 路径的 `total_bytes` 只统计选中条目
- [x] 3.4 实测 4 GiB 条目下的进度条观感，决定是否实现设计决策 2 第 3 点的磁盘大小轮询（200-500ms 读目标文件已写字节数）

  **实测结论（2026-10-16）：该方案不可行，已回退实现。** unrar 在解压每个条目时会先把目标文件 `SetEndOfFile` 预留到完整解压大小，因此 `fs::metadata(dest).len()` 在条目刚开始的 ~100ms 内就直接等于最终大小，随后不再变化——轮询会立刻把该条目报成 100% 然后卡住，与目的相反。实测：`Master.pck`（1.3 GiB）在 100ms 时 size 已达 1305430263，而实际解压耗时 10.9s；`runtimeanimatorcontrollers_assets_all.bundle`（509 MiB）同样在 100ms 报满、实际耗时 3.8s。

  可行的替代信号：`GetProcessIoCounters().WriteTransferCount`（进程级累计写入字节，`Win32_System_Threading` 已启用，无需打开文件句柄）在实测中随解压平滑增长（1.1s 时 176 MB）。代价是**进程级**共享计数：若同时有其他传输（copy/FTP 下载）在写大量数据，会污染该条目的进度映射。

  **最终方案（2026-10-16，用户选"选项 3"）：不做假百分比，大条目走不确定态 + 文案。** 后端在 RAR 每个条目开始时通过新增的 `EntryStartProgress` 回调上报条目名与 `unpacked_size`，`transfer-progress` 事件新增两个**附加**字段 `entry_name` / `entry_size`（既有字段不变）。前端 Transfer Manager 对 `opType === 'extract'` 且 `entry_size >= 256 MiB` 的条目，把百分比替换为 `Extracting <name>` 文案 + 不确定动画进度条（`size-text` 仍显示整体 `bytesDone/totalBytes`）；下一个条目回到确定态。非 RAR 格式与未上报条目的路径行为完全不变。

## 4. 诊断日志

- [x] 4.1 每个条目开始前输出 `[archive] extract entry: {name} size={unpacked_size}`（debug；超过 256 MiB 用 info）
- [x] 4.2 解压任务开始与结束记录 task id、归档路径、目标目录，结束时记录结果（条目数 / 总字节数或失败原因）
- [x] 4.3 `src-tauri/Cargo.toml` 的 `windows` features 增加 `Win32_System_ProcessStatus`
- [x] 4.4 解压任务运行期间每 5 秒输出一次进程 working set / private bytes，任务结束时停止

## 5. 前端适配

- [x] 5.1 归档列表 / 预览收到 "archive busy" 错误时提示"归档正在解压中"，不静默失败
- [x] 5.2 确认 `transfer-progress` 事件字段未变，Transfer Manager 无需改动

  **修订（见 3.4）：** 为支持大条目不确定态，`transfer-progress` **新增** `entry_name` / `entry_size` 两个字段（纯附加，既有字段与语义不变）。Transfer Manager 消费这两个新字段；其它传输类型不发送它们，行为不变。

## 6. 验证

- [x] 6.1 用崩溃现场归档（`[暴走VR]刺客信条…v76.rar`，15.8 GiB，含两个 4 GiB 条目）整包解压，`du -sb` 结果与 scheduler 上报的 `total_bytes`（16986650072）一致
- [x] 6.2 解压该归档期间检查日志：条目级大小与内存水位可见，内存不随条目大小线性增长
- [x] 6.3 选中条目解压（`extract_files`）路径回归
- [x] 6.4 解压进行中点开该 `.rar`，确认返回 busy 错误且 UI 不卡死
- [x] 6.5 加密归档与分卷归档（`.partN.rar`、`.rar`/`.rXX`）回归
- [x] 6.6 `cargo check`（`src-tauri/`）与 `npx svelte-check` 通过
