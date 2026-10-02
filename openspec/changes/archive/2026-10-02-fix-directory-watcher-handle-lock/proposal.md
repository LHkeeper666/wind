## Why

Wind 使用 `notify` crate 的默认 Windows 后端 (`ReadDirectoryChangesW`) 监听目录变化，但 `notify 6.x` 创建句柄时未指定 `FILE_SHARE_DELETE` 共享模式。这导致被监听的目录及其所有子目录被锁定，用户在 Windows 资源管理器中无法删除/重命名这些目录，提示"资源被占用"。

这是一个基础性缺陷——文件管理器不应该阻止用户对外部目录的操作。

## What Changes

- 用 Windows 原生 API (`CreateFileW` + `FILE_SHARE_DELETE` + `ReadDirectoryChangesW` + `OVERLAPPED`) 替换 `notify` crate 的 `DirectoryWatcher` 后端，实现不锁定目录的递归监听
- 新增 `Win32_Storage_FileSystem`、`Win32_System_IO`、`Win32_System_Threading` 三个 `windows` crate feature flags
- 实现可靠的 `stop()` 退出机制：OVERLAPPED Event 对象 + `CancelIo` + `WaitForSingleObject` 超时轮询
- 正确解析 `FILE_NOTIFY_INFORMATION` 变长结构体，映射 `FILE_ACTION` 到现有事件格式
- 处理目录被外部删除、buffer 溢出、磁盘弹出等边界情况
- `FileWatcher` 保持不变（已使用轮询模式，无此问题）

## Capabilities

### New Capabilities

- `native-directory-watcher`: 基于 Windows 原生 API 的目录监听器，使用 `FILE_SHARE_DELETE` 避免锁定目录，支持递归监听和可靠的 stop/cleanup 生命周期

### Modified Capabilities

- `project-tree-auto-refresh`: 现有目录变更通知机制的后端实现变更，事件格式保持兼容，前端无需修改

## Impact

- **代码**: `src-tauri/src/directory_watcher.rs` 完全重写，`src-tauri/Cargo.toml` 新增 feature flags
- **依赖**: 无新依赖，仅启用 `windows` crate 已有但未开启的 feature
- **兼容性**: 事件格式 (`directory-changed` + `Vec<String>`) 保持不变，前端无需修改
- **最低系统要求**: Windows 10 1709+ (Fall Creators Update)，支持 `ReadDirectoryChangesW` + `FILE_SHARE_DELETE` 组合