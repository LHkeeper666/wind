## Context

Wind 是一个 vim 驱动的 Windows 文件工作站，当前浮动终端在底部显示，高度有限。用户需要全屏终端模式来专注于终端操作。

现有架构关键点：
- `FloatingTerminal.svelte` 是浮动终端组件，支持 insert/normal 模式
- `FullscreenEditor.svelte` 是全屏覆盖层的现有模式（fixed + z-index:1000）
- `PanelLayout.svelte` 处理全局键盘事件，包括 `Ctrl+`` 切换终端、`Ctrl+W` 面板切换
- Layout store 管理所有 UI 状态，包括 `fullscreenEditorOpen`、`fullscreenImageViewerOpen` 等

## Goals / Non-Goals

**Goals:**
- 新增终端全屏模式，`Ctrl+Shift+`` 切换
- 全屏时 100% 覆盖窗口
- 全屏时禁用 `Ctrl+W` 面板切换
- `Ctrl+`` 在全屏时关闭终端（退出全屏 + 隐藏）
- 保持 insert/normal 模式状态

**Non-Goals:**
- 不创建新的 FullscreenTerminal 组件（复用 FloatingTerminal）
- 不修改终端的 shell 类型切换逻辑
- 不添加全屏时的额外 UI 元素（shell selector、目录路径等）

## Decisions

### 1. 实现方式：修改 FloatingTerminal 支持全屏状态

**决定**：在 FloatingTerminal 中添加 `fullscreen` prop，全屏时使用 `position: fixed; inset: 0; z-index: 1000`。

**理由**：复用现有组件，xterm.js 实例保持不变，无需重建终端。与创建新组件相比，代码更简洁。

**替代方案**：创建独立的 FullscreenTerminal 组件。被否决，因为需要传递或重建 xterm.js 实例，增加复杂度。

### 2. 状态管理：在 layout store 中添加 `fullscreenTerminalOpen`

**决定**：与 `fullscreenEditorOpen`、`fullscreenImageViewerOpen` 等状态一致，在 layout store 中添加 `fullscreenTerminalOpen: boolean`。

**理由**：保持状态管理的一致性，便于在 PanelLayout 中统一处理全屏状态。

### 3. 快捷键处理：全屏时禁用 Ctrl+W 面板切换

**决定**：在 `handleGlobalKeydown` 中检查 `fullscreenTerminalOpen` 状态，如果为 true 则跳过 `Ctrl+W` 的 h/l/j/k 处理。

**理由**：全屏终端时用户看不到其他面板，面板切换没有意义。逻辑简单，用户容易理解：想切换面板先退出全屏。

### 4. Ctrl+` 行为扩展：全屏时关闭终端

**决定**：`Ctrl+`` 在全屏终端时的行为改为"关闭终端"（退出全屏 + 隐藏终端），而不是普通的切换显示/隐藏。

**理由**：用户期望 `Ctrl+`` 是"关闭终端"的快捷键，全屏时也应该有关闭功能。`Ctrl+Shift+`` 负责"退出全屏但保持终端可见"。

## Risks / Trade-offs

- **xterm.js resize** → 全屏切换时需要触发 `fitAddon.fit()` 重新适配终端尺寸。FloatingTerminal 已有 ResizeObserver 处理此情况。
- **z-index 层级** → 全屏终端使用 z-index: 1000，与 FullscreenEditor 等一致。需要确保不与命令面板（z-index: 1000）冲突。
- **状态同步** → 退出全屏时需要恢复终端高度。可以通过保存/恢复 `terminalHeight` 实现。
