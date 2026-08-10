## Context

当前 Wind 的文件操作分为两条独立的路径：

1. **本地文件操作**（`file_ops.rs`）：有完整的进度系统（`op-scan-complete`、`op-progress`、`op-complete`、`op-failed`、`op-cancelled` 事件），由 `FileOpProgress.svelte` 底部栏展示。但完���的任务 3 秒后自动消失，无历史记录。

2. **FTP 传输**（`lib.rs` 中的 `ftp_download`/`ftp_upload`）：仅 `await invoke()` + toast 提示。后端只发一次 `ftp-progress` 完成事件（done==total），无流式进度。多文件串行执行，UI 阻塞。

目标是将两条路径统一到 Transfer Manager 系统中，提供可观测、可控制、有历史记录的统一传输体验。

## Goals / Non-Goals

**Goals:**
- 统一的传输管理面板（可切换，类似 Floating Terminal）
- 后端调度器管理并发槽位（FTP 每连接 2 个，本地 2 个）
- FTP 流式进度上报（每 64KB），支持实时进度条和速度计算
- 传输历史持久化到 JSON 文件，保留最近 7 天/500 条
- 支持取消正在进行的传输、重试失败任务
- 队列中 pending 任务可通过拖拽或键盘重排
- 面板全局共享（非 per-tab），关闭面板不影响传输继续

**Non-Goals:**
- 不支持暂停/恢复（FTP REST 命令复杂度高）
- 不引入 SQLite（JSON 文件足够）
- 不支持跨端 FTP 服务器中转（已有独立 issue）
- batch 折叠功能（后续迭代考虑）
- 多选拖拽重排（仅支持单项拖拽）

## Decisions

### 1. 混合驱动架构（后端调度 + 前端 UI 镜像）

```
┌──────────────────────────────────────────────────────┐
│  前端 (Svelte)              后端 (Rust)               │
│  ┌──────────────────┐      ┌──────────────────────┐  │
│  │ TransferStore    │      │ TransferScheduler     │  │
│  │ (UI state)       │◀─────│ emits transfer-*      │  │
│  │                  │      │ events                │  │
│  │ invoke ──────────┼─────▶│ manages workers       │  │
│  │ enqueue/cancel/  │      │ controls concurrency  │  │
│  │ reorder/retry    │      │ persists history      │  │
│  └──────────────────┘      └──────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

**Why:** 关闭 TransferManager 面板后传输继续运行（后端驱动），前端只负责展示和控制。与当前 `op-*` 事件模式一致，前端通过 `listen()` 订阅事件更新 UI。

**Alternative considered:** 纯前端驱动（invoke + await）—— 面板关闭传输中断，不符合需求。

### 2. 统一事件模型

三种新事件替代旧的 `op-*` + `ftp-progress`：

| 事件 | Payload | 用途 |
|------|---------|------|
| `transfer-progress` | `{ id, batch_id, op_type, status, bytes_done, total_bytes, speed_bps, source, dest }` | 进度更新（throttled 100ms） |
| `transfer-complete` | `{ id, batch_id, bytes_done, elapsed_ms }` | 传输完成 |
| `transfer-failed` | `{ id, batch_id, error }` | 传输失败 |
| `transfer-cancelled` | `{ id, batch_id }` | 用户取消 |
| `transfer-queue-updated` | `{ ids: [] }` | 队列重排后通知 |

**Why:** 统一事件让 TransferManager 只需监听一组事件，FTP 和本地操作无需区分。

**Migration:** 保留旧事件继续发射（过渡期），TransferManager 只监听新事件。旧的 `FileOpProgress.svelte` 移除。

### 3. 传输作为独立操作（非 linked to batch invoke）

每次 `transfer_enqueue` 调用 = 一个 batch。同一次 paste/拖放产生的多个文件共享一个 batch_id。

```
paste 3 files to FTP:
  → invoke('transfer_enqueue', { tasks: [
      { type: 'ftp-upload', source, dest, size },  // id=1, batch_id=A
      { type: 'ftp-upload', source, dest, size },  // id=2, batch_id=A
      { type: 'ftp-upload', source, dest, size },  // id=3, batch_id=A
    ]})
  → 返回 [1, 2, 3]，前端立即看到队列
```

**Why:** Batch 粒度对用户有意义（"我粘贴的这批文件"），也方便 UI 用 batch 分隔符分组展示。

### 4. 数据结构

**后端 TransferTask：**
```rust
struct TransferTask {
    id: u64,
    batch_id: u64,
    op_type: TransferType,  // Copy, Move, Delete, FtpDownload, FtpUpload
    source: String,
    destination: String,
    total_bytes: u64,
    status: TaskStatus,     // Queued, Running, Done, Failed, Cancelled
}
```

**前端 TransferEntry（镜像 + UI 状态）：**
```typescript
interface TransferEntry {
  id: number;
  batchId: number;
  opType: 'copy' | 'move' | 'delete' | 'ftp-download' | 'ftp-upload';
  source: string;
  destination: string;
  totalBytes: number;
  bytesDone: number;
  speedBps: number;
  status: 'queued' | 'running' | 'done' | 'failed' | 'cancelled';
  error?: string;
  startTime: number;
}
```

### 5. TransferScheduler 并发策略

```
FTP 传输:
  - 同一连接：最多 2 个并发槽位
  - 不同连接：各自独立，不互相影响
  - 大文件 (>100MB)：独占 1 个槽，另一槽给小文件

本地传输:
  - 同盘复制：强制串行（1 个槽，避免磁头抖动）
  - 跨盘复制：最多 2 个并发
  - FTP↔Local：与 FTP 槽位独立，不影响彼此
```

调度器 FIFO 取队列头，有空闲槽位时补入。队列已被拖拽重排时取最新顺序。

### 6. 面板 UI 结构

```
TransferManager.svelte
├── Header bar (# active count, clear completed button)
├── Scrollable list (reverse chronological, newest at bottom)
│   ├── Completed batch divider  ← `──── date · type ────`
│   ├── Done/failed items (single line compact)
│   ├── Active batch divider     ← `──── Now · type ────`
│   └── Running/queued items (double line + progress bar + cancel)
└── Floating "↓ back to bottom" button (when scrolled up)
```

每行结构：
- **Running**: `[≡] [icon] source → dest` / `pct% ████░░░ size/size speed [✕]`
- **Queued**: `[≡] [⏳] source → dest` / `size queued [✕]`
- **Done**: `[✓] source → dest · size · speed · elapsed`
- **Failed**: `[✕] source → dest · error`

### 7. 快捷键设计

面板聚焦时：
- `j/k` — 上下选择
- `gg/G` — 跳到顶部/底部
- `c` — 取消选中的 queued/running 项
- `x` — 清除选中的 done/failed 项
- `Ctrl+Shift+J/K` — 队列中上下移动
- `Escape` — 关闭面板

全局（PanelLayout key handler）：
- `Ctrl+T` — 切换 TransferManager 面板

### 8. 历史持久化

`~/.local/share/wind/transfer-history.json`，只存档 done/failed/cancelled 项。

保留策略：
- 最多 500 条（FIFO 淘汰）
- done/cancelled 超过 7 天自动清理
- failed 超过 3 天自动清理
- 每次写入时检查清理

## Risks / Trade-offs

- **[Risk] FTP 并发连接过多触发服务器限流** → 默认每连接 2 个槽位，温和不激进。多数 FTP 服务器允许 4-8 并发连接。
- **[Risk] 大量历史记录导致 JSON 文件过大** → 500 条 × ~200 bytes ≈ 100KB，完全可控。
- **[Risk] 后端 scheduler 与旧 FileOpProgress 事件共存期可能重复显示** → TransferManager 只监听 `transfer-*`，旧 `op-*` 不变，逐步废弃。
- **[Risk] suppaftp 库对并发多连接的支持** → 需要为每个并发 FTP 传输创建独立的 `FtpSession`（当前每个连接名复用同一个 session）。解决方案：TransferScheduler 持有自己的 `FtpManager` 或通过 `reconnect` 创建额外 session。
