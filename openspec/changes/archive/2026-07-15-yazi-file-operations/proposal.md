## Why

Wind 已实现 yazi 风格的 y/x/p 跨 tab 复制剪切，但还缺少多个核心文件操作：强制粘贴、删除确认、重命名快捷键、新建快捷键、隐藏文件切换、回收站支持。这些是文件管理器的基础交互，需要补齐。

## What Changes

- 新增 `P` 键：强制粘贴（跳过冲突确认，直接覆盖）
- 新增 `d` 键：删除文件/目录（带确认对话框）
- 新增 `r` 键：重命名（内联输入框）
- 新增 `a` 键：新建文件或目录（内联输入框，支持 `a` 文件名和 `a/` 目录名）
- 新增 `.` 键：切换隐藏文件显示/隐藏
- 新增 `D` 键：永久删除（带确认，区别于回收站）
- 后端 FileEntry 增加 `is_hidden` 字段
- 后端引入 `trash` crate，`d` 键移到回收站而非硬删除
- 新建 InputDialog 组件（内联输入框，用于重命名和新建）

## Capabilities

### New Capabilities
- `input-dialog`: 内联输入框组件，支持重命名和新建文件/目录
- `hidden-files`: 隐藏文件过滤，支持 `.` 键切换显示
- `trash-support`: 回收站支持，`d` 移到回收站，`D` 永久删除
- `force-paste`: `P` 强制粘贴，跳过冲突确认直接覆盖
- `file-delete`: `d` 删除操作（带确认）
- `file-rename`: `r` 重命名操作（内联输入框）
- `file-create`: `a` 新建操作（内联输入框）

### Modified Capabilities
- `file-clipboard`: 新增 `P` 强制粘贴行为（跳过冲突确认）

## Impact

- **前端新增**: `src/lib/components/InputDialog.svelte`（内联输入框组件）
- **前端修改**: `DirectoryPanel.svelte`（新快捷键 + 隐藏文件过滤）、`PanelLayout.svelte`（删除/重命名/新建逻辑）、`keybindings.ts`
- **后端修改**: `src-tauri/src/lib.rs`（FileEntry 加 is_hidden、delete_file 改用 trash）
- **后端新增依赖**: `trash = "4"`（Cargo.toml）
- **依赖**: 无新前端依赖
