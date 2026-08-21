## Why

当前 terminal 虽然按 tab 独立维护，但 shell 的启动时机和 cwd 取值容易与 tab 的目录状态耦合，无法稳定表达“每个 tab 首次打开 terminal 时使用当时 directory panel 所在目录”。同时，全屏 terminal 使用覆盖整个窗口的 fixed overlay，会遮挡标题栏和 TabBar，导致用户无法通过鼠标直接切换 tab。

## What Changes

- 将每个 tab 的 terminal shell 启动延迟到该 tab 首次真正显示 terminal 时。
- 首次启动时记录 directory panel 的 `currentPath` 作为该 tab 的初始 terminal cwd，后续不因 directory panel 导航而改变。
- shell 重启或切换 shell 时优先沿用 terminal 自身通过 shell integration 记录的当前目录，并在不可用时回退到该 tab 的初始 cwd。
- 调整全屏 terminal 的布局边界，使其只覆盖标题栏和 TabBar 下方的内容区。
- 全屏 terminal 保留标题栏、TabBar 和底部状态栏，允许通过点击 TabBar 切换 tab。
- 保留并明确全屏 terminal 下 normal 模式的 tab 快捷键操作和 Ctrl+L 焦点恢复行为。

## Capabilities

### New Capabilities

无

### Modified Capabilities

- `per-tab-terminal`: 增加 terminal 懒启动、首次 cwd 快照和 shell 重启 cwd 保持要求。
- `fullscreen-terminal`: 将全屏范围从整个窗口调整为内容区，并增加标题栏、TabBar、状态栏可见以及点击 TabBar 切换 tab 的要求。
- `terminal-panel-navigation`: 明确全屏 terminal normal 模式下仍可使用 tab 前缀快捷键，且 TabBar 鼠标切换不受 terminal overlay 拦截。

## Impact

- 前端状态与生命周期：`src/lib/stores/tabs.ts`、`src/lib/components/FloatingTerminal.svelte`、`src/lib/terminal/terminal-manager.ts`。
- 前端布局与导航：`src/lib/components/PanelLayout.svelte`、相关 terminal/fullscreen 样式。
- 现有 Tauri `terminal_spawn` API 保持参数形式不变，但调用时机和 cwd 来源会调整。
- 需要通过 `npx svelte-check`、`cargo check` 以及启动应用后的手动交互验证确认行为。
