## REMOVED Requirements

### Requirement: Ctrl+L 手动恢复焦点
**Reason**: `Ctrl+L` 改用于切换到下一个预览位。
**Migration**: 使用 `Ctrl+'` 手动恢复焦点。

## ADDED Requirements

### Requirement: Ctrl+' 手动恢复焦点
用户按 `Ctrl+'` 时，系统 SHALL 将焦点恢复到 `layout.activeColumn` 对应的面板，并显示 toast 提示。

#### Scenario: 正常面板状态下恢复焦点
- **WHEN** 用户在任意面板（非终端 insert 模式）按下 `Ctrl+'`
- **AND** 命令面板和文件搜索均未打开
- **AND** 无全屏 overlay 打开
- **THEN** 系统调用 `focusPanel(activeColumn)`
- **AND** 显示 toast `Focus: {activeColumn 大写}`

#### Scenario: 终端 insert 模式
- **WHEN** 用户在终端 insert 模式下按下 `Ctrl+'`
- **THEN** 系统不拦截该按键，透传给 shell

#### Scenario: 命令面板、文件搜索或全屏模式
- **WHEN** 命令面板、文件搜索或全屏 viewer 打开时用户按下 `Ctrl+'`
- **THEN** 系统不触发手动焦点恢复
