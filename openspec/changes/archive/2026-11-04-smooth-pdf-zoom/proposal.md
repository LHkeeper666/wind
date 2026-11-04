## Why

PDF 预览目前把每次 Ctrl+滚轮事件转换为固定 15% 缩放，全屏快捷键每次变化 25%；缩放值也直接跳到目标值。鼠标和触控板操作因此显得突兀，难以精细调整阅读比例。

## What Changes

- 普通预览的 Ctrl+滚轮依据滚轮位移连续调整目标比例，并以指针位置作为缩放锚点。
- 普通预览的 Ctrl++/- 以及全屏的 h/l 使用更细的缩放步进；缩放画面逐帧平滑过渡。
- 缩放期间保留已有画面，操作稳定后再补齐清晰内容，避免缩放过程闪白或频繁重绘。
- 全屏预览提供与普通预览一致的 Ctrl+滚轮缩放体验，并保持原有翻页、平移和搜索操作。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `pdf-preview`: 定义连续滚轮缩放、细粒度快捷键缩放、缩放锚点和过渡期间的画面行为。
- `pdf-fullscreen-viewer`: 调整 h/l 缩放行为，并增加 Ctrl+滚轮缩放与画面过渡要求。

## Impact

主要影响 `PdfPreviewPanel.svelte`、`FullscreenPdfViewer.svelte` 及其共用 PDF 缩放/渲染辅助逻辑。保留现有 Tauri PDF 命令接口；无需新增依赖或修改文件格式。
