# Vim Editor

CodeMirror 6 集成 vim 编辑模式的规格，包括 vim 键映射、ex 命令处理、替换预览高亮等。

## Requirements

### Requirement: :s 替换预览正确高亮所有匹配行

在 normal 模式下输入 `:%s/pattern/` 或 `:s/pattern/` 时，系统 SHALL 对文档中每一行独立进行正则匹配，高亮所有匹配行（不受前一行匹配状态影响）。

#### Scenario: :%s 全局预览高亮

- **WHEN** 用户在 normal 模式输入 `:%s/md/` 且文档多行包含 "md"
- **THEN** 所有包含 "md" 的行均显示高亮，不受行序号影响

### Requirement: 符号键正确映射到 Vim 动作

在 normal 模式下，Shift+数字组合键 SHALL 正确映射为对应的符号字符，使 `$`（行尾）、`%`（括号跳转）等 vim 动作正常工作。

#### Scenario: $ 移动到行尾

- **WHEN** 用户在 normal 模式下按下 `Shift+4`（即 `$`）
- **THEN** 光标移动到当前行末尾

### Requirement: Visual 选区后 : 命令自动添加范围前缀

用户在 visual 模式选中文本后按 `:` 时，命令行 SHALL 自动预填 `'<,'>` 范围前缀，确保后续 ex 命令作用于选区内。

#### Scenario: 选中后替换仅作用于选区

- **WHEN** 用户使用 `v` 选中若干行后按 `:` 并输入 `s/old/new/g`
- **THEN** 替换操作仅作用于选中的行范围

### Requirement: :! 命令在 overlay 命令行中被识别和处理

系统 SHALL 在 overlay 的 `processOverlayCommand()` 中识别 `:!` 前缀命令，转发给 `exec_shell_command` 后端处理，而非将其视为未知命令传递给 `Vim.handleEx()`。

#### Scenario: :! 被 overlay 拦截

- **WHEN** 用户在 normal 模式输入 `:!echo hello` 并按 Enter
- **THEN** overlay 检测到 `!` 前缀，提取 `echo hello` 作为 shell 命令
- **THEN** 命令被 invoke 到后端执行，而非传递给 CodeMirror vim 插件

#### Scenario: 非 ! 命令不受影响

- **WHEN** 用户输入 `:w`、`:q`、`:s/old/new/g` 等常规 ex 命令
- **THEN** 命令处理行为与此前完全一致，不受 `!` 逻辑影响
