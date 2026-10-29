## ADDED Requirements

### Requirement: 预览后提交保持目标内容

系统 SHALL 在切换器已显示目标 tab 的同一预览槽与文件后，提交时保留已恢复的内容、预览 DOM 和编辑器会话，同时更新 `activeTabId`、MRU 元数据并完成正式激活所需的状态恢复。提交 SHALL NOT 清空该目标内容或触发无必要的重新读取。

#### Scenario: 预览非空 Markdown 后提交

- **WHEN** 用户从 tab A 预览 tab B 的非空 Markdown 文件并确认 B
- **THEN** B 的内容在确认前后保持可见
- **AND** `activeTabId` 与 MRU 按既有规则更新
- **AND** B 的有效缓存不被空内容覆盖

#### Scenario: 快速连续预览

- **WHEN** 用户依次预览 B、C、B 并确认 B
- **THEN** B 的内容及编辑状态与确认前一致
- **AND** 过期的异步恢复不得覆盖 B

#### Scenario: 返回原 tab

- **WHEN** 用户预览其他 tab 后返回原 tab 并退出切换器
- **THEN** 原 tab 内容保持可见
- **AND** 原有不更新 MRU 的语义保持不变

#### Scenario: 同路径不同 tab

- **WHEN** tab A 与 B 打开同一路径且用户预览并确认 B
- **THEN** 系统按 B 的 tab 和槽身份恢复独立状态
- **AND** 不把 A 的显示状态当作 B 的有效状态
