## Why

当前所有 tab 共享一个 FloatingTerminal 单例和一个 shell 进程。切换 tab 时 terminal 内容保持不变，无法在不同 tab 中运行独立的命令会话。每个 tab 拥有独立终端可以提升多 tab 工作流的实用性。

## What Changes

- 新增 `TerminalInstance` 接口，封装单个 terminal 实例（xterm.js Terminal + shell 进程 + DOM 容器）
- 新增 `TerminalManager` 模块，管理多个 terminal 实例的生命周期
- `FloatingTerminal` 从单例组件改为根据当前 tab ID 显示对应的 terminal 实例
- `TabState` 增加 terminal 实例相关状态
- tab 切换时隐藏当前 terminal DOM、显示目标 terminal DOM，不销毁/重建
- tab 关闭时销毁对应的 terminal 实例和 shell 进程

## Capabilities

### New Capabilities

- `per-tab-terminal`: 每个 tab 拥有独立的 terminal 实例和 shell 进程

### Modified Capabilities

- `fullscreen-terminal`: 全屏终端需要适配多实例架构
- `tab-state-persistence`: Tab 状态需要包含 terminal 实例引用

## Impact

- `src/lib/components/FloatingTerminal.svelte` — 重构为多实例管理
- `src/lib/components/PanelLayout.svelte` — terminal 切换逻辑适配
- `src/lib/stores/tabs.ts` — TabState 增加 terminal 实例状态
- `src/lib/terminal/` — 可能需要新增 TerminalManager 模块
