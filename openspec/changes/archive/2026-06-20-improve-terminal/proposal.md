## Why

当前终端功能存在两个关键问题：
1. Ctrl+` 快捷键绑定在 `app-layout` div 元素上，只有焦点在应用内部时才有效，且焦点在终端内部时会被 xterm.js 拦截
2. 终端缺少 shell 集成，无法追踪命令执行状态、检测当前目录变化等基本功能

## What Changes

- 将 Ctrl+` 快捷键改为全局监听（`window.addEventListener`），确保在任何焦点状态下都能切换终端
- 修复终端内部的快捷键冲突，确保 Escape 和 Ctrl+` 在终端内也能正常工作
- 实现 shell 集成协议，支持：
  - 命令执行追踪（OSC 133 协议）
  - 当前目录检测（OSC 7/OSC 1337 协议）
  - 命令完成通知
  - 提示符标记

## Capabilities

### New Capabilities
- `terminal-shell-integration`: Shell 集成协议实现，包括 OSC 序列解析、命令追踪、目录同步

### Modified Capabilities
- 无

## Impact

- 前端：`PanelLayout.svelte` - 快捷键监听改为全局
- 前端：`FloatingTerminal.svelte` - 添加 shell 集成事件处理
- 后端：`src-tauri/src/terminal/mod.rs` - 可能需要支持 OSC 序列透传
- 依赖：可能需要引入 `@xterm/addon-serialize` 用于 OSC 解析
