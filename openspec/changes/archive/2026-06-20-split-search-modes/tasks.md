## 1. 后端：search_files 参数扩展

- [x] 1.1 `search_files` 命令增加 `recursive: Option<bool>` 参数，默认 `true`（保持向后兼容）
- [x] 1.2 `search_with_fd` 增加 `max_depth` 参数，替换硬编码的 `"3"`，当前目录模式传 `1`，递归模式传 `10`
- [x] 1.3 `search_with_rust` 增加 `max_depth` 参数，替换硬编码的 `5`，当前目录模式传 `1`，递归模式传 `10`
- [x] 1.4 `search_with_rg`（实际调用 `search_with_rust`）同步传递 `max_depth` 参数

## 2. 前端：SearchModal 模式支持

- [x] 2.1 SearchModal 增加 `mode: 'current' | 'recursive'` prop，默认 `'current'`
- [x] 2.2 SearchModal 根据 mode 向 `search_files` 传递 `recursive` 参数（`'current'` → `false`，`'recursive'` → `true`）
- [x] 2.3 SearchModal 在搜索输入框旁显示模式标签（"Current Directory" / "Recursive"）

## 3. 前端：DirectoryPanel 快捷键

- [x] 3.1 DirectoryPanel 的 `handleKeydown` 增加 `g/` 组合键检测（`g` pending 状态 + `/` 触发）
- [x] 3.2 `g/` 打开 SearchModal 时传入 `mode='recursive'`，`/` 传入 `mode='current'`
- [x] 3.3 新增 `searchMode` 状态变量，控制 SearchModal 的 mode prop

## 4. 验证

- [x] 4.1 验证 `/` 搜索仅返回当前目录直接子项
- [x] 4.2 验证 `g/` 搜索返回递归子目录结果
- [x] 4.3 验证 SearchModal 模式标签正确显示
- [x] 4.4 验证 `gg` 跳顶部功能不受影响
- [x] 4.5 运行 `cargo check` 和 `npx svelte-check` 确保无编译错误
