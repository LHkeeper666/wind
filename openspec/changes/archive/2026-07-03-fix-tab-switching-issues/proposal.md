## Why

Tab 切换功能存在三个相互关联的 bug，严重影响多 tab 工作流的可用性：(1) terminal normal 模式下 `t` 前缀快捷键完全失效；(2) 切换到有 terminal 的 tab 时焦点状态不一致，导致 j/k 导航失效；(3) tab 切换时 selectedFile 等状态丢失，每次切换都重新加载预览内容。

## What Changes

- 将 `t` 前缀 tab 操作从 DirectoryPanel 私有逻辑提取到全局 keydown 处理层，使 terminal normal 模式下也能使用 `t n/p/c` 等快捷键
- 修复 `restoreTabAndFocus()` 中的焦点竞争问题：terminal 可见时不再出现 store 状态与 DOM 焦点不一致的情况
- 修复 tab 状态保存/恢复：`saveActiveTabState()` 正确保存 selectedFile/cursorIndex/scrollOffset，`restoreTabAndFocus()` 正确恢复而不被 `setCurrentPath` 覆盖
- 修复 `restoreTabAndFocus()` 中 terminal 可见时焦点分配逻辑，避免 terminal 50ms 后抢焦点

## Capabilities

### New Capabilities
- `tab-state-persistence`: Tab 状态保存与恢复，包括 selectedFile、cursorIndex、scrollOffset 的持久化

### Modified Capabilities
- `terminal-panel-navigation`: `t` 前缀 tab 操作需要在全局 keydown 层处理，而非仅在 DirectoryPanel 内部；terminal normal 模式下应支持 tab 切换
- `focus-restore`: tab 切换时焦点恢复逻辑需要与 terminal 可见状态协调，避免焦点竞争

## Impact

- `src/lib/components/PanelLayout.svelte` — `handleGlobalKeydown` 增加 `t` 前缀处理；`restoreTabAndFocus` 重写焦点恢复逻辑
- `src/lib/components/FloatingTerminal.svelte` — 移除 terminal 可见时自动 `terminal.focus()` 的 50ms 延迟逻辑，改为由 `focusPanel` 统一控制
- `src/lib/components/DirectoryPanel.svelte` — 移除私有 `t` 前缀处理（迁移到全局）
- `src/lib/stores/tabs.ts` — `saveActiveTabState` 增加 cursorIndex/scrollOffset 保存；`restoreActiveTabState` 增加 selectedFile 恢复
- `src/lib/stores/layout.ts` — `setCurrentPath` 不再重置 selectedFile 为 null
