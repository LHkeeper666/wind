# Vim Shell Command

vim 编辑模式下通过 `:!` 执行 shell 命令并显示输出的规格。

## Requirements

### Requirement: 用户通过 :! 执行 shell 命令

系统 SHALL 允许用户在 vim normal 模式下输入 `:!<command>`，在当前编辑文件所在目录用 bash 执行该命令，并在编辑器底部面板中显示输出结果。

#### Scenario: 执行简单命令并查看输出

- **WHEN** 用户在 normal 模式输入 `:!ls` 并按下 Enter
- **THEN** 系统在编辑器底部弹出面板，显示当前文件目录的文件列表
- **THEN** 编辑区自动缩小为面板腾出空间

#### Scenario: 执行带参数的命令

- **WHEN** 用户输入 `:!git diff --stat` 并按下 Enter
- **THEN** 系统执行 `git diff --stat` 并显示完整输出
- **THEN** 参数中的空格和特殊字符被正确传递给 bash

### Requirement: 命令工作目录为当前文件所在目录

系统 SHALL 将 `:!` 命令的工作目录设置为当前编辑文件的父目录。

#### Scenario: 命令在文件目录执行

- **WHEN** 用户正在编辑 `D:\projects\myapp\main.py`
- **THEN** `:!ls` 显示 `D:\projects\myapp\` 的内容

#### Scenario: 文件路径为空时使用当前目录

- **WHEN** 用户未打开任何文件（`filePath` 为 null）
- **THEN** 命令在启动目录或最近浏览目录执行（best-effort）

### Requirement: 输出面板支持文本选中和复制

输出面板中的文本 SHALL 允许用户使用鼠标选中内容，并通过 Ctrl+C 进行原生复制。

#### Scenario: 鼠标选中并复制输出

- **WHEN** 命令输出显示在底部面板中
- **THEN** 用户可以用鼠标拖拽选中任意文本
- **THEN** 按 Ctrl+C 将选中文本复制到系统剪贴板

### Requirement: 输出面板可通过 Enter 或 Esc 关闭

系统 SHALL 允许用户通过按 Enter 或 Esc 键关闭输出面板，并恢复编辑器焦点。

#### Scenario: Enter 关闭面板

- **WHEN** 输出面板可见且用户按下 Enter
- **THEN** 输出面板关闭，编辑器恢复编辑区大小，focus 返回编辑器

#### Scenario: Esc 关闭面板

- **WHEN** 输出面板可见且用户按下 Esc
- **THEN** 输出面板关闭，行为与 Enter 一致

### Requirement: 空命令提示错误

系统 SHALL 在用户输入 `:!` 后不跟命令内容直接按 Enter 时，显示 "E471: Argument required" 提示，不执行任何操作。

#### Scenario: 空 ! 命令

- **WHEN** 用户输入 `:!` 或 `:! `（仅空格）并按 Enter
- **THEN** 不弹出面板，显示错误提示 "E471: Argument required"

### Requirement: bash 路径自动检测 fallback

系统 SHALL 按以下优先级检测可用的 bash 路径：`C:\Program Files\Git\bin\bash.exe`、`C:\msys64\usr\bin\bash.exe`、`C:\cygwin64\bin\bash.exe`、PATH 中的 `bash`。使用第一个存在的路径执行命令。

#### Scenario: Git Bash 可用

- **WHEN** 系统安装了 Git for Windows
- **THEN** 使用 `C:\Program Files\Git\bin\bash.exe -c` 执行命令

#### Scenario: bash 不可用

- **WHEN** 所有 fallback 路径均无效
- **THEN** 输出面板显示错误信息 "bash not found. Install Git for Windows or MSYS2."
- **THEN** 用户可关闭面板，不影响编辑器正常使用

### Requirement: 非零退出码在输出中体现

系统 SHALL 在命令以非零退出码结束时，在输出面板状态栏显示退出码信息。

#### Scenario: 命令执行失败

- **WHEN** 命令以非零退出码结束（如 `:!cat nonexistent_file`）
- **THEN** 面板显示命令的标准输出和标准错误
- **THEN** 状态栏显示退出码（如 "exit: 1"）
