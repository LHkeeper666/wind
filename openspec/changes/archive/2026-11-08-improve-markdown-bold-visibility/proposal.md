## Why

Markdown 预览中的加粗文本只比正文增加字重，颜色与正文相同。在当前等宽字体和中文内容下，这种差异不够明显，影响阅读时定位重点。

## What Changes

- 让 Markdown 预览中的 `strong` 和 `b` 文本同时通过字重与主题强调色体现加粗。
- 保证深色与浅色主题中的加粗文本清晰可辨，并检查标题、链接等组合内容的显示效果。

## Capabilities

### New Capabilities

- `markdown-bold-visibility`: 规定 Markdown 预览中加粗文本的视觉表现及主题适配。

### Modified Capabilities

无。

## Impact

- 主要影响 `src/lib/components/PreviewEditor.svelte` 中的 Markdown 预览样式，以及 `src/lib/styles/themes.css` 中已有的主题颜色变量使用。
- 不改变 Markdown 解析、文件内容、后端接口或依赖。
