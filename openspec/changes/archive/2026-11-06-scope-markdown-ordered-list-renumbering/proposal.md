## Why

Markdown 编辑器把文档中多个独立的顶层有序列表当成同一个编号容器。在后面的列表按 Enter、Tab 或 Shift+Tab 时，标题或正文之前的列表也可能参与重新编号，导致后一个列表无法从自己的起点编号，并改写无关内容。

## What Changes

- 根据 Markdown 列表的实际边界建立独立的列表分区；标题、正文和列表类型切换能结束当前分区，属于列表内部的空行与续行保留在原分区。
- 列表树的父子关系和有序列表重新编号共用分区信息，分别处理顶层与嵌套容器。
- Enter、Tab、Shift+Tab 仅重编号编辑影响的容器，避免改写其他章节的有序列表。
- 补充跨章节、嵌套、空行、混合列表及代码块的回归验证。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vim-editor`: Markdown 列表树和有序列表编号遵守独立列表边界，编辑一个列表不会改写其他列表。

## Impact

- 主要修改 `src/lib/utils/editor-text-keys.ts` 中的列表识别、树构建和重编号逻辑，以及相关编辑器测试。
- 与尚未实施的 `renumber-ordered-lists-after-normal-indent` 共享编号规则；该变更接入 Vim 普通模式缩进时应复用本次确定的分区逻辑。
- 不涉及后端、数据库或文件格式变更。
