## 1. Rust 后端

- [x] 1.1 新增 `ShellOutput` 结构体（stdout, stderr, exit_code）
- [x] 1.2 新增 `detect_bash_path()` 辅助函数，实现 bash 路径 fallback 检测
- [x] 1.3 新增 `exec_shell_command` Tauri 命令，使用 bash 执行命令并捕获输出
- [x] 1.4 在 `generate_handler!` 中注册 `exec_shell_command`

## 2. PreviewEditor 前端

- [x] 2.1 新增 `outputVisible` 和 `outputText` state 变量
- [x] 2.2 在 `processOverlayCommand()` 中检测 `:!` 前缀，调用新命令
- [x] 2.3 新增 `outputPanelActive` state 变量
- [x] 2.4 在 `handleOverlayKeydown()` 中添加 output 模式处理（Enter/Esc 关闭面板）
- [x] 2.5 添加底部输出面板 UI 标记（可滚动 pre 块 + 状态栏）
- [x] 2.6 编辑区在面板可见时自动缩小

## 3. FullscreenEditor 前端

- [x] 3.1 同步所有 PreviewEditor 的改动到 FullscreenEditor
- [x] 3.2 新增 output state 和面板 UI
- [x] 3.3 在 `processOverlayCommand()` 和 `handleOverlayKeydown()` 中添加对应逻辑

## 4. 兜底处理

- [x] 4.1 在 `vim-commands.ts` 的 `processCommand()` 中添加 `!` 前缀处理（CodeMirror 层兜底）

## 5. 验证

- [x] 5.1 手动测试 `:!ls` 在 PreviewEditor 和 FullscreenEditor 中的行为
- [x] 5.2 手动测试 `:!` 空命令的错误提示
- [x] 5.3 手动测试输出文本的选中和 Ctrl+C 复制
- [x] 5.4 手动测试 Enter 和 Esc 关闭面板
- [x] 5.5 验证非 `!` 命令（`:w`、`:q`、`:s/`）不受影响
