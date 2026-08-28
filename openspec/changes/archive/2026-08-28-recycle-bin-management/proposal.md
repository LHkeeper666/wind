## Why

Wind 已支持通过 `d` 键将文件移入回收站，但用户无法查看回收站内容，也无法还原误删的文件。如果用户需要还原文件，必须切换到 Windows 资源管理器操作，打断了 vim 风格的工作流。

## What Changes

- 新增回收站视图，以快捷键 `gr` 进入，展示回收站中的文件列表
- 支持还原回收站中的文件到原路径（`r` 键）
- 支持永久删除回收站中的文件（`d` 键）
- 支持清空整个回收站（`gd` 键）
- 回收站文件预览（文本、图片、PDF 等），与普通文件预览一致
- 三栏布局中，parent 栏显示回收站统计和快捷操作提示，current 栏显示回收站文件列表

## Capabilities

### New Capabilities

- `recycle-bin`: 回收站管理——查看、还原、永久删除、清空回收站中文件的能力

### Modified Capabilities

<!-- No existing capabilities need spec-level changes -->

## Impact

- **Rust 后端**: 新增 4 个 Tauri command（`list_recycle_bin`、`restore_recycle_items`、`purge_recycle_items`、`empty_recycle_bin`），依赖已有的 `trash` crate
- **前端**: 新增 `RecycleBinPanel.svelte` 组件，修改 `PanelLayout.svelte` 以支持回收站模式切换，修改 `keybindings.ts` 添加快捷键声明
- **依赖**: 无新增依赖，`trash` crate v4.1.1 已完整支持所需 API