## Why

在预览面板编辑文本的 insert 模式中，Alt 标签页快捷键被整体禁用，用户必须退出 insert 才能切换工作区或调整标签顺序。Ctrl 预览槽快捷键虽有入口，但切换后能否保持真实的 Vim insert 状态、光标和焦点尚缺少完整保障。

## What Changes

- 允许在预览面板的 editor-insert 模式中使用 `Alt+H/L/M/1-9` 切换工作区 tab，使用 `Ctrl+H/L` 切换预览槽。
- 允许同一模式中使用 `Alt+,/.` 调整当前 tab 顺序，使用 `Ctrl+,/.` 调整当前预览槽顺序；排序操作保持当前编辑器身份及输入状态。
- 切离并返回编辑槽时恢复独立草稿、Vim insert 状态、光标选区、滚动位置和输入焦点，避免快速切换的过期异步操作覆盖当前目标。
- 保持命令面板、搜索、媒体全屏、终端输入等现有按键作用域。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `workspace-preview-slots`：明确 insert 模式中的预览槽切换、排序及状态恢复。
- `tab-state-persistence`：明确 insert 模式中的工作区切换和独立编辑器会话恢复。
- `focus-restore`：明确快捷键切换后 insert 输入焦点的交付与异步保护。

## Impact

涉及前端全局快捷键、`PanelLayout` 的 tab/槽切换入口、`PreviewEditor` 与 `TextEditorHost` 的会话恢复，以及快捷键帮助和相关前端验证。无 Rust、数据库、依赖或持久化格式变更。当前仍存在独立全屏文本编辑器；本变更只覆盖预览面板编辑。
