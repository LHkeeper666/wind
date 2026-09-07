## Why

预览面板中的 PDF 预览目前只支持单页静态显示和 J/K 翻页，缺少缩放、平移和搜索能力。用户必须按 E 进入全屏查看器才能使用这些功能，打断了工作流。目标是将全屏查看器的核心交互能力搬到预览面板中，让用户在面板内就能完成大部分 PDF 阅读操作。

## What Changes

- 将 `PdfPreviewer` 从 class-based 的静态 img 显示器重构为 Svelte 组件 `PdfPreviewPanel.svelte`，使用 canvas 渲染 + CSS transform 实现缩放平移
- 新增鼠标滚轮缩放（仅在 PDF 面板获得焦点时生效，避免与全局缩放冲突）
- 新增鼠标拖拽平移
- 新增键盘缩放（h/l）和平移（Ctrl+hjkl）
- 新增 vim 风格文本搜索（`/` 触发，`n/N` 跳转匹配，canvas 高亮）
- 修改 j/k 快捷键语义：从"切换文件"改为"翻页"（PDF 模式下）
- 保留 `E` 键进入全屏查看器的入口
- 保留预加载缓存机制（当前页 ±2 页）

## Capabilities

### New Capabilities
- `pdf-preview-zoom`: 预览面板内 PDF 缩放，支持键盘 h/l 和鼠标滚轮
- `pdf-preview-pan`: 预览面板内 PDF 平移，支持键盘 Ctrl+hjkl 和鼠标拖拽
- `pdf-preview-search`: 预览面板内 vim 风格文本搜索与高亮

### Modified Capabilities
- `pdf-preview`: 从静态 img 显示升级为 canvas 渲染，j/k 改为翻页，新增缩放/平移/搜索

## Impact

- **修改文件**: `PdfPreviewer.ts`（删除或废弃）、`PreviewEditor.svelte`（PDF 渲染逻辑改为组件化）、`PreviewRouter.ts`（移除 PdfPreviewer 注册）
- **新增文件**: `src/lib/components/PdfPreviewPanel.svelte`
- **复用逻辑**: 从 `FullscreenPdfViewer.svelte` 抽取共享的 canvas 渲染、缩放、搜索逻辑
- **无后端变更**: 所有 PDF 渲染命令（`render_pdf_page`、`search_pdf_text`）已存在
