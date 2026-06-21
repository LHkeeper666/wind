## 1. 全局快捷键修复

- [x] 1.1 修改 `PanelLayout.svelte`，将 `handleGlobalKeydown` 从 div 元素绑定改为 `window.addEventListener` 全局监听
- [x] 1.2 在 `onMount` 中注册全局 keydown 事件，在 `onDestroy` 中移除
- [x] 1.3 修改 `FloatingTerminal.svelte`，拦截 Ctrl+` 事件防止传递到 shell
- [ ] 1.4 测试：焦点在目录面板、终端内部、命令面板时 Ctrl+` 都能正常切换

## 2. Shell 集成 - OSC 解析器

- [x] 2.1 创建 `src/lib/terminal/shell-integration.ts` 模块
- [x] 2.2 实现 OSC 序列解析器，支持 OSC 7（目录同步）和 OSC 133（命令追踪）
- [x] 2.3 实现状态管理：命令执行状态、当前目录、退出码
- [x] 2.4 在 `FloatingTerminal.svelte` 中集成 OSC 解析器，拦截 terminal.onData 输出

## 3. Shell 集成 - UI 展示

- [x] 3.1 在 `FloatingTerminal.svelte` 添加状态栏，显示当前目录和命令状态
- [x] 3.2 实现命令退出码非 0 时的错误指示样式
- [x] 3.3 目录变化时触发事件通知主应用（可选，用于同步目录面板）

## 4. 测试与文档

- [ ] 4.1 编写 OSC 解析器的单元测试
- [x] 4.2 更新 CLAUDE.md 中的键盘快捷键文档
- [x] 4.3 添加 shell 集成启用说明（PowerShell oh-my-posh / bash starship 配置）
