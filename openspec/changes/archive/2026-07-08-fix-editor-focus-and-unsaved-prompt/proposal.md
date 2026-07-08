## Why

编辑器中存在两个体验问题：(1) 鼠标点击编辑区外部后再点回编辑区，键盘输入无法恢复，焦点被路由到了外层 div 而非 CodeMirror 的 contentDOM；(2) 编辑文件时有未保存修改，从左侧目录面板打开其他文件会静默丢弃修改，没有保存提示。

## What Changes

- **修复编辑器焦点路由**：当 preview panel 获得焦点且处于编辑模式（editor-normal / editor-insert）时，自动将焦点转发到 CodeMirror overlay 或 contentDOM，确保键盘输入始终有效
- **新增未保存修改提示**：在编辑文件过程中切换文件时，弹出确认对话框，提供保存 / 放弃 / 取消三个选项

## Capabilities

### New Capabilities
- `editor-unsaved-prompt`: 编辑文件时有未保存修改，切换文件或关闭 tab 时弹出确认对话框

### Modified Capabilities
- `focus-restore`: 编辑模式下焦点恢复行为需要变更——panel 焦点应转发到 overlay 或 CodeMirror contentDOM 而非外层 div

## Impact

- `src/lib/components/PreviewEditor.svelte` — 添加 panelElement onfocus 处理，暴露 isModified getter
- `src/lib/components/PanelLayout.svelte` — handleActivate 中添加脏状态检查，新增未保存确认对话框状态
