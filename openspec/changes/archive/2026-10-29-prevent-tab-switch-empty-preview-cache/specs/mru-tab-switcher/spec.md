## ADDED Requirements

### Requirement: 预览后提交保持目标内容

系统 SHALL 在切换器已显示目标 tab 的同一预览槽与文件后，提交时保留已恢复的内容、预览 DOM 和编辑器会话，同时更新 `activeTabId`、MRU 元数据并完成正式激活所需的状态恢复。提交 SHALL NOT 将该目标的内容清空或触发无必要的重新读取。

#### Scenario: 预览 Markdown 后提交

- **WHEN** 用户在切换器中预览 tab B 的非空 Markdown 文件，再确认切换到 B
- **THEN** B 的文件内容在确认前后保持可见
- **AND** `activeTabId` 更新为 B，MRU 顺序按既有规则更新
- **AND** B 的缓存不被空内容覆盖

#### Scenario: 快速连续预览后提交

- **WHEN** 用户从 tab A 依次预览 B、C、B，再确认 B
- **THEN** B 的文件内容和编辑状态与预览结束时一致
- **AND** 过期的异步恢复不得覆盖 B 的内容

#### Scenario: 同路径不同 tab

- **WHEN** tab A 和 B 打开相同文件路径，用户预览并确认 B
- **THEN** 系统仍以 B 的 tab 和预览槽身份恢复 B 独立的编辑状态
- **AND** 不把 A 的内容状态当成 B 已完成的恢复
