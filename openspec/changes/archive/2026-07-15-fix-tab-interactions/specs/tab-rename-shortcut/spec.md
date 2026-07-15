## ADDED Requirements

### Requirement: 键盘快捷键 t r 触发 tab 重命名
系统 SHALL 支持通过键盘快捷键 `t` 后按 `r` 来触发当前激活 tab 的内联重命名，效果与双击 tab 名称一致。

#### Scenario: t r 进入重命名模式
- **WHEN** 用户在 normal 模式下依次按下 `t` 和 `r`
- **AND** 当前有激活的 tab
- **THEN** 当前激活 tab 的名称变为可编辑的输入框
- **AND** 输入框自动获得焦点
- **AND** 输入框预填当前 tab 名称

#### Scenario: Enter 确认重命名
- **WHEN** 用户在重命名输入框中修改名称后按 Enter
- **THEN** tab 名称更新为新值
- **AND** 输入框消失，tab 显示新名称

#### Scenario: Escape 取消重命名
- **WHEN** 用户在重命名输入框中按 Escape
- **THEN** 重命名取消，tab 名称保持不变
- **AND** 输入框消失

#### Scenario: 空名称默认值
- **WHEN** 用户在重命名输入框中清空所有文字后按 Enter
- **THEN** tab 名称设为 'untitled'
