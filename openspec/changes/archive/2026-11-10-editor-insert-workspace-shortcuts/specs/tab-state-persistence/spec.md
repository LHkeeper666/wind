## ADDED Requirements

### Requirement: Insert 模式工作区快捷键

预览面板文本编辑器处于 insert 模式时，系统 SHALL 响应 `Alt+H/L/M` 和 `Alt+1-9` 切换 tab，响应 `Alt+,/.` 与相邻 tab 交换显示顺序。切回 tab SHALL 恢复其活动预览槽的草稿、真实 Vim insert 状态、光标选区、滚动位置和输入焦点；排序 SHALL 不切换活动 tab 或重建编辑器。

#### Scenario: Insert 模式切换 tab 并返回

- **WHEN** 用户在 tab A 的 insert 模式中编辑未保存文本，使用指定 Alt 切换键到 tab B，再返回 A
- **THEN** A 的草稿、活动槽、Vim insert 状态、光标选区和滚动位置恢复
- **AND** 后续字符立即写入 A 的原光标位置
- **AND** B 的编辑状态不被 A 覆盖

#### Scenario: Alt 切换器候选预览

- **WHEN** 用户从 insert 模式按住 Alt 并用 `H/L/M` 预览多个 tab，最后提交目标或返回原 tab
- **THEN** 候选预览不改写任何 tab 已保存的编辑焦点和 Vim 模式
- **AND** 最终 tab 的内容、活动槽与输入状态正确恢复

#### Scenario: Insert 模式交换 tab 顺序

- **WHEN** 用户在 insert 模式按 `Alt+,` 或 `Alt+.` 且该方向存在相邻 tab
- **THEN** 当前 tab 与相邻 tab 交换显示顺序
- **AND** 活动 tab ID、EditorView、草稿、Vim insert 状态、光标和焦点不变
- **AND** 正文不插入逗号或句点

#### Scenario: Tab 顺序边界

- **WHEN** 用户在最左 tab 按 `Alt+,` 或最右 tab 按 `Alt+.`
- **THEN** 顺序和编辑状态不变
