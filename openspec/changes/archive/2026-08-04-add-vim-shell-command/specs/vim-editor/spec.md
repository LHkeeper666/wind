## ADDED Requirements

### Requirement: :! 命令在 overlay 命令行中被识别和处理

系统 SHALL 在 overlay 的 `processOverlayCommand()` 中识别 `:!` 前缀命令，转发给 `exec_shell_command` 后端处理，而非将其视为未知命令传递给 `Vim.handleEx()`。

#### Scenario: :! 被 overlay 拦截

- **WHEN** 用户在 normal 模式输入 `:!echo hello` 并按 Enter
- **THEN** overlay 检测到 `!` 前缀，提取 `echo hello` 作为 shell 命令
- **THEN** 命令被 invoke 到后端执行，而非传递给 CodeMirror vim 插件

#### Scenario: 非 ! 命令不受影响

- **WHEN** 用户输入 `:w`、`:q`、`:s/old/new/g` 等常规 ex 命令
- **THEN** 命令处理行为与此前完全一致，不受 `!` 逻辑影响
