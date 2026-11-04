## Why

预览短页、多页 PDF 时，连续按 `j/k` 会看到空白区域和逐块出现的瓦片。现有固定预取与按视口代次丢弃任务的策略跟不上连续跨页滚动，需要让进入视口的页面始终有可用画面。

## What Changes

- 为滚动即将进入视口的页面预先准备完整低清底图，并在高清瓦片就绪前保留底图。
- 根据滚动方向和速度调整有界预取范围；优先完成可见区域和前方页面的低清画面。
- 视口滚动时重新排序可复用的渲染任务，保留已开始任务的可复用结果；文件或缩放档位变化时仍丢弃不兼容结果。
- 明确低清底图、高清瓦片与缓存预算的协作方式，并增加连续滚动的回归验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `pdf-progressive-presentation`: 从首屏低清批量呈现扩展到连续滚动时的完整低清底图与无空白高清替换。
- `pdf-preview-resource-management`: 调整滚动预取、任务代次和缓存保留规则，保证有界资源使用。

## Impact

主要涉及 `PdfPreviewPanel.svelte`、`pdf-shared.ts` 及 PDF 预览测试。优先复用现有 `render_pdf_page`/`render_pdf_tile` 命令；不增加外部依赖或改变持久数据格式。全屏 PDF 查看器不在本次范围内。
