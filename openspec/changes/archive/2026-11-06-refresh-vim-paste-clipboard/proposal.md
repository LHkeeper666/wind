## Why

在 PDF 标签页复制文本后切到 Markdown 编辑标签页，Vim 普通模式的 `p` 可能粘贴上一次复制的内容；Insert 模式的 `Ctrl+V` 却能粘贴最新内容。普通模式依赖未及时刷新的剪贴板缓存，导致同一次系统复制在两种模式下结果不同。

## What Changes

- 普通模式执行默认寄存器的 `p/P` 前读取本次系统剪贴板内容，并在读取完成后执行 Vim 粘贴。
- 处理剪贴板为空、读取失败、连续按键和等待期间切换标签页的情况，避免旧缓存或异步回调将内容粘贴到错误文件。
- 保留显式寄存器粘贴及 `yy`、`dd` 等本地 Vim 操作后的粘贴语义。
- 在现存的面板编辑器与全屏编辑器入口应用相同的修复；若全屏编辑器退役的并行 change 已先落地，则只处理仍存在的入口。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `vim-editor`：默认 `p/P` 必须使用本次可读取的系统剪贴板内容，并保持寄存器、空剪贴板和标签页切换时的正确行为。

## Impact

- 前端剪贴板桥接及 Vim 按键入口：`src/lib/utils/clipboard-bridge.ts`、`src/lib/components/VimOverlay.svelte`、当前仍存在的 `src/lib/components/FullscreenEditor.svelte`。
- 无 Rust 命令、依赖、数据库或用户配置变更。
