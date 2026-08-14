# command-palette Specification

## Purpose
命令面板（`:` 触发）支持 `:cd`、`:e` 命令及 Tab 路径自动补全，用于快速切换目录和打开文件。

## Requirements

### Requirement: :cd 命令切换当前 tab 目录

系统 SHALL 支持 `:cd` 命令切换当前 tab 的目录，支持绝对路径、相对路径、`..` 与无参数（回 home），并归一化 `/` 与 `\`。

#### Scenario: 绝对路径切换
- **WHEN** 用户输入 `:cd D:\code\proj`
- **THEN** 当前 tab 目录切换到 `D:\code\proj`

#### Scenario: 相对路径切换
- **WHEN** 用户输入 `:cd subdir`
- **THEN** 当前 tab 目录切换到 `当前路径\subdir`

#### Scenario: 无参数回 home
- **WHEN** 用户输入 `:cd` 且无参数
- **THEN** 当前 tab 目录切换到用户 home 目录

#### Scenario: 路径不存在提示错误
- **WHEN** 用户输入 `:cd 不存在的路径`
- **THEN** toast 提示 `E344: Can't find directory: <path>`

### Requirement: :e 命令打开文件或导航目录

系统 SHALL 支持 `:e` 命令打开文件（在 preview 面板选中）或导航到目录，支持 `:e .` 与无参数刷新当前目录。

#### Scenario: 打开文件
- **WHEN** 用户输入 `:e path/to/file.txt`
- **THEN** preview 面板选中该文件

#### Scenario: 导航目录
- **WHEN** 用户输入 `:e path/to/dir`
- **THEN** 导航到该目录

#### Scenario: 无参数刷新
- **WHEN** 用户输入 `:e` 且无参数
- **THEN** 刷新当前目录

#### Scenario: 路径不存在提示错误
- **WHEN** 用户输入 `:e 不存在的路径`
- **THEN** toast 提示 `E344: Can't find directory: <path>`

### Requirement: Tab 路径自动补全

系统 SHALL 在命令输入中按 Tab 触发路径补全，`cd` 只补全目录，`e` 补全文件与目录。

#### Scenario: 唯一匹配立即补全
- **WHEN** 用户输入 `:cd Doc` 后按 Tab，且只有一个以 `Doc` 开头的目录
- **THEN** 立即补全完整目录名（目录补全时带尾部 `\`）

#### Scenario: 多个匹配循环
- **WHEN** 用户输入的前缀有多个匹配项
- **THEN** 每次按 Tab 在匹配项间循环

#### Scenario: 无匹配无操作
- **WHEN** 用户输入的前缀无匹配项
- **THEN** 补全不做任何更改

#### Scenario: 非 Tab 键重置补全状态
- **WHEN** 用户在补全后按下任意非 Tab 键
- **THEN** 补全状态被重置
