## ADDED Requirements

### Requirement: Floating terminal overlay
系统 SHALL 提供浮动终端，默认隐藏。

#### Scenario: Hidden by default
- **WHEN** 应用启动
- **THEN** 终端默认不显示

#### Scenario: Toggle terminal
- **WHEN** 用户按 `` Ctrl+` ``（反引号）
- **THEN** 切换浮动终端的显示/隐藏

#### Scenario: Terminal appearance
- **WHEN** 浮动终端显示
- **THEN** 从窗口底部滑出，占据窗口高度的 30%

### Requirement: Terminal shell selection
浮动终端 SHALL 支持多种 shell。

#### Scenario: Default shell
- **WHEN** 浮动终端首次打开
- **THEN** 使用 PowerShell 作为默认 shell

#### Scenario: Shell switching
- **WHEN** 用户点击终端标题栏的 shell 按钮
- **THEN** 切换到对应 shell（PowerShell, CMD, Git Bash）

#### Scenario: Shell state preservation
- **WHEN** 用户切换 shell
- **THEN** 保留原 shell 的状态，可以切换回来

### Terminal mode management
浮动终端 SHALL 支持 vim 风格的模式切换。

#### Scenario: Insert mode
- **WHEN** 浮动终端打开
- **THEN** 默认进入 Insert 模式，用户可以直接输入命令

#### Scenario: Normal mode
- **WHEN** 用户按 `Escape` 键
- **THEN** 从 Insert 模式切换到 Normal 模式，终端输入被禁用

#### Scenario: Switch to Insert mode
- **WHEN** 用户在 Normal 模式下按 `i` 键
- **THEN** 切换回 Insert 模式，可以继续输入

### Requirement: Terminal focus management
浮动终端 SHALL 正确管理焦点。

#### Scenario: Auto focus
- **WHEN** 浮动终端打开
- **THEN** 自动获得焦点，用户可以直接输入

#### Scenario: Restore focus
- **WHEN** 用户关闭浮动终端
- **THEN** 焦点恢复到之前活动的面板

#### Scenario: Click to focus
- **WHEN** 用户点击浮动终端
- **THEN** 终端获得焦点

### Requirement: Terminal resize
浮动终端 SHALL 支持调整高度。

#### Scenario: Drag to resize
- **WHEN** 用户拖拽终端顶部的分隔线
- **THEN** 终端高度按鼠标移动距离调整

#### Scenario: Minimum height
- **WHEN** 用户调整终端高度
- **THEN** 终端最小高度为 100px

#### Scenario: Maximum height
- **WHEN** 用户调整终端高度
- **THEN** 终端最大高度为窗口高度的 80%

### Requirement: Terminal keyboard shortcuts
浮动终端 SHALL 支持标准终端快捷键。

#### Scenario: Clear terminal
- **WHEN** 用户在终端中输入 `clear` 或按 `Ctrl+L`
- **THEN** 清空终端输出

#### Scenario: Copy/Paste
- **WHEN** 用户按 `Ctrl+C`
- **THEN** 复制选中的文本或发送中断信号
- **WHEN** 用户按 `Ctrl+V`
- **THEN** 粘贴剪贴板内容

### Requirement: Terminal shell buttons
浮动终端 SHALL 提供 shell 切换按钮。

#### Scenario: PowerShell button
- **WHEN** 用户点击 "PowerShell" 按钮
- **THEN** 切换到 PowerShell shell

#### Scenario: CMD button
- **WHEN** 用户点击 "CMD" 按钮
- **THEN** 切换到 CMD shell

#### Scenario: Git Bash button
- **WHEN** 用户点击 "Git Bash" 按钮
- **THEN** 切换到 Git Bash shell

#### Scenario: Clear button
- **WHEN** 用户点击 "Clear" 按钮
- **THEN** 清空终端输出
