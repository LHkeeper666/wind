## Why

Markdown 文件预览时，正文内容出现明显滞后于大纲（TOC）。原因是 `MarkdownPreviewer.render()` 中 Shiki 语法高亮对每个代码块都创建新的 highlighter 实例，导致重复初始化开销。同时本地图片串行加载、Mermaid 动态 import 也增加了总渲染时间。

## What Changes

- 缓存 Shiki highlighter 实例，避免每个代码块重复创建
- 本地图片并行加载（`Promise.all`）替代串行 `await`
- 缓存 Mermaid 模块引用，避免重复动态 import
- 代码块高亮与图片加载并行执行

## Capabilities

### New Capabilities
- `markdown-render-perf`: Markdown 预览渲染性能优化，包括 Shiki highlighter 缓存、图片并行加载、Mermaid 缓存

### Modified Capabilities

## Impact

- `src/lib/previewers/MarkdownPreviewer.ts` — 主要修改文件
- 无 API 变更、无依赖变更
- 预期效果：大纲与正文出现的时间差显著缩小
