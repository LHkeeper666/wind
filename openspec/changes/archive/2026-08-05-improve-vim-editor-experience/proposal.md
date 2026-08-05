## Why

当前 vim 编辑器的自动缩进为 2 空格（CodeMirror 6 默认 indentUnit），不符合 Python/大部分语言的 4 空格惯例。同时编辑器对第三方 Python 库没有任何补全提示，只能补全关键字，编辑体验不够好。

## What Changes

- **自动缩进统一为 4 空格**：通过 `indentUnit.of('    ')` 设置 CodeMirror 的缩进单位为 4 空格，影响自动缩进（回车后）、`=` 命令（vim `=` 缩进）和 >/< 缩进命令
- **Python Level 2 补全**：Rust 后端发现已安装的第三方包（通过 `pip list`），动态导入提取 API + 函数签名，缓存到磁盘；前端解析 import 语句追踪别名，通过 CodeMirror CompletionSource 提供补全

## Capabilities

### Modified Capabilities
- `vim-editor`：自动缩进从 2 空格改为 4 空格，新增 Python 第三方库补全

### New Capabilities
- `python-completion`：Python 第三方库 API 发现与 CodeMirror 补全集成

## Impact

- **新增文件**：
  - `src-tauri/src/python_completion.rs` — 后端：pip 扫描 + 动态导入签名提取 + JSON 缓存
  - `src/lib/completions/python-completion.ts` — 前端：CompletionSource + import 别名解析
- **修改文件**：
  - `src-tauri/src/lib.rs` — 注册新 Tauri 命令
  - `src-tauri/Cargo.toml` — 新增 `serde_json` 依赖（如未引入）
  - `src/lib/components/PreviewEditor.svelte` — 加 `indentUnit` + `pythonCompletion` 到 extensions
  - `src/lib/components/FullscreenEditor.svelte` — 同上
- **新增依赖**：无（前后端均复用现有依赖）
