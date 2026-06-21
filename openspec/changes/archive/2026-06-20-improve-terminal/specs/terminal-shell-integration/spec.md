## ADDED Requirements

### Requirement: 全局终端切换快捷键
系统 SHALL 支持 Ctrl+` 快捷键在任何焦点状态下切换浮动终端的显示/隐藏。

#### Scenario: 焦点在目录面板时切换终端
- **WHEN** 用户焦点在目录面板，按下 Ctrl+`
- **THEN** 终端显示/隐藏状态切换

#### Scenario: 焦点在终端内部时切换终端
- **WHEN** 用户焦点在 xterm.js 终端内部，按下 Ctrl+`
- **THEN** 终端显示/隐藏状态切换，不向 shell 发送按键

#### Scenario: 焦点在命令面板时切换终端
- **WHEN** 命令面板打开，用户按下 Ctrl+`
- **THEN** 终端显示/隐藏状态切换

### Requirement: 终端内部快捷键不冲突
系统 SHALL 确保 Escape 键在终端 Insert 模式下切换到 Normal 模式，而非关闭终端。

#### Scenario: 终端 Insert 模式按 Escape
- **WHEN** 终端处于 Insert 模式，用户按下 Escape
- **THEN** 终端切换到 Normal 模式，不关闭终端

#### Scenario: 终端 Normal 模式按 i
- **WHEN** 终端处于 Normal 模式，用户按下 i
- **THEN** 终端切换到 Insert 模式

### Requirement: Shell 集成 - 命令追踪
系统 SHALL 解析 OSC 133 协议序列，追踪 shell 命令的执行状态。

#### Scenario: 检测命令开始执行
- **WHEN** 终端接收到包含 `OSC 133 ; C` 的输出
- **THEN** 系统标记命令开始执行状态

#### Scenario: 检测命令执行完成
- **WHEN** 终端接收到包含 `OSC 133 ; D ; <exit_code>` 的输出
- **THEN** 系统记录命令完成状态和退出码

### Requirement: Shell 集成 - 当前目录同步
系统 SHALL 通过 OSC 7 协议同步 shell 的当前工作目录。

#### Scenario: 目录变化同步
- **WHEN** 终端接收到包含 `OSC 7 ; file://<path>` 的输出
- **THEN** 系统更新内部记录的当前目录路径

### Requirement: Shell 集成状态可视化
系统 SHALL 在终端 UI 中显示 shell 集成状态信息。

#### Scenario: 显示命令退出状态
- **WHEN** 命令执行完成且退出码非 0
- **THEN** 终端状态栏显示错误指示

#### Scenario: 显示当前目录
- **WHEN** shell 集成检测到目录变化
- **THEN** 终端状态栏更新显示当前目录
