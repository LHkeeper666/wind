## Why

CodeMirror 将载入文档中的 CRLF/CR 换行规范化为 LF，但预览编辑器仍用原始文件文本作为未保存修改的比较基准。因此用户仅按 `e` 或 `l` 进入编辑模式，预览槽就可能出现修改标记；继续保存还可能无意改写整份文件的换行格式。

## What Changes

- 载入文本时记录原始换行风格，并为编辑器内容和已保存基准使用一致的规范化形式。
- 仅在实际编辑导致内容偏离基准时显示修改标记，包括缓存恢复及预览槽切换。
- 保存时按原始换行风格序列化草稿，避免仅因进入编辑器或局部修改而重写其余行的换行符。
- 增加 LF、CRLF、CR 及相关缓存和保存流程的回归验证。

## Capabilities

### New Capabilities

- `text-line-ending-preservation`: 文本编辑器的修改状态和保存行为正确处理原文件换行格式。

### Modified Capabilities

无。

## Impact

涉及文本读取与编辑状态、`TextEditorHost`、`PreviewEditor`、预览槽缓存及文本保存路径。无需改变 Tauri 命令接口或增加依赖。
