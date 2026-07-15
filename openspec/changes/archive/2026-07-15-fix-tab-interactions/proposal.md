## Why

TabBar 与 PanelLayout 之间缺少通信机制，导致鼠标点击 tab 无法触发完整的状态保存/恢复流程（tab 视觉高亮切换了但面板内容不更新），且 `t r` 重命名快捷键只显示提示而不真正触发重命名。同时 tab 宽度固定导致观感不统一。

## What Changes

- 修复鼠标点击 tab 无法切换的问题：TabBar 通过 callback prop 通知 PanelLayout 执行完整的状态保存/切换/恢复
- `t r` 快捷键改为真正触发当前激活 tab 的内联重命名（而非仅显示 toast 提示）
- Tab 宽度从固定 `flex-shrink: 0` + `max-width: 120px` 改为 `flex: 1 1` 等分可用宽度，配合 `min-width`/`max-width` 约束
- 给 TabBar 添加 `onSwitchTab` 和 `onRenameTab` 两个 callback props

## Capabilities

### New Capabilities
- `tab-rename-shortcut`: 键盘快捷键 `t r` 触发当前激活 tab 的内联重命名
- `responsive-tab-bar`: tab 标签根据窗口宽度动态等分可用空间

### Modified Capabilities
- `tab-state-persistence`: 鼠标点击 tab 也触发状态保存/恢复（修复 bug——spec 要求切换 tab 时保存恢复，但鼠标点击未覆盖此流程）

## Impact

- `src/lib/components/TabBar.svelte` — 添加 props + CSS 改动
- `src/lib/components/PanelLayout.svelte` — 提供 callback 实现 + `t r` handler 改为触发重命名
