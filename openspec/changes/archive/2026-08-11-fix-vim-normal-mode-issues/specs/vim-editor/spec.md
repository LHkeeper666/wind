## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: FullscreenEditor 支持未知 ex 命令的 Vim.handleEx fallback

FullscreenEditor 的 `processOverlayCommand()` SHALL 对不认识的 ex 命令调用 `Vim.handleEx()` 作为 fallback，与 PreviewEditor 行为保持一致。

#### Scenario: 未知命令传递给 vim 引擎

- **WHEN** 用户在 FullscreenEditor normal 模式下输入一个不在白名单中的 ex 命令（如 `:noh`）
- **THEN** 命令被传递给 `Vim.handleEx(cm, trimmed)` 处理
- **AND** 不再被静默丢弃
