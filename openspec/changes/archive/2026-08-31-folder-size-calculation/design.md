## Context

当前 FileInfoPanel 只对文件显示大小，文件夹的 Size 固定为 0 B。`get_file_info` 命令同步返回元数据，不做递归计算。后端 `file_ops.rs` 已有成熟的进度事件模式（`op-progress` 每 100ms 节流），但前端从未接入。

需要新增一个异步流式计算能力，让文件夹大小像 Windows 资源管理器属性一样实时跳动。

## Goals / Non-Goals

**Goals:**
- 文件夹按 `i` 打开 FileInfoPanel 时，Size 行从 0 B 开始实时跳动
- 递归遍历计算文件夹总大小，通过 Tauri 事件流式推送进度
- 关闭面板时取消计算，释放资源

**Non-Goals:**
- 不缓存计算结果，每次打开都重新算
- 不设目录大小上限
- 不添加"预扫描"阶段，直接开始计数
- 不在目录列表中显示文件夹大小列

## Decisions

### 1. 事件名与数据结构

使用两个事件，与 `file_ops.rs` 的模式一致：

**`folder-size-tick`**（进度，每 100ms 节流）:
```json
{ "path": "/abs/path/to/folder", "total_bytes": 1234567, "files": 42, "dirs": 5 }
```

**`folder-size-done`**（完成）:
```json
{ "path": "/abs/path/to/folder", "total_bytes": 1234567, "files": 42, "dirs": 5 }
```

**Why**: 单一事件结构简单，`path` 用于前端路由到正确的面板实例，防止多个面板同时计算时串数据。

### 2. 取消机制

使用全局 `HashMap<PathBuf, Arc<AtomicBool>>` + `cancel_flag` 模式（与 `file_ops.rs` 一致）。

**Why**: 文件夹计算是纯只读遍历，不需要回滚，只需一个 flag 阻止继续遍历。`Arc<AtomicBool>` 在线程间零成本共享，HashMap 用 path 做 key 防止取消错操作。

### 3. 前端状态管理

FileInfoPanel 内部用 `$state` 管理：
- `folderSize: number` — 当前累计 bytes
- `folderFileCount: number` — 当前累计文件数
- `isCalculating: boolean` — 是否正在计算中

通过 `$effect` 在 visible 变为 true 且 info 为文件夹时：
1. 调用 `invoke('calculate_folder_size', { path })`
2. 用 `listen` 注册事件监听
3. 更新 `$state` 触发 UI 重渲染

**Why**: Svelte 5 `$state` 的细粒度响应式让数字跳动几乎零开销。`$effect` 返回的清理函数自然处理组件卸载时的取消和监听移除。

### 4. Rust 实现路径

在 `lib.rs` 中直接实现，不走 `file_ops.rs` 模块：

```rust
#[tauri::command]
async fn calculate_folder_size(path: String, app: AppHandle) -> Result<(), String> {
    let path = PathBuf::from(&path);
    let cancel_flag = Arc::new(AtomicBool::new(false));
    // 注册到全局 HashMap
    register_cancel_flag(&path, &cancel_flag);

    let path_clone = path.clone();
    std::thread::spawn(move || {
        let mut total_bytes = 0u64;
        let mut files = 0u64;
        let mut dirs = 0u64;
        let mut last_emit = Instant::now();

        walk_dir(&path, &cancel_flag, &mut total_bytes, &mut files, &mut dirs, &mut last_emit, &app);

        let _ = app.emit("folder-size-done", json!({
            "path": path_clone.display().to_string(),
            "total_bytes": total_bytes, "files": files, "dirs": dirs
        }));
        unregister_cancel_flag(&path_clone);
    });

    Ok(())
}
```

**Why NOT `spawn_blocking`**: 遍历可能持续数秒甚至数十秒，不能用 `spawn_blocking` 阻塞 Tauri 的 async runtime 线程池。裸 `std::thread::spawn` 更合适，Rust 的 daemon 线程不会阻塞进程退出。

### 5. 节流策略

每 100ms 最多 emit 一次 `folder-size-tick`。遍历循环中：
```rust
if last_emit.elapsed() >= Duration::from_millis(100) {
    app.emit("folder-size-tick", ...)?;
    last_emit = Instant::now();
}
```

**Why**: 100ms 对应约 10fps 的 UI 更新率，视觉效果流畅（数字平滑跳动），同时不会给前端渲染队列带来压力。与 `file_ops.rs` 的 `Progress::emit_progress` 保持一致。

## Risks / Trade-offs

- **大目录性能**: 遍历 `node_modules`（数十万文件）可能耗时 10-30 秒。Mitigation: 每 100ms emit 一次进度，用户始终能看到数字在增长，不会误以为卡死。取消功能让用户随时可以关闭面板中断。
- **权限错误**: 遍历到无权限的目录会触发 `io::Error`。Mitigation: 跳过无权限目录（`io::ErrorKind::PermissionDenied`），继续遍历其他部分，不中断整体计算。
- **符号链接循环**: Windows 符号链接可能导致无限循环。Mitigation: 遍历时不跟随符号链接（`symlink_metadata` + 不进入 is_symlink 的目录），或者对 Windows junction 做特殊处理。