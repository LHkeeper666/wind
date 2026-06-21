## 1. TerminalPanel 初始化修复

- [x] 1.1 将 `TerminalPanel.svelte` 中 `mode` 初始值从 `'insert'` 改为 `'normal'`
- [x] 1.2 将 `initTerminal()` 中 Terminal 构造函数的 `disableStdin` 从 `false` 改为 `true`
- [x] 1.3 从 `startShell()` 中移除 `terminal.focus()` 调用，防止启动时抢夺焦点

## 2. PanelLayout 面板切换修复

- [x] 2.1 从 `switchPanel()` 中移除进入终端时的 `setMode('insert')` 调用
- [x] 2.2 从终端面板的 `onclick` 处理器中移除 `setMode('insert')` 调用

## 3. 验证

- [x] 3.1 运行 `npx svelte-check` 确认无类型错误
- [ ] 3.2 运行 `npm run tauri dev` 验证：启动后状态栏显示 NORMAL，Ctrl+W 面板切换立即可用，切换到终端后按 'i' 进入插入模式（需手动验证）
