## 1. 自动缩进 4 空格

- [ ] 1.1 `PreviewEditor.svelte`：从 `@codemirror/language` 导入 `indentUnit`，在 extensions 中添加 `indentUnit.of('    ')`
- [ ] 1.2 `FullscreenEditor.svelte`：同上

## 2. Python 补全 — 后端

- [ ] 2.1 新建 `src-tauri/src/python_completion.rs`：
  - `get_pip_list(python_exe)` → 运行 `pip list --format=json`，返回包名和版本
  - `extract_package_api(python_exe, package_name)` → 运行 Python 脚本提取公开 API 和签名
  - `scan_all(python_exe)` → 遍历所有包，提取 API，返回 JSON
  - 缓存读写：`%APPDATA%/wind/python-completions/` 下 JSON 文件
- [ ] 2.2 `src-tauri/src/lib.rs`：注册 `scan_python_completions` Tauri 命令
- [ ] 2.3 `src-tauri/Cargo.toml`：检查是否需要添加 `serde` / `serde_json` 依赖

## 3. Python 补全 — 前端

- [ ] 3.1 新建 `src/lib/completions/python-completion.ts`：
  - `parseImports(doc: Text)` → 解析 import 语句，返回 `Map<string, string>` (alias → package)
  - `pythonCompletionSource(context)` → CodeMirror CompletionSource，识别 alias 并触发后端查询
  - 结果缓存在前端内存中（Map<package, Completion[]>），避免重复 invoke
- [ ] 3.2 `PreviewEditor.svelte`：在 Python 文件编辑时将 `pythonCompletionSource` 加入 extensions
- [ ] 3.3 `FullscreenEditor.svelte`：同上

## 4. 验证

- [ ] 4.1 运行 `npx svelte-check` 确认无新增类型错误
- [ ] 4.2 运行 `cargo check` 确认 Rust 编译通过
- [ ] 4.3 编辑 Python 文件验证自动缩进为 4 空格
- [ ] 4.4 编辑 Python 文件验证第三方库补全（如 `numpy.` 触发 API 列表）
