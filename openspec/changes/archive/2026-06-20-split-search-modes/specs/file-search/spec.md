## MODIFIED Requirements

### Requirement: 递归搜索

系统 SHALL 支持两种搜索模式：当前目录搜索和递归深度搜索，通过不同快捷键触发。

#### Scenario: 通过 / 触发当前目录搜索
- **WHEN** 用户在目录面板按 `/` 键
- **THEN** 系统打开搜索浮窗，搜索范围限定为当前目录（depth 1），标签显示 "Current Directory"

#### Scenario: 通过 g/ 触发递归深度搜索
- **WHEN** 用户在目录面板按 `g/` 键
- **THEN** 系统打开搜索浮窗，搜索范围为递归子目录（depth 10），标签显示 "Recursive"

#### Scenario: 当前目录搜索不匹配子目录文件
- **WHEN** 用户在当前目录搜索 `package.json`
- **THEN** 系统仅显示当前目录直接子项中的 `package.json`，不显示子目录中的同名文件

#### Scenario: 递归搜索匹配深层文件
- **WHEN** 用户在递归模式搜索 `package.json`
- **THEN** 系统显示所有子目录（depth ≤ 10）中的 `package.json` 文件

#### Scenario: 显示相对路径
- **WHEN** 搜索结果显示匹配文件
- **THEN** 每个结果显示相对于根目录的路径（如 `Projects/terminal/package.json`）

### Requirement: 搜索工具优先级

系统 SHALL 优先使用 fd 进行搜索，不可用时自动回退。两种搜索模式共享相同的工具优先级。

#### Scenario: fd 可用时使用 fd
- **WHEN** 系统检测到 fd 已安装
- **THEN** 使用 fd 执行搜索，当前目录模式使用 `--max-depth 1`，递归模式使用 `--max-depth 10`

#### Scenario: fd 不可用时回退
- **WHEN** 系统检测到 fd 未安装
- **THEN** 使用 Rust 原生实现搜索，并显示安装提示

## ADDED Requirements

### Requirement: 搜索模式标签

系统 SHALL 在搜索浮窗中显示当前搜索模式标签，让用户明确搜索范围。

#### Scenario: 当前目录模式显示标签
- **WHEN** 用户通过 `/` 打开搜索
- **THEN** 搜索输入框旁显示 "Current Directory" 标签

#### Scenario: 递归模式显示标签
- **WHEN** 用户通过 `g/` 打开搜索
- **THEN** 搜索输入框旁显示 "Recursive" 标签
