## 1. PanelLayout 焦点恢复修复

- [x] 1.1 `handleWindowFocusChanged` 改用 `focusPanel` 代替 `focusPanelNow`（PanelLayout.svelte:823）
- [x] 1.2 `previewPanel` div 的 `tabindex` 从 `-1` 改为 `0`（PanelLayout.svelte:1009）

## 2. ProjectTreePanel toggle 焦点修复

- [x] 2.1 `handleToggleClick` 中将 `panelElement?.focus()` 改为 `setTimeout(() => panelElement?.focus(), 0)`（ProjectTreePanel.svelte:36-39）

## 3. 验证

- [x] 3.1 验证 Alt-Tab 切回后焦点自动恢复，Ctrl+L 第一次按下即生效
- [x] 3.2 验证项目树 toggle 点击后焦点保持，j/k 导航不中断
- [x] 3.3 验证 preview 面板无文件时 Alt-Tab 切回不丢焦点
