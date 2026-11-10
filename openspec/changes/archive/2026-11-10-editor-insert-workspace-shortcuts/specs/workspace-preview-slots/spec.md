## ADDED Requirements

### Requirement: Insert 模式预览槽快捷键

预览面板文本编辑器处于 insert 模式时，系统 SHALL 响应 `Ctrl+H/L` 切换相邻预览槽，响应 `Ctrl+,/.` 交换当前预览槽与相邻槽的显示顺序。切槽 SHALL 按槽身份恢复草稿、真实 Vim 模式、光标选区、滚动位置和输入焦点；排序 SHALL 保持当前槽身份、编辑器状态与焦点。

#### Scenario: Insert 模式切槽再返回

- **WHEN** 用户在槽 A 的 insert 模式中编辑未保存文本，并用 `Ctrl+L` 切到槽 B，再用 `Ctrl+H` 返回
- **THEN** 槽 A 的草稿、insert 模式、光标选区和滚动位置恢复
- **AND** CodeMirror 内容区获得焦点，后续字符写入槽 A 的光标处
- **AND** 槽 B 的状态保持独立

#### Scenario: Insert 模式交换槽顺序

- **WHEN** 用户在槽 A 的 insert 模式中按 `Ctrl+,` 或 `Ctrl+.` 且该方向存在相邻槽
- **THEN** 槽 A 与相邻槽交换显示顺序
- **AND** 活动槽 ID、EditorView、草稿、Vim insert 状态、光标和焦点不变
- **AND** 正文不插入逗号或句点

#### Scenario: 槽顺序边界

- **WHEN** 用户在最左槽按 `Ctrl+,`、在最右槽按 `Ctrl+.`，或仅有一个槽
- **THEN** 顺序和编辑状态不变

#### Scenario: 快速切槽

- **WHEN** 用户在多个槽之间快速往返且先前的加载或焦点恢复尚未完成
- **THEN** 仅当前槽的恢复结果可以改变可见内容、光标和焦点
