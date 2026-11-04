## ADDED Requirements

### Requirement: 工作区焦点恢复遵循活动面板

系统 SHALL 以目标工作区保存的活动面板决定切换后 DOM 焦点所在面板。只有属于该活动面板的面板内焦点目标才能用于恢复；不匹配或缺失时 MUST 使用该面板的有效默认目标。恢复期间产生的程序性焦点事件 MUST NOT 覆盖已保存的目标状态。

#### Scenario: 从编辑器切到目录面板工作区
- **WHEN** 用户从处于编辑模式的工作区切换到保存为 `activeColumn=current` 的工作区，即使目标的旧 `focusTarget` 为 `editor`
- **THEN** DOM 焦点落在目标工作区的 current 目录面板
- **AND** `layout.activeColumn` 保持为 `current`

#### Scenario: 返回编辑器工作区
- **WHEN** 用户切回保存为 `activeColumn=preview` 且编辑器处于 normal 或 insert 模式的工作区
- **THEN** DOM 焦点分别落在该工作区的 Vim overlay 或 CodeMirror 输入区
- **AND** 原有编辑内容、光标和选区保持不变

#### Scenario: 快速连续切换
- **WHEN** 用户连续切换多个工作区，旧工作区仍有等待执行的聚焦回调
- **THEN** 只有最终正式激活工作区的有效回调能改变 DOM 焦点
- **AND** 旧回调不能改写任何工作区的焦点记忆

#### Scenario: 恢复完成后用户主动改变焦点
- **WHEN** 正式切换的焦点恢复已完成，用户点击另一个可见面板
- **THEN** 活动面板和该工作区的焦点记忆更新为用户点击的面板
