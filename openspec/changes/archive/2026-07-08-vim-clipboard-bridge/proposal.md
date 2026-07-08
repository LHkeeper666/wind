## Why

当前使用 vim 模式编辑文件时，`y`(yank)/`d`(delete)/`p`(put) 等操作使用的是 `@replit/codemirror-vim` 的内部 register 系统，与系统剪贴板完全隔离。用户在 vim 中 yank 的文本无法 Ctrl+V 粘贴到其他应用，其他应用 Ctrl+C 的文本也无法在 vim 中用 `p` 粘贴。这违背了 vim `set clipboard=unnamed` 的用户预期。

## What Changes

- 新增 `clipboard-bridge.ts` 工具模块，桥接 vim register 和系统剪贴板
- yank/delete 操作自动同步到系统剪贴板（`navigator.clipboard.writeText()`）
- `p`/`P` 操作自动从系统剪贴板读取（`navigator.clipboard.readText()`，通过焦点预读缓存解决 async 问题）
- 两个编辑器组件（PreviewEditor、FullscreenEditor）在 vim 初始化后接入桥接

## Capabilities

### New Capabilities
- `vim-clipboard`: vim 编辑器与系统剪贴板的双向同步

### Modified Capabilities
- `editor`: PreviewEditor 和 FullscreenEditor 增加剪贴板桥接调用

## Impact

- 新增 `src/lib/utils/clipboard-bridge.ts`
- 修改 `src/lib/components/PreviewEditor.svelte`（initEditor 中调用桥接）
- 修改 `src/lib/components/FullscreenEditor.svelte`（initEditor 中调用桥接）
- 纯前端变更，无 Rust 后端修改
- 无 API 变更
