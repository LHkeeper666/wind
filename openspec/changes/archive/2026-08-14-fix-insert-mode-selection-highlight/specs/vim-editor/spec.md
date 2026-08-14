## ADDED Requirements

### Requirement: Insert 模式选区高亮可见

在 insert 模式下，当编辑器存在非空选区（例如鼠标拖动选中文本）时，系统 SHALL 取消活动行高亮（`.cm-activeLine` 与 `.cm-activeLineGutter` 背景设为透明），使选中高亮 `.cm-selectionBackground` 正常显示，避免被活动行背景覆盖。

#### Scenario: insert 模式鼠标选中单行

- **WHEN** 用户在 insert 模式下用鼠标在活动行上拖动选中一段文本
- **THEN** 选中区域显示高亮（`.cm-selectionBackground` 可见）
- **AND** 活动行背景不再覆盖选中高亮

#### Scenario: insert 模式鼠标选中多行

- **WHEN** 用户在 insert 模式下用鼠标跨多行拖动选中文本
- **THEN** 所有选中行均显示高亮
- **AND** 其中属于活动行的部分同样可见，不被活动行背景覆盖

#### Scenario: insert 模式选区被取消后恢复活动行高亮

- **WHEN** 用户在 insert 模式下选中文本后又取消选区（如点击或按方向键使选区变为空）
- **THEN** 活动行高亮恢复正常显示

#### Scenario: visual 模式行为不受影响

- **WHEN** 用户在 visual 模式下选中文本
- **THEN** 选区高亮与活动行取消行为与此前一致，不受本改动影响
