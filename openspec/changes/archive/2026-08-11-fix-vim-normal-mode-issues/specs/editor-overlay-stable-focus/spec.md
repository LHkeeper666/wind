## ADDED Requirements

### Requirement: 鼠标选中在编辑器外释放后 overlay 保持焦点

在 normal 模式下，系统 SHALL 注册文档级 mouseup 监听器，当用户在编辑器面板外释放鼠标时，自动将焦点恢复到 overlay div，防止 CodeMirror 获得焦点导致键盘输入绕过 overlay handler。

#### Scenario: 鼠标拖拽选中后在面板外释放

- **WHEN** 用户在 editor-normal 模式下用鼠标拖拽选择文本
- **AND** 鼠标在编辑器面板区域外释放（mouseup 事件不命中 .editor-content）
- **THEN** 下一帧 overlay 重新获得焦点
- **AND** 键盘输入继续由 overlay 的 keydown handler 处理

#### Scenario: 鼠标拖拽选中后在面板内释放

- **WHEN** 用户在 editor-normal 模式下用鼠标拖拽选择文本
- **AND** 鼠标在编辑器面板区域内释放
- **THEN** 现有的 `onmouseup` handler 正常工作（重新聚焦 overlay）

#### Scenario: 切换到其他面板后不抢回焦点

- **WHEN** 用户通过 Ctrl+W h/l 切换到其他面板
- **AND** `activeColumn` 不再是 'preview'
- **THEN** mouseup listener 不重新聚焦 overlay

#### Scenario: overlay 隐藏时清理 listener

- **WHEN** 编辑器从 normal 模式切换到 insert 模式（overlay 隐藏）
- **THEN** 文档级 mouseup listener 被移除
