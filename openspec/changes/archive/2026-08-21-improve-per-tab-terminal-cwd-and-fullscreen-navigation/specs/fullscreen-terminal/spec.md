## MODIFIED Requirements

### Requirement: Ctrl+Shift+` toggles fullscreen terminal
系统 SHALL 在用户按下 Ctrl+Shift+` 时切换 terminal 全屏模式；全屏 terminal SHALL 仅覆盖标题栏和 TabBar 下方、状态栏上方的应用内容区。

#### Scenario: Enter fullscreen mode
- **WHEN** 用户按下 Ctrl+Shift+` 且 terminal 未处于全屏
- **THEN** 当前 tab 的 terminal 进入全屏模式并覆盖整个内容区
- **AND** 标题栏、TabBar 和状态栏保持可见

#### Scenario: Exit fullscreen mode
- **WHEN** 用户按下 Ctrl+Shift+` 且 terminal 处于全屏
- **THEN** terminal 退出全屏，恢复之前的布局和高度
- **AND** 标题栏、TabBar 和状态栏继续可见

#### Scenario: Open terminal in fullscreen when hidden
- **WHEN** terminal 隐藏且用户按下 Ctrl+Shift+`
- **THEN** terminal 显示并直接进入内容区全屏模式

### Requirement: Fullscreen terminal 不遮挡应用 chrome
系统 SHALL 在 fullscreen terminal 打开时保留 WindowTitlebar、TabBar 和 status bar 的布局及交互层级。

#### Scenario: 全屏时标题栏和状态栏可见
- **WHEN** terminal 处于全屏模式
- **THEN** 用户仍可看到窗口标题栏、TabBar 和底部状态栏
- **AND** terminal 不覆盖这些区域

#### Scenario: 全屏时点击 TabBar 切换 tab
- **WHEN** terminal 处于全屏模式
- **AND** 用户点击 TabBar 中的另一个 tab
- **THEN** 系统切换到被点击的 tab
- **AND** 目标 tab 的 terminal 内容和 cwd 状态被显示
