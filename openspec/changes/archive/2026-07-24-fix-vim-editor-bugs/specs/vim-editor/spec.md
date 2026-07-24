## ADDED Requirements

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
