## Why

最近的重构（提取 DirectoryPanel composables、拆分 PanelLayout 模块）后，焦点管理出现两个回归问题：
1. Alt-Tab 切回 Wind 窗口后焦点丢失，Ctrl+L 快捷键第一次按下也失效
2. 项目树模式下，鼠标点击展开控件后焦点丢失，展开控件可点击区域变小

这些问题导致用户必须多次按 Ctrl+L 或点击面板才能恢复键盘操作，严重影响 vim-driven 工作流的流畅性。

## What Changes

- `handleWindowFocusChanged` 改用 `focusPanel` 代替 `focusPanelNow`，获得 tick 重试机制
- `focusPanelNow` 的 preview 分支增加 fallback，当 `.preview-editor` 元素不可聚焦时回退到 `previewPanel` 本身
- `previewPanel` 的 `tabindex` 从 `-1` 改为 `0`，使其可作为焦点 fallback
- ProjectTreePanel 的 toggle 点击后增加延迟焦点恢复，确保 Svelte 重渲染完成后再 focus

## Capabilities

### Modified Capabilities

- `focus-restore`: 修复 `handleWindowFocusChanged` 使用 `focusPanelNow` 缺少重试机制的问题；修复 preview 面板焦点恢复的静默失败
- `project-tree-focus-management`: 修复 toggle 展开/折叠后焦点丢失的问题

### New Capabilities

（无）

## Impact

- `src/lib/components/PanelLayout.svelte` — `handleWindowFocusChanged`、`focusPanelNow` 函数、`previewPanel` 元素的 tabindex
- `src/lib/components/ProjectTreePanel.svelte` — `handleToggleClick` 函数
- 无 API 变更，无依赖变更，纯前端焦点管理修复
