## Context

`src-tauri/src/transfer.rs` 当前 1455 行，包含 10 种职责。项目已有 `archive/`、`commands/`、`terminal/`、`video/` 等目录模块先例。Tauri 命令层已在 `commands/transfer_cmd.rs` 中分离，transfer 模块内部仍需进一步拆分。

## Goals / Non-Goals

**Goals:**
- 将 `transfer.rs` 拆分为 `transfer/` 目录模块，每个子文件 100-250 行
- 保持所有 public API 不变（TransferScheduler、EnqueueTask、TransferType 等类型签名不变）
- `commands/transfer_cmd.rs` 和 `lib.rs` 的 import 路径自动兼容（模块名不变）

**Non-Goals:**
- 不改变任何功能行为
- 不重构执行器内部逻辑（如进度回调机制）
- 不修改 `commands/transfer_cmd.rs` 的结构

## Decisions

### 1. 模块边界划分

**Decision**: 按职责拆分为 6 个文件

| 文件 | 职责 | 估算行数 |
|------|------|----------|
| `mod.rs` | 类型定义 + re-export + `execute_transfer` 分发 | ~120 |
| `scheduler.rs` | TransferScheduler 结构体及所有方法 | ~550 |
| `ftp.rs` | FTP 执行器 + ProgressAsyncReader + ensure_remote_dir | ~280 |
| `local.rs` | 本地执行器 + copy/delete 辅助函数 | ~350 |
| `conflict.rs` | scan_dir_conflicts + walk_local_dir | ~100 |
| `helpers.rs` | now_secs, dir_size, extract_ftp_conn, check_cancelled | ~50 |

**Rationale**: 按数据流而非协议分组。scheduler 管理队列和调度，execution 按 FTP/本地分离因为依赖不同（suppaftp vs std::fs），conflict 独立因为被 transfer_cmd.rs 直接调用。

### 2. execute_transfer 放在 mod.rs

**Decision**: `execute_transfer` 分发函数放在 `mod.rs`，而非 `scheduler.rs` 或独立文件。

**Rationale**: 它桥接 scheduler（调用方）和各执行器（实现方），放在 mod.rs 作为模块内部路由点，避免 scheduler 依赖 execution 模块或反之。

### 3. TransferScheduler 的 FTP folder enqueue 方法留在 scheduler.rs

**Decision**: `enqueue_ftp_folder_download` 和 `enqueue_ftp_folder_upload` 保留在 `scheduler.rs`。

**Alternatives**: 移到 `ftp.rs`。
**Rationale**: 这两个方法是 `&mut self` 方法，操作 scheduler 内部状态（queue, next_batch_id）。虽然内部调用 FTP API，但它们本质上是调度器的入队逻辑。将它们移到 ftp.rs 会导致 scheduler 字段需要 pub(crate) 暴露。

### 4. 冲突扫描独立为 conflict.rs

**Decision**: `scan_dir_conflicts` 和 `walk_local_dir` 放在 `conflict.rs`。

**Rationale**: 这两个函数被 `transfer_cmd.rs` 直接调用（scan_transfer_conflicts、scan_ftp_upload_conflicts），且不依赖 TransferScheduler 状态。独立后 transfer_cmd.rs 只需改 import 路径。

## Risks / Trade-offs

- **[Risk] 拆分后跨文件引用增加** → 通过 mod.rs 统一 re-export，外部调用者 import 路径不变
- **[Risk] 遗漏某个函数的迁移** → cargo check 会立即暴露未解析的符号
- **[Trade-off] 文件数从 1 变 6** → 每个文件职责单一，可独立理解；总行数不变
