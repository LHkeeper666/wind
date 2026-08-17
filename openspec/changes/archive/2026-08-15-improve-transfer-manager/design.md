## Context

`TransferScheduler` 用 `queue: VecDeque<TransferTask>` + `active: HashMap<u64, ActiveTask>` 管理任务，槽位按 FTP 连接 / 本地同盘串行约束分配。前端通过事件 `transfer-progress`/`transfer-cancelled`/`transfer-complete`/`transfer-failed`/`transfer-queue-updated` 同步 UI。

当前 `cancel(id)` 对 queued 任务：`queue.remove` + emit cancelled + `free_slot_direct` + `dispatch_pending`；对 active 任务：`set cancel_flag` + emit cancelled。`dispatch_pending` 会 `cleanup_finished` + `dispatch_queued`，后者把队首 promote 成 running。

## Goals / Non-Goals

**Goals:**
- Cancel All 一次后端调用原子完成，不 promote queued→running，单次 UI 更新
- running 任务显示单任务级 ETA
- 本地目录复制/剪切正确显示递归总大小（queued 阶段即有，running 阶段兜底修正）

**Non-Goals:**
- 不改并发槽位策略本身（`local_max_slots`/`ftp_max_per_conn` 语义不变）
- 不改 batch 级聚合进度/ETA（本次只做单任务级）
- 不改历史记录格式

## Decisions

### Decision 1: 新增后端 `cancel_all` + 批量事件，而非前端循环

**选择**: 后端 `cancel_all(sched)` 一次持锁完成全部取消，前端 `cancelAllTransfers()` 单次 `invoke('transfer_cancel_all')`；后端 emit 一次 `transfer-cancelled-batch { ids: [...] }`，store 单次 `update()` 批量标记。

**理由**: 消除 N 次 IPC 与 N 次 re-render；取消 queued 时不再触发 `dispatch_pending`，避免 promote churn。批量事件是"面板频繁更新"的根治点。

### Decision 2: 修复 `cancel()` 的 `free_slot_direct` 误调用

**选择**: 从 `cancel()` queued 分支移除 `free_slot_direct()` 调用（queued 任务从未 `take_slot`）；`cancel_all` 同样不对 queued 任务释放槽位。

**理由**: 现有代码错误地把 `local_slots_used`/`ftp_conn_slots` 减一，`saturating_sub` 掩盖下溢，但会破坏同盘串行约束（cancel 一个 queued 后可能错误地并发 dispatch 第二个同盘任务）。active 任务由 worker 完成回调负责释放槽位。

### Decision 3: ETA 在前端用滚动窗口速度计算

**选择**: store 维护 `Map<id, {lastBytes, lastTime, samples[]}>`，每次 `transfer-progress` 事件计算滚动窗口（最近约 1~2s）的 EMA 速度，`etaSecs = (totalBytes - bytesDone) / speed`。后端不改 worker 的 speed 计算。

**理由**: 后端平均速度（`done/elapsed`）对 ETA 偏慢，且本地传输 `speed_bps` 恒 0，需统一覆盖 copy/move/delete/ftp。前端 100ms 节流的 `bytesDone` 足够估算。

### Decision 4: 目录总大小 A+B 混合

**选择**: (A) enqueue 时对目录 `is_dir()` 递归求和（复用/抽 `dir_size` helper，内部走 `walk_local_dir`），写入 `total_bytes`，queued 行立即显示正确大小；(B) `execute_local_copy` 在 `spawn_blocking` 内先计算目录大小，若与 `task.total_bytes` 不符则 emit 一次带正确 total 的 running 事件兜底。

**理由**: A 解决"queued 即显示"，B 解决"大目录 enqueue 同步扫描阻塞 async 命令"的兜底。A 的同步扫描成本对中小目录可接受；超大目录依赖 B 在 blocking 线程内修正。

## Risks / Trade-offs

- [Risk] enqueue 对大目录同步递归求和会短暂阻塞 async 命令（持调度器锁）→ Mitigation: B 运行时修正兜底；后续可改为 `spawn_blocking` 异步预扫描。
- [Risk] EMA 速度在传输初期抖动大，ETA 不稳定 → Mitigation: 窗口内样本不足或 speed 过小时不显示 ETA。
- [Risk] `transfer-cancelled-batch` 新增事件与现有 `transfer-cancelled` 并存，需确保 store 两侧都处理，避免漏更新 → Mitigation: 单任务取消仍走 `transfer-cancelled`，仅 cancel-all 走批量事件。
