## ADDED Requirements

### Requirement: Tab 标签根据窗口宽度动态调整宽度
系统 SHALL 让 tab 标签在 tab bar 中等分可用宽度，而非使用固定宽度。

#### Scenario: 少量 tab 时等分宽度
- **WHEN** 窗口宽度为 1200px
- **AND** 有 3 个 tab
- **THEN** 每个 tab 宽度约为 400px（不超过 max-width 250px 时拉伸到 250px，剩余空间留白）

#### Scenario: 多个 tab 时收缩
- **WHEN** 窗口宽度为 1200px
- **AND** 有 10 个 tab
- **THEN** 每个 tab 等分 bar 宽度，约为 120px

#### Scenario: 大量 tab 时保持最小宽度
- **WHEN** tab 数量增多导致每个 tab 的计算宽度小于 80px
- **THEN** 每个 tab 保持 80px 最小宽度
- **AND** tab bar 出现水平滚动条

#### Scenario: 窗口缩放时动态调整
- **WHEN** 用户拖动窗口边缘改变窗口宽度
- **THEN** 所有 tab 标签实时调整宽度以等分新的可用空间
