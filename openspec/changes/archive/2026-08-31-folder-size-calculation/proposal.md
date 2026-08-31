## Why

文件夹在目录列表中不显示大小，用户只能看到文件数却无法快速了解文件夹实际占用空间。这类似于 Windows 资源管理器的"属性"功能——选中文件夹后实时展示递归计算的总大小。

## What Changes

- 在 FileInfoPanel 中，对文件夹按 `i` 键时，Size 行从 0 开始实时跳动显示递归计算的大小
- 新增 Rust 后端命令 `calculate_folder_size`，在后台线程中递归遍历目录并定时发回进度
- 新增 Rust 后端命令 `cancel_folder_size`，关闭面板时停止计算
- 进度通过 Tauri 事件 `folder-size-tick` 和 `folder-size-done` 从前端监听

## Capabilities

### New Capabilities
- `folder-size-calculation`: 对文件夹递归计算总大小，并通过 FileInfoPanel 实时展示进度

### Modified Capabilities
<!-- None - this is a new capability, no existing spec requirements change -->

## Impact

- **Rust**: `src-tauri/src/lib.rs` — 新增两个 Tauri command，注册 invoke_handler
- **Svelte**: `src/lib/components/FileInfoPanel.svelte` — 文件夹情况下新增大小实时更新逻辑与取消清理
- **Svelte**: `src/lib/components/DirectoryPanel.svelte` — 传递取消回调，FileInfoPanel 关闭时触发