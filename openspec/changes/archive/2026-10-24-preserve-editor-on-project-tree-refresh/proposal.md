## Why

在 project mode 中编辑 Markdown 并执行 `:w` 后，保存产生的文件系统事件会刷新项目树。刷新时重新通知当前选中项，意外打开只读选中预览层，遮住仍处于编辑模式的预览位。用户会误以为保存导致退出编辑器。

## What Changes

- 项目树自动刷新和手动刷新恢复选中路径时，仅更新树的选中位置，不触发用户选择事件。
- 保留用户主动移动项目树选中项时的只读选中预览行为。
- 覆盖保存 Markdown 后的编辑器可见性与项目树选中状态验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `project-tree-auto-refresh`: 刷新恢复选中路径时不触发选中预览。
- `selection-preview-overlay`: 仅由用户主动选择目录项显示选中预览层；后台刷新不应使其出现。

## Impact

涉及 `src/lib/composables/project-tree.svelte.ts` 中刷新后的选中恢复，以及项目树和选中预览层的行为验证。不涉及后端接口或新增依赖。
