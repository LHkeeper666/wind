## ADDED Requirements

### Requirement: 使用 FILE_SHARE_DELETE 创建目录句柄
系统 SHALL 使用 `CreateFileW` 以 `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` 共享模式打开目录句柄，允许外部进程在监听期间删除或重命名该目录。

#### Scenario: 监听期间外部删除目录
- **WHEN** DirectoryWatcher 正在监听目录 `C:\projects\myapp`
- **AND** 用户在 Windows 资源管理器中删除 `C:\projects\myapp\temp`
- **THEN** 资源管理器的删除操作 SHALL 成功，不报"资源被占用"错误

#### Scenario: 监听期间外部重命名目录
- **WHEN** DirectoryWatcher 正在监听目录 `C:\projects\myapp`
- **AND** 用户在 Windows 资源管理器中重命名 `C:\projects\myapp\old` 为 `new`
- **THEN** 重命名操作 SHALL 成功

### Requirement: 递归监听子目录树
系统 SHALL 使用 `ReadDirectoryChangesW` 的 `bWatchSubtree=TRUE` 参数，以单个句柄递归监听目录树中所有子目录的文件名、目录名和最后写入时间变更。

#### Scenario: 子目录中创建文件
- **WHEN** 在被监听目录的任意深度子目录中创建新文件
- **THEN** 系统 SHALL 在 100ms 内发出包含该文件路径的 `directory-changed` 事件

#### Scenario: 子目录中删除文件
- **WHEN** 在被监听目录的任意深度子目录中删除文件
- **THEN** 系统 SHALL 发出包含该文件路径的 `directory-changed` 事件

#### Scenario: 子目录中修改文件
- **WHEN** 在被监听目录的任意深度子目录中修改文件内容
- **THEN** 系统 SHALL 发出包含该文件路径的 `directory-changed` 事件

### Requirement: 使用 OVERLAPPED 异步模式提交监听
系统 SHALL 为 `ReadDirectoryChangesW` 传入 `OVERLAPPED` 结构体和关联的 `Event` 对象，使用异步模式提交监听请求。`WaitForSingleObject` SHALL 使用 1 秒超时等待事件。

#### Scenario: 正常事件到达
- **WHEN** `ReadDirectoryChangesW` 完成并产生事件数据
- **THEN** `OVERLAPPED` 的 `Event` 被 signaled，`WaitForSingleObject` 返回 `WAIT_OBJECT_0`
- **AND** 系统解析 buffer 中的 `FILE_NOTIFY_INFORMATION` 并发出事件

#### Scenario: 超时无事件
- **WHEN** 1 秒内无目录变更
- **THEN** `WaitForSingleObject` 返回超时
- **AND** 系统检查 stop flag，若为 false 则重新提交 `ReadDirectoryChangesW`

### Requirement: 可靠的 stop 退出机制
系统 SHALL 通过 `SetEvent` 唤醒等待中的线程，配合 `AtomicBool` stop flag 和 `CancelIo`/`CancelIoEx` 实现可靠的 stop。`stop()` SHALL 阻塞直到线程完全退出（`JoinHandle::join`）。

#### Scenario: stop 被调用时线程在等待事件
- **WHEN** `stop()` 被调用，且 watch 线程正阻塞在 `WaitForSingleObject`
- **THEN** `SetEvent` 唤醒线程
- **AND** 线程检测 stop flag 为 true
- **AND** 调用 `CancelIo` 取消 pending I/O
- **AND** 关闭 directory handle 和 event handle
- **AND** `stop()` 返回时，线程已完全退出

#### Scenario: stop 被调用时线程正在处理事件
- **WHEN** `stop()` 被调用，且 watch 线程正在解析 buffer
- **AND** 下次 `WaitForSingleObject` 超时或下次 `ReadDirectoryChangesW` 被 `CancelIo` 取消
- **THEN** 线程检测 stop flag 为 true 并清理退出

#### Scenario: 快速连续 start/stop
- **WHEN** `stop()` 后立即 `start()` 被调用
- **THEN** `stop()` 必须等待旧线程完全退出后，`start()` 才创建新线程和句柄

### Requirement: 正确解析 FILE_NOTIFY_INFORMATION 变长结构体
系统 SHALL 正确遍历 `ReadDirectoryChangesW` 返回的 buffer，按 `NextEntryOffset` 前进指针，解析每条 `FILE_NOTIFY_INFORMATION` 的 `Action` 和 `FileName` 字段。

#### Scenario: 单条事件
- **WHEN** buffer 包含一条 `FILE_NOTIFY_INFORMATION`（`NextEntryOffset=0`）
- **THEN** 系统解析该条事件的 `Action` 和 `FileName`

#### Scenario: 多条事件
- **WHEN** buffer 包含多条 `FILE_NOTIFY_INFORMATION`，最后一条 `NextEntryOffset=0`
- **THEN** 系统按 `NextEntryOffset` 遍历所有条目
- **AND** 将 `FILE_ACTION_ADDED`/`REMOVED`/`MODIFIED`/`RENAMED_NEW_NAME` 映射为事件路径

#### Scenario: buffer 溢出
- **WHEN** `ReadDirectoryChangesW` 返回成功但 buffer 不足以容纳所有变更
- **THEN** 系统丢弃本次不完整结果
- **AND** 重新提交 `ReadDirectoryChangesW`

### Requirement: 过滤特定目录的事件
系统 SHALL 过滤掉 `.git`、`target`、`node_modules` 目录内的变更事件，不向上层发出通知。

#### Scenario: target 目录内变更
- **WHEN** `target/debug/build.o` 被修改
- **AND** 其路径包含 `.git`、`target` 或 `node_modules` 目录段
- **THEN** 系统不发出 `directory-changed` 事件

### Requirement: 句柄清理无泄漏
系统 SHALL 确保在任何退出路径（正常 stop、错误、panic）上都关闭 directory handle 和 event handle。

#### Scenario: ReadDirectoryChangesW 失败
- **WHEN** `ReadDirectoryChangesW` 返回错误
- **THEN** 系统关闭 directory handle 和 event handle
- **AND** 线程退出

#### Scenario: 目录被外部删除后 handle 失效
- **WHEN** 被监听的根目录被外部删除
- **AND** `ReadDirectoryChangesW` 返回错误或 0 字节
- **THEN** 系统关闭 handle 并通知前端监听已停止