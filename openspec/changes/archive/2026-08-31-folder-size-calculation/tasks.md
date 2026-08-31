## 1. Rust 后端

- [x] 1.1 添加取消标记注册表（全局 `HashMap<PathBuf, Arc<AtomicBool>>`），包含 `register_cancel_flag`、`unregister_cancel_flag`、`cancel_folder_size` 函数
- [x] 1.2 实现 `walk_dir` 递归遍历函数：跳过无权限目录、跳过符号链接/junction、每 100ms 节流 emit `folder-size-tick`
- [x] 1.3 实现 `calculate_folder_size` Tauri command：接收路径，注册取消标记，`std::thread::spawn` 启动遍历，完成时 emit `folder-size-done`
- [x] 1.4 实现 `cancel_folder_size` Tauri command：设置对应路径的取消标记
- [x] 1.5 在 `invoke_handler` 中注册两个新命令

## 2. Svelte 前端

- [x] 2.1 更新 `FileInfoPanel.svelte`：添加 `$effect` 在文件夹可见时调用 `invoke('calculate_folder_size')`，监听 `folder-size-tick` 和 `folder-size-done` 事件更新 UI，关闭时触发取消
- [x] 2.2 更新 `DirectoryPanel.svelte`：取消由 FileInfoPanel 的 `$effect` 清理自动处理，无需修改

## 3. 验证

- [x] 3.1 验证文件夹按 `i` 键打开面板后 Size 行数字实时跳动（需运行 app 手动测试）
- [x] 3.2 验证关闭面板后计算被取消（需运行 app 手动测试）
- [x] 3.3 验证包含无权限子目录的文件夹仍能正确计算（需运行 app 手动测试）
- [x] 3.4 运行 `cargo check` 和 `npx svelte-check` 确保无编译错误