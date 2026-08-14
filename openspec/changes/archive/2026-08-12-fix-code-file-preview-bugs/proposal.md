## Why

预览面板切换 .cpp/.py 等代码文件时出现两个 bug：(1) 第二个文件切换后编辑器空白；(2) `:q` 退出编辑后偶尔出现不应该有的 Shiki 预览样式。根因是 `editorView`/`editorFilePath` 非响应式变量 + 无条件销毁 editorView 导致的竞态条件，以及 `handlePanelFocus` 错误修改 `codeFileDirectEdit`。

## What Changes

- 将 `editorView` 和 `editorFilePath` 改为 `$state` 响应式变量，确保 mode `$effect` 能在 editorView 被销毁后正确触发 `initEditor()`
- `loadFile()` 中对代码文件保留 editorView 不销毁，改为通过 `dispatch` 原地替换内容，消除不必要的全量重建
- 修复 `handlePanelFocus` 中错误设置 `codeFileDirectEdit = false` 的逻辑，改为仅触发模式切换

## Capabilities

### New Capabilities
- `code-file-editor-stability`: 代码文件（非 markdown/json/ipynb）在预览面板中始终以 CodeMirror editor 模式显示，切换文件时内容正确更新，`:q` 后回到 plain text 预览而非 Shiki 预览样式

### Modified Capabilities
<!-- 无现有 spec 要求变更，此 fix 修复实现层面的 bug -->

## Impact

- `src/lib/components/PreviewEditor.svelte` — 主要修改文件
  - `editorView`/`editorFilePath` 变量声明改为 `$state`
  - `loadFile()` 中区分对待代码文件和预览文件
  - `handlePanelFocus()` 修复 codeFileDirectEdit 逻辑
