## 1. 后端原子 cancel all

- [x] 1.1 在 `transfer.rs` 新增 `cancel_all(sched)` 方法：排空 `queue`（emit cancelled + 写历史，不 `free_slot_direct`、不 `dispatch_pending`）；遍历 `active` 置 `cancel_flag` 并 emit cancelled；收集全部 cancelled ids 后 emit 一次 `transfer-cancelled-batch`
- [x] 1.2 在 `lib.rs` 注册 `transfer_cancel_all` 命令并加入 `invoke_handler`
- [x] 1.3 修复 `cancel()` queued 分支：移除 `free_slot_direct()` 误调用

## 2. 前端 cancel all 原子化

- [x] 2.1 在 `transfer.ts` 新增 `cancelAllTransfers()`，单次 `invoke('transfer_cancel_all')`
- [x] 2.2 在 `transfer.ts` 新增 `transfer-cancelled-batch` 监听，单次 `update()` 批量标记 `status: 'cancelled'`
- [x] 2.3 `TransferManager.svelte` 的 `cancelAll()` 改为调用 `transfer.cancelAllTransfers()`

## 3. 目录总大小（A+B）

- [x] 3.1 在 `transfer.rs` 抽 `dir_size(path) -> u64` helper（递归求和，复用 `walk_local_dir` 逻辑）
- [x] 3.2 `enqueue()` 中 `total_bytes==0` 且 `path.is_dir()` 时调用 `dir_size` 填充
- [x] 3.3 `execute_local_copy` 在 `spawn_blocking` 内对目录先计算大小，若与 `task.total_bytes` 不符则 emit 一次带正确 total 的 running 事件（仿 FTP 下载 `850-857` 模式）

## 4. 单任务 ETA

- [x] 4.1 在 `transfer.ts` 维护 per-id 滚动速度样本，每次 `transfer-progress` 计算 EMA 速度与 `etaSecs`，挂到 entry
- [x] 4.2 `TransferManager.svelte` running 行渲染 `ETA ...`（复用/扩展 `formatElapsed`；speed 为 0 或 total 为 0 时不显示）

## 5. 验证

- [x] 5.1 `cargo check`（src-tauri）+ `npx svelte-check`
- [ ] 5.2 手动：队列多任务 + 多 running 时点 Cancel All，确认面板只更新一次、无 queued→running 抖动
- [ ] 5.3 手动：本地复制/剪切目录，确认 queued 与 running 均显示正确总大小与进度百分比
- [ ] 5.4 手动：running 任务显示合理 ETA，初期速度过小时不显示
