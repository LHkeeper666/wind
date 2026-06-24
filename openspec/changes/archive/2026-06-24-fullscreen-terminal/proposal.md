## Why

当前浮动终端在底部显示，高度有限，执行复杂命令或查看大量输出时不够方便。用户需要一个全屏终端模式，专注于终端操作，再次按下快捷键即可回到原布局。

## What Changes

- 新增 `fullscreenTerminalOpen` 状态到 layout store
- 修改 `FloatingTerminal.svelte` 支持全屏模式（`position: fixed; inset: 0`）
- 在 `PanelLayout.svelte` 中添加 `Ctrl+Shift+`` 快捷键切换终端全屏
- 全屏终端时禁用 `Ctrl+W` 面板切换（h/l/j/k 均不响应）
- `Ctrl+`` 在全屏时行为：关闭终端（退出全屏 + 隐藏终端）
- `Ctrl+Shift+`` 在全屏时行为：退出全屏，终端仍然可见

## Capabilities

### New Capabilities
- `fullscreen-terminal`: 终端全屏模式，支持 Ctrl+Shift+` 切换，Ctrl+` 关闭

### Modified Capabilities
(无现有 spec 需要修改)

## Impact

- 修改文件：`src/lib/stores/layout.ts`、`src/lib/components/FloatingTerminal.svelte`、`src/lib/components/PanelLayout.svelte`
- 无新依赖
- 快捷键变更：
  - `Ctrl+Shift+`` — 切换终端全屏（新）
  - `Ctrl+`` — 全屏时关闭终端（行为扩展）
