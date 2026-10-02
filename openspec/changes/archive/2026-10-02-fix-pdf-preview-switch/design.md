## Context

`PdfPreviewPanel.svelte` 使用 tile 渲染系统来显示 PDF 内容。当用户在文件夹中选中不同 PDF 文件时，组件通过路径切换 `$effect` 检测 `pdfPath` 变化并重置内部状态。

当前的路径切换效果清理了：
- `tileCache` — 内存中的 tile 缓存
- `renderingTiles` — 正在渲染的 tile 集合
- `renderedTileData` — 已渲染 tile 的数据引用
- `renderedPages` — 已渲染页面集合
- `loadedLinks` — 已加载链接集合
- `renderScheduler` — 渲染调度器队列

但**没有清理 `scrollEl` DOM 中已有的 tile canvas 元素**。

当新旧 PDF 的 `pageCount` 相同时，Svelte 的 `{#each { length: totalPages }}` 不会重建 page-slot DOM 元素。`getTileLayer` 函数复用已存在的 `.pdf-page-wrapper`，导致旧 tile canvas 残留在 DOM 中。

## Goals / Non-Goals

**Goals:**
- 修复 PDF 切换时预览不更新的问题
- 确保路径切换后 DOM 状态干净

**Non-Goals:**
- 不改变 tile 渲染架构
- 不影响单个 PDF 的预览性能

## Decisions

### 在路径切换效果中清理 scrollEl DOM

**选择**：在路径切换 `$effect` 中增加 `if (scrollEl) scrollEl.innerHTML = '';`

**理由**：
- 最直接的修复方式，一行代码解决问题
- 与已有的清理逻辑（tileCache.clear、renderedPages.clear 等）在同一位置，语义一致
- 不影响其他组件，修改范围最小

**替代方案**：
- 在 `getTileLayer` 中检测路径变化并清理旧内容 — 增加了渲染路径的复杂度
- 使用 keyed each `{#each pages as page (page.path)}` — 需要重构整个渲染系统

## Risks / Trade-offs

- **风险**：`scrollEl.innerHTML = ''` 会同时移除所有子元素包括滚动位置锚点 → **缓解**：路径切换后会重新初始化 scale 和滚动位置，无需保留旧状态
- **风险**：如果 `scrollEl` 在路径切换时为 null（组件已销毁）→ **缓解**：已有 `if (scrollEl)` 守卫