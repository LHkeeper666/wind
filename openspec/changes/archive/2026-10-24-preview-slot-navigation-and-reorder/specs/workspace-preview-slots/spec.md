## MODIFIED Requirements

### Requirement: 预览位键盘操作
系统 SHALL 使用 `Ctrl+N` 新建预览位、`Ctrl+U` 关闭当前预览位、`Ctrl+H` 切换到上一个预览位、`Ctrl+L` 切换到下一个预览位、`Ctrl+,` 将当前预览位与左侧相邻位交换、`Ctrl+.` 将当前预览位与右侧相邻位交换。`Ctrl+W h/j/k/l/m` 的现有面板与布局操作 SHALL 保持不变。

#### Scenario: 新建与关闭
- **WHEN** 用户按 `Ctrl+N` 或 `Ctrl+U`
- **THEN** 系统分别新建预览位或关闭当前预览位，并遵守至少保留一个位及未保存修改保护规则

#### Scenario: 前后切换
- **WHEN** 用户按 `Ctrl+H` 或 `Ctrl+L`
- **THEN** 系统按标签显示顺序分别激活上一个或下一个预览位
- **AND** 切换跨越列表末端时循环到另一端
- **AND** 各槽的内容和编辑状态保持独立

#### Scenario: 从 current 面板切换
- **WHEN** current 面板持有键盘焦点且用户按 `Ctrl+H` 或 `Ctrl+L`
- **THEN** 活动预览位和标签背景色更新
- **AND** DOM 焦点与 `layout.activeColumn` 均保持在 current 面板

#### Scenario: 从 preview 面板切换
- **WHEN** preview 面板持有键盘焦点且用户按 `Ctrl+H` 或 `Ctrl+L`
- **THEN** 活动预览位更新且焦点处于激活预览位的内容

#### Scenario: 向相邻位置移动
- **WHEN** 用户按 `Ctrl+,` 或 `Ctrl+.` 且对应方向存在相邻预览位
- **THEN** 系统分别向左或向右交换活动预览位与相邻位的显示顺序
- **AND** 活动预览位的 ID、内容和编辑状态不变
- **AND** 现有面板焦点不变

#### Scenario: 位于边界或仅有一位
- **WHEN** 用户在左端按 `Ctrl+,`、在右端按 `Ctrl+.`，或工作区仅有一个预览位
- **THEN** 预览位顺序不变
