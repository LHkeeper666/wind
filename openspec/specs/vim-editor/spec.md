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

### Requirement: :reg 命令查看寄存器内容

系统 SHALL 支持 `:reg`、`:registers`、`:di`、`:display` 命令，在 normal 模式下输入后按 Enter 显示当前所有 vim 寄存器的内容。

#### Scenario: 查看所有寄存器

- **WHEN** 用户在 normal 模式输入 `:reg` 并按 Enter
- **THEN** 系统显示所有非空寄存器的名称和内容（每行格式: `"x   content`）
- **AND** 默认/未命名寄存器显示为 `""`

#### Scenario: yy 后查看寄存器

- **WHEN** 用户执行 `yy` 复制一行后输入 `:reg` 并按 Enter
- **THEN** 未命名寄存器 `""` 显示刚复制的行内容

### Requirement: yank/delete 后粘贴使用本次内容

系统 SHALL 确保 dd、yy、cc 等 vim 操作更新未命名寄存器后，后续的 p/P 操作粘贴出本次操作的内容，而非 clipboardCache 中的过期内容。

#### Scenario: dd 后粘贴

- **WHEN** 用户执行 `dd` 删除一行
- **AND** 立即按下 `p` 粘贴
- **THEN** 粘贴出刚被 dd 删除的那行内容

#### Scenario: yy 后粘贴

- **WHEN** 用户执行 `yy` 复制一行
- **AND** 移动光标到其他行
- **AND** 按下 `p` 粘贴
- **THEN** 粘贴出 yy 复制的行内容

#### Scenario: 外部复制后粘贴

- **WHEN** 用户在编辑器外部（系统其他应用）复制文本
- **AND** overlay 重新获得焦点（触发 clipboardCache 更新）
- **AND** 按下 `p` 粘贴
- **THEN** 粘贴出系统剪贴板中最新的内容

### Requirement: FullscreenEditor 支持未知 ex 命令的 Vim.handleEx fallback

FullscreenEditor 的 `processOverlayCommand()` SHALL 对不认识的 ex 命令调用 `Vim.handleEx()` 作为 fallback，与 PreviewEditor 行为保持一致。

#### Scenario: 未知命令传递给 vim 引擎

- **WHEN** 用户在 FullscreenEditor normal 模式下输入一个不在白名单中的 ex 命令（如 `:noh`）
- **THEN** 命令被传递给 `Vim.handleEx(cm, trimmed)` 处理
- **AND** 不再被静默丢弃

### Requirement: ex 命令通过统一入口注册

系统 SHALL 在 `initEditor()` 中通过 `setupAllVimCommands()` 统一注册所有 vim ex 命令，而非在多个位置散落注册。

#### Scenario: 命令注册聚合

- **WHEN** 编辑器初始化
- **THEN** `:set`、`:reg`、`:w`/`:q` 等所有命令通过同一个 setup 流程注册
- **AND** 未来新增命令只需在 `setupAllVimCommands()` 中添加一行注册调用

### Requirement: basicSetup 替换为独立 extensions

编辑器初始化 SHALL 使用从 `@codemirror/view`、`@codemirror/commands`、`@codemirror/language` 等子包独立导入的 extensions 替代 `basicSetup`，其中 `lineNumbers()` 由 compartment 控制。

#### Scenario: Editor 功能保持一致

- **WHEN** 编辑器初始化
- **THEN** 所有 `basicSetup` 提供的功能（语法高亮、历史、折叠、自动补全等）仍然正常工作
- **AND** 只有 `lineNumbers()` 被 compartment 替代管理
