## Why

Vim normal 模式有三个影响日常使用的 bug：鼠标拖拽选中时焦点会意外逃逸到 CodeMirror，大部分 ex 命令（如 `:reg`）不受支持，以及 dd/yy/cc 操作后粘贴出的是过期内容。这三个问题都是已有架构的缺陷，修复代价小但体验改善显著。

## What Changes

- 修复 normal 模式下鼠标选中在编辑器外释放后，焦点逃离 overlay 进入 CodeMirror 的问题
- 增加 `:reg` / `:registers` / `:di` / `:display` 等寄存器查看命令的支持
- 修复 dd/yy/cc 等操作后按 p/P 粘贴出上次内容（而非本次复制内容）的 bug
- FullscreenEditor 增加未知 ex 命令的 `Vim.handleEx` fallback（与 PreviewEditor 一致）

## Capabilities

### New Capabilities
<!-- No new capabilities — all changes are fixes to existing behavior -->

### Modified Capabilities
- `vim-editor`: 新增 `:reg` 等寄存器查看命令；修复 vim 寄存器内容被过期 clipboardCache 覆盖的问题
- `editor-overlay-stable-focus`: 新增文档级 mouseup 监听以确保 overlay 在鼠标选中后保持焦点

## Impact

- `src/lib/components/FullscreenEditor.svelte` — 新增 document mouseup listener；增加 `Vim.handleEx` fallback
- `src/lib/components/PreviewEditor.svelte` — 新增 document mouseup listener
- `src/lib/utils/clipboard-bridge.ts` — `pushText` patch 中同步更新 `clipboardCache`
- `src/lib/utils/vim-commands.ts` — 注册 `:reg` 自定义 ex 命令（或在 editor init 中注册）
