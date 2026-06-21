## Why

FileTreePanel 目前只有简单的内联过滤功能，不支持正则表达式、递归搜索和键盘导航。用户需要一个类似 Telescope 的浮窗搜索组件，支持快速模糊搜索文件并导航。

## What Changes

- 新增后端 `search_files` 命令，支持 rg (ripgrep) 优先 + Rust 原生回退
- 新增 `SearchModal.svelte` 浮窗组件，支持正则搜索、j/k 导航、进度显示
- 移除 FileTreePanel 的内联搜索功能，用浮窗替代
- 文件树上方添加可点击的搜索框触发器

## Capabilities

### New Capabilities

- `file-search`: 浮窗文件搜索功能，支持正则表达式、递归搜索、键盘导航

### Modified Capabilities

-

## Impact

- 新增依赖：`regex`, `walkdir` (Cargo.toml)
- 新增 Rust 命令：`search_files`
- 新增组件：`SearchModal.svelte`
- 修改组件：`FileTreePanel.svelte`
