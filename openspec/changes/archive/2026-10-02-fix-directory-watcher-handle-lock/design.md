## Context

Wind 使用 `notify 6.x` crate 的 `DirectoryWatcher` 监听项目目录变化，驱动项目树自动刷新。在 Windows 上，`notify` 默认使用 `ReadDirectoryChangesW` 后端，创建句柄时共享模式为 `FILE_SHARE_READ | FILE_SHARE_WRITE`，**未包含 `FILE_SHARE_DELETE`**。

这导致 `CreateFileW` 使用 `FILE_FLAG_BACKUP_SEMANTICS` 打开的目录句柄会阻止外部进程删除/重命名该目录。由于 `DirectoryWatcher` 使用 `RecursiveMode::Recursive`，项目根下的**每一个子目录**都会被锁定。

`FileWatcher` 已经通过切换到 `PollWatcher`（轮询模式）规避了此问题，但 `DirectoryWatcher` 仍使用原生后端。

## Goals / Non-Goals

**Goals:**
- `DirectoryWatcher` 不再阻止外部进程删除/重命名被监听的目录
- 保持递归监听能力（一个句柄 watch 整棵子目录树）
- 保持现有事件格式（`directory-changed` + `Vec<String>`）不变，前端零改动
- 可靠的 start/stop 生命周期，无句柄泄漏

**Non-Goals:**
- 不修改 `FileWatcher`（已正常工作）
- 不替换整个 `notify` crate（仅 `DirectoryWatcher` 不使用它）
- 不支持 Windows 10 1709 以下版本（`FILE_SHARE_DELETE` + `ReadDirectoryChangesW` 需要 1709+）
- 不实现 IOCP 多句柄管理（当前只需 watch 一个目录树）

## Decisions

### Decision 1: 直接调用 Windows API，不修改 notify crate

**选择**: `directory_watcher.rs` 完全重写，直接使用 `windows` crate 调用 `CreateFileW` + `ReadDirectoryChangesW`，绕过 `notify`。

**替代方案**:
- Fork `notify` 添加 `FILE_SHARE_DELETE` 支持 → 维护负担大
- 向 `notify` 上游提 PR → 周期不可控，且 `notify` 的 Config API 不暴露共享模式
- 使用 `PollWatcher` → 最简单，但丢失实时性（1-2 秒延迟）

**理由**: `DirectoryWatcher` 只有一个实例，接口简单（start/stop），直接调 Windows API 改动量可控且效果最精确。

### Decision 2: OVERLAPPED + Event 对象的同步等待模式

**选择**: 使用 `OVERLAPPED` 结构体配合手动创建的 `Event` 对象，通过 `WaitForSingleObject(event, timeout)` 实现可中断等待。

**替代方案**:
- IOCP (`CreateIoCompletionPort` + `GetQueuedCompletionStatus`) → 过度设计，当前只需管理一个句柄
- 纯同步 `ReadDirectoryChangesW`（不传 OVERLAPPED）→ 无法可靠中断，`CancelIo` 行为不确定

**理由**: 单句柄场景下，OVERLAPPED + Event 足够。`stop()` 通过 `SetEvent` 唤醒线程，配合 `CancelIo` 取消 pending I/O，实现可靠的退出。

### Decision 3: 1 秒超时轮询 stop_flag

**选择**: `WaitForSingleObject(event, 1000)` 使用 1 秒超时，每次超时检查 `AtomicBool` stop flag。

**替代方案**:
- 无限等待 `INFINITE`，依赖 `SetEvent` 唤醒 → 如果 `SetEvent` 在 `ReadDirectoryChangesW` 提交前调用，会错过唤醒信号
- 更短超时（100ms）→ 频繁唤醒浪费 CPU

**理由**: 1 秒是合理的折中。最坏情况下 stop 延迟 1 秒，但保证了即使在 `ReadDirectoryChangesW` 提交前 `SetEvent` 被调用，线程也能在 1 秒内检测到。

### Decision 4: 不使用 notify::Watcher trait

**选择**: `DirectoryWatcher` 不实现 `notify::Watcher` trait，直接暴露 `start`/`stop` 方法。

**替代方案**: 实现 `notify::Watcher` trait 保持兼容 → 需要包装异步逻辑到 trait 的同步接口中，增加复杂度且无实际收益。

**理由**: `DirectoryWatcher` 只被 `lib.rs` 的 Tauri command 调用，无需多态。保持简单接口更易维护。

## Risks / Trade-offs

**[Risk] Windows 10 1709 以下版本不支持** → Mitigation: 在 `start()` 中检测 Windows 版本，低于 1709 时回退到 `notify` 的 `PollWatcher` 模式。实际上 1709 发布于 2017 年，绝大多数用户已满足。

**[Risk] 目录被外部删除后 handle 失效** → Mitigation: `ReadDirectoryChangesW` 返回错误或 0 字节时，清理句柄并通知前端。项目树收到 `directory-changed` 事件后会尝试刷新，刷新失败的分支自然失效。

**[Risk] buffer 溢出导致 `ERROR_NOTIFY_ENUM_DIR`** → Mitigation: 使用 16KB buffer（比默认 4KB 大），收到溢出错误时丢弃本次结果并重新提交 `ReadDirectoryChangesW`，前端下次刷新时会拿到完整状态。

**[Risk] `CancelIo` 可能无法取消正在执行的 I/O** → Mitigation: `CancelIoEx` 是更可靠的替代（可以取消特定 OVERLAPPED 的 I/O）。优先使用 `CancelIoEx`，`CancelIo` 作为 fallback。配合 1 秒超时，即使取消失败线程也会自然退出。

**[Trade-off] 丢失实时性 vs notify 原生后端** → `ReadDirectoryChangesW` 本身是实时的（内核通知），但 OVERLAPPED 模式下每次事件后需要重新提交调用，有极小的窗口期。实际上这个窗口期可以忽略不计。