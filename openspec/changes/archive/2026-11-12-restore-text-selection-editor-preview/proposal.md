## Why

引入预览槽后，目录选中文本文件会经过独立选中预览层和 `PreviewRouter`，重新显示旧的文本预览样式，与打开到预览槽后的编辑器视图不一致。AUTOSAR `.arxml` 文件也缺少 XML 语法高亮。

## What Changes

- 目录选中没有专用预览样式的文本文件时，在选中预览层显示只读的编辑器样式代码视图。
- 保持选中预览层与预览槽的编辑状态、焦点和阅读位置隔离；快速切换选中项时仅显示最新文件。
- 将 `.arxml` 映射为 XML 语法，适用于选中预览和预览槽编辑器。
- Markdown、JSON、ipynb、SVG 等专用预览和二进制预览维持既有行为。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `selection-preview-overlay`: 普通文本选中预览改用只读编辑器样式，并保持预览槽状态隔离。
- `code-file-editor-stability`: `.arxml` 在编辑器中使用 XML 语法高亮。

## Impact

涉及 `SelectionPreviewOverlay.svelte`、编辑器语言映射与可能的只读代码视图组件；不改后端接口或新增依赖。
