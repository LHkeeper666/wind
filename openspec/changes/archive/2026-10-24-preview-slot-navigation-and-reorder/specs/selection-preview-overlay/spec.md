## ADDED Requirements

### Requirement: 从目录切换或移动预览位时保留选中预览层
系统 SHALL 在 current 面板通过键盘切换或移动预览位时，保持选中预览层的可见性、目标条目和当前显示内容，不因活动位变化而聚焦预览面板。

#### Scenario: 选中预览层可见时切槽
- **WHEN** current 面板持有焦点且选中预览层可见，用户按 `Ctrl+H` 或 `Ctrl+L`
- **THEN** 活动预览位标签背景色随之更新
- **AND** 选中预览层继续显示原条目的当前预览内容
- **AND** current 面板继续持有键盘焦点

#### Scenario: 选中预览层可见时移动槽
- **WHEN** current 面板持有焦点且选中预览层可见，用户按 `Ctrl+,` 或 `Ctrl+.`
- **THEN** 选中预览层及其当前显示内容保持不变
- **AND** current 面板继续持有键盘焦点
