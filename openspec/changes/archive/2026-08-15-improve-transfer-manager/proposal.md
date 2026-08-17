## Why

传输管理器存在三个体验问题：

1. **Cancel All 非原子**：前端 `cancelAll()` 逐个 `await transfer_cancel(id)`，N 次 IPC 往返；且每次取消 queued 任务都会触发 `dispatch_pending`，把下一个 queued 任务 promote 成 running（worker 真的开始读写）后才被 cancel flag 停下，导致面板频繁从 queued→running→cancelled 抖动。
2. **缺少剩余时间**：running 状态只显示速度，没有 ETA；且本地复制/删除后端 emit 的 `speed_bps` 恒为 0，无法用于估算。
3. **目录总大小缺失**：本地复制/剪切目录时，文件列表对目录 `size = None`，前端传 `total_bytes = 0`，后端 `enqueue()` 对目录返回 0，导致进度永远显示 `0 B / 0 B`、0%。

## What Changes

- 新增后端 `cancel_all` 方法 + `transfer_cancel_all` 命令，一次持锁原子取消全部 queued（直接移除，不 dispatch、不 promote）与 running（置 cancel flag）；新增批量事件 `transfer-cancelled-batch`，前端单次 `update()` 完成 re-render。
- 顺带修复 `cancel()` queued 分支误调用 `free_slot_direct()` 的 bug（queued 任务从未 `take_slot`，错误释放槽位会破坏同盘串行约束）。
- 前端 store 基于滚动窗口速度计算单任务 ETA，并在 running 行渲染。
- 后端 enqueue 对目录递归求和（复用 `walk_local_dir`），并在 worker 运行时修正 total（A+B 混合），使目录复制/剪切正确显示总大小。

## Capabilities

### New Capabilities

无

### Modified Capabilities

- `transfer-manager`: 新增原子 cancel all、单任务 ETA、目录递归总大小的要求

## Impact

- `src-tauri/src/transfer.rs` — `cancel_all()` 方法、`dir_size` 递归求和、`free_slot_direct` bug 修复、worker 运行时 total 修正
- `src-tauri/src/lib.rs` — 注册 `transfer_cancel_all` 命令
- `src/lib/stores/transfer.ts` — `cancelAllTransfers()`、`transfer-cancelled-batch` 监听、ETA/滚动速度计算
- `src/lib/components/TransferManager.svelte` — `cancelAll()` 改单次调用、ETA 渲染
