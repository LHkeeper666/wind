## 1. Feature Flags

- [x] 1.1 在 `src-tauri/Cargo.toml` 中为 `windows` crate 新增 `Win32_Storage_FileSystem`、`Win32_System_IO`、`Win32_System_Threading` 三个 feature flags

## 2. DirectoryWatcher 核心实现

- [x] 2.1 重写 `src-tauri/src/directory_watcher.rs`，定义 `DirectoryWatcher` 结构体（`stop_flag: Arc<AtomicBool>`、`thread_handle: Option<JoinHandle<()>>`）
- [x] 2.2 实现 `start()` 方法：`CreateFileW` 以 `FILE_LIST_DIRECTORY` + `FILE_SHARE_READ|WRITE|DELETE` + `FILE_FLAG_BACKUP_SEMANTICS` 打开目录句柄
- [x] 2.3 实现 `start()` 中的 watch 循环：创建 `OVERLAPPED` + `Event` 对象，提交 `ReadDirectoryChangesW`，`WaitForSingleObject(event, 1000)` 超时轮询
- [x] 2.4 实现 `FILE_NOTIFY_INFORMATION` buffer 解析：按 `NextEntryOffset` 遍历，提取 `Action` 和 `FileName`（UTF-16 → String）
- [x] 2.5 实现路径过滤逻辑：排除 `.git`、`target`、`node_modules` 目录段内的事件
- [x] 2.6 实现事件发送：将解析后的路径列表通过 `app_handle.emit("directory-changed", paths)` 发送到前端

## 3. Stop 生命周期

- [x] 3.1 实现 `stop()` 方法：设置 `stop_flag`，`SetEvent` 唤醒线程，`CancelIoEx`/`CancelIo` 取消 pending I/O，`join` 等待线程退出
- [x] 3.2 实现线程内 cleanup：关闭 directory handle（`CloseHandle`）和 event handle，确保所有退出路径（正常/错误/溢出）都执行

## 4. 错误处理与边界情况

- [x] 4.1 处理 `ReadDirectoryChangesW` 失败：关闭句柄，通知前端监听停止，线程退出
- [x] 4.2 处理 buffer 溢出（`ERROR_NOTIFY_ENUM_DIR`）：丢弃不完整结果，重新提交 `ReadDirectoryChangesW`
- [x] 4.3 处理目录被外部删除：检测错误码，清理句柄，通知前端
- [x] 4.4 处理快速连续 `start()`/`stop()`：`stop()` 中 `join` 确保旧线程完全退出后再允许 `start()`

## 5. 验证

- [x] 5.1 运行 `cargo check` 确认编译通过
- [x] 5.2 运行 `cargo build` 确认构建成功
- [ ] 5.3 手动测试：启动项目树模式，在资源管理器中删除/重命名子目录，确认不报"资源被占用"
- [ ] 5.4 手动测试：快速切换 tab，确认 stop/start 生命周期无泄漏
- [ ] 5.5 手动测试：在大型目录树（node_modules 级别）中确认事件正常触发且过滤生效