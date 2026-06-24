## 1. Layout Store 扩展

- [x] 1.1 在 `layout.ts` 的 `LayoutState` 接口中添加 `fullscreenTerminalOpen: boolean` 字段
- [x] 1.2 在 `initialState` 中设置 `fullscreenTerminalOpen: false`
- [x] 1.3 添加 `openFullscreenTerminal()`、`closeFullscreenTerminal()` 和 `toggleFullscreenTerminal()` 方法

## 2. FloatingTerminal 全屏支持

- [x] 2.1 添加 `fullscreen` prop（默认 false）
- [x] 2.2 全屏时使用 `position: fixed; inset: 0; z-index: 1000` 样式
- [x] 2.3 全屏时隐藏拖拽手柄（terminal-handle）

## 3. PanelLayout 快捷键处理

- [x] 3.1 在 `handleGlobalKeydown` 中添加 `Ctrl+Shift+`` 处理，调用 `layout.toggleFullscreenTerminal()`
- [x] 3.2 修改 `Ctrl+`` 处理：全屏时调用关闭逻辑（退出全屏 + 隐藏终端）
- [x] 3.3 修改 `Ctrl+W` 处理：全屏终端时跳过面板切换逻辑
- [x] 3.4 将 `fullscreenTerminalOpen` 状态传递给 FloatingTerminal 组件

## 4. 验证

- [x] 4.1 运行 `npx svelte-check` 确认类型检查通过
- [x] 4.2 运行 `cargo check` 确认 Rust 侧无影响
