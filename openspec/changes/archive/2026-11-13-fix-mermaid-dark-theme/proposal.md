## Why

Markdown 预览中的 Mermaid 图表按错误的主题状态渲染，深色模式下连线与背景对比不足。主题切换后已有 SVG 和缓存也不会随之更新，导致图表持续显示旧主题颜色。

## What Changes

- Mermaid 渲染使用应用实际的 `data-theme` 状态，分别生成适合深色和浅色模式的图表。
- 主题切换时更新已显示的 Mermaid 图表，包括保留在预览标签中的内容。
- Mermaid 缓存按主题区分，避免复用另一主题生成的 SVG。

## Capabilities

### New Capabilities

- `mermaid-theme-rendering`: Markdown Mermaid 图表随应用主题正确渲染和更新。

### Modified Capabilities

无。

## Impact

涉及 Markdown 预览器的 Mermaid 渲染与缓存、预览组件的主题变化响应。无后端、外部 API 或新依赖变更。
