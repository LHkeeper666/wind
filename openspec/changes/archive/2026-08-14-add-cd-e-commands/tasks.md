## 1. cd 命令实现

- [x] 1.1 在 `handleCommandKeydown` 中添加 `cd` 命令解析分支
- [x] 1.2 处理空参数 `cd` → 调用 `get_home_dir()` 并导航
- [x] 1.3 处理相对路径：基于 `currentPath` 拼接
- [x] 1.4 处理绝对路径：直接使用
- [x] 1.5 调用 `read_directory` 验证路径存在，失败时 toast 提示 `E344: Can't find directory`

## 2. e 命令实现

- [x] 2.1 在 `handleCommandKeydown` 中添加 `e` 命令解析分支
- [x] 2.2 空参数 `e` → 调用 `currentDirectoryPanel.refresh()` 刷新当前目录
- [x] 2.3 路径是目录 → 调用 `handleNavigate`（相同路径时用 refresh）
- [x] 2.4 路径是文件 → 调用 `handleSelect` 并切换焦点到 preview

## 3. Tab 自动补全

- [x] 3.1 添加补全状态变量：`completions`、`completionIndex`、`completionPrefix`、`completionDir`
- [x] 3.2 实现路径解析函数：分离命令名和路径部分
- [x] 3.3 实现补全逻辑：按 Tab 时调用 `read_directory` 获取候选
- [x] 3.4 实现轮询：多次 Tab 循环匹配项
- [x] 3.5 `cd` 补全只过滤 is_dir 条目
- [x] 3.6 `e` 补全过滤文件+目录
- [x] 3.7 输入变化时重置补全状态

## 4. Rust 后端

- [x] 4.1 添加 `file_exists` Tauri 命令用于 `:e` 文件验证
- [x] 4.2 注册到 `invoke_handler`

## 5. DirectoryPanel 扩展

- [x] 5.1 添加 `export function refresh()` 方法（调用 `loadDirectory(path, true)`）

## 6. 验证

- [x] 6.1 运行 `npx svelte-check` 确认类型检查通过（无新增 error）
- [x] 6.2 运行 `cargo check` 确认 Rust 侧无影响
