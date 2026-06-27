## Why

从其他应用切换回 Wind 时，Tauri 窗口获得 OS 级焦点，但 DOM 级焦点丢失——没有任何面板响应键盘输入，用户必须用鼠标点击某个面板才能恢复操作。这打断了键盘驱动的工作流。

## What Changes

- 新增 Tauri 窗口 focus 事件监听：当窗口重获焦点且无面板持有 DOM 焦点时，自动恢复焦点到上次活跃面板
- 新增 `Ctrl+L` 快捷键：手动恢复焦点到上次活跃面板
- 两种方式均显示 toast 提示（如 `Focus: CURRENT`）
- 排除场景：终端 insert 模式下 `Ctrl+L` 透传给 shell；全屏模式不干预；命令面板/文件搜索打开时不拦截

## Capabilities

### New Capabilities
- `focus-restore`: 窗口焦点恢复机制，包括自动恢复（Tauri focus 事件）和手动恢复（Ctrl+L 快捷键）

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- `src/lib/components/PanelLayout.svelte` — 新增 focus 事件监听、Ctrl+L 快捷键处理、焦点检测逻辑
- `src/lib/stores/layout.ts` — 可能需要新增状态字段（如 `focusRestoring` 标志位，防止窗口首次加载时误触发）
