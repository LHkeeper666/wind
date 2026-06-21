## Why

当前文件搜索（`/`）递归搜索整个子目录树，在大型项目中返回结果过多、响应慢。用户在日常浏览中更常需要的是"在当前目录找某个文件"，而非全局递归搜索。缺少轻量的当前目录搜索和深度递归搜索的区分。

## What Changes

- `/` 快捷键改为**当前目录搜索**（depth 1），只匹配当前目录的直接子项，更快更精准
- 新增 `g/` 快捷键触发**递归深度搜索**（depth 10），覆盖深层项目结构，替代原有 `/` 的递归行为
- 后端 `search_files` 命令增加 `recursive` 参数，`false` 时仅搜索当前目录一层
- `fd` 路径根据模式使用 `--max-depth 1` 或 `--max-depth 10`
- Rust 原生搜索根据模式使用 `depth 1` 或 `depth 10`
- `SearchModal` 增加 `mode` 属性，显示当前搜索范围标签（"Current Directory" / "Recursive"）

## Capabilities

### Modified Capabilities
- `file-search`: 搜索行为从单一递归模式拆分为双模式（当前目录 / 递归），需更新搜索范围、深度、快捷键绑定等需求

### New Capabilities
（无）

## Impact

- **后端**: `src-tauri/src/lib.rs` — `search_files`、`search_with_fd`、`search_with_rust` 函数签名和逻辑变更
- **前端**: `DirectoryPanel.svelte` — 新增 `g/` 快捷键绑定
- **前端**: `SearchModal.svelte` — 新增 `mode` prop 和 UI 标签
- **前端**: `PanelLayout.svelte` — 如果 Ctrl+P 搜索也使用 search_files，需同步适配
- **无破坏性变更**: 现有 `/` 的 UI 交互不变，仅搜索范围收窄
