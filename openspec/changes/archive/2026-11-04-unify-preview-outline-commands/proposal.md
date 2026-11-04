## Why

命令面板中的 `:toc` 目前只切换 PDF 目录，Markdown 预览虽然有标题大纲，却无法通过同一命令控制。PDF 页码跳转已有 `:数字` 快捷形式，但缺少可发现的命令名和明确的无效输入反馈。

## What Changes

- 让 `:toc` 根据当前主预览文件切换 PDF 或 Markdown 大纲。
- 增加 `:toc open` 和 `:toc close`，分别明确显示和隐藏当前预览的大纲。
- 增加 `:page N` 跳转当前 PDF 的第 N 页，保留现有的 `:N` 快捷形式。
- 对不适用的文件、缺失的大纲和无效页码给出反馈，并在命令帮助中说明用法。

## Capabilities

### New Capabilities

- `preview-navigation-commands`: 定义主预览中的大纲控制和 PDF 页码跳转命令及边界行为。

### Modified Capabilities

无。

## Impact

主要涉及 `CommandPalette.svelte`、`PreviewEditor.svelte`、PDF 与 Markdown 大纲状态接口及命令帮助。仅影响前端主预览，不新增后端命令或依赖。
