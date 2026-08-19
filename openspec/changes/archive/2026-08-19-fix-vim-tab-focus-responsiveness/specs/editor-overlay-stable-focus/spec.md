## ADDED Requirements

### Requirement: 仅活动 Vim 会话可接收编辑器焦点

当多个 Tab 保留 Vim 编辑器会话时，系统 SHALL 只允许活动 Tab 的会话接收焦点和编辑器级键盘事件。后台会话 MUST 隐藏且不得通过 overlay、CodeMirror 内容区或会话观察器抢占焦点。

#### Scenario: 切换到另一个 Vim normal 模式 Tab
- **WHEN** Tab A 和 Tab B 都保留 Vim normal 模式会话
- **AND** 用户从 Tab A 切换到 Tab B
- **THEN** Tab B 的 overlay 成为唯一可聚焦的 Vim normal 输入目标
- **AND** Tab A 的 overlay 不再接收键盘输入或抢回焦点

#### Scenario: 后台会话触发延迟测量
- **WHEN** 非活动 Tab 的隐藏 CodeMirror 会话产生延迟测量或观察器回调
- **THEN** 该回调不得改变 `document.activeElement`
- **AND** 不得改变活动 Tab 的 Vim mode 或光标状态
