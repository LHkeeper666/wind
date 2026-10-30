## Why

PDF 页面目前只以 Canvas 图像呈现，用户无法选中并复制其中的文字。虽然已有全文搜索，修改查询词后重新搜索和定位到具体命中位置的体验也不完整。

## What Changes

- 在三栏 PDF 预览和全屏 PDF 视图中支持鼠标选中可提取文字，并通过系统复制快捷键复制选中内容。
- 为 PDF 页面提供与渲染画面对齐的文字层；缩放、滚动和切换页面后仍可准确选择。
- 改进现有搜索：修改查询词后可重新执行搜索，切换命中项时定位到具体文字位置，并保持高亮与当前位置一致。
- 扫描版或仅含图片的 PDF 不增加 OCR；无可提取文字时不显示虚假的文字选择结果。

## Capabilities

### New Capabilities

- `pdf-text-selection`: PDF 预览与全屏视图中的文字层、范围选择及复制行为。

### Modified Capabilities

- `pdf-rendering`: 扩展现有 PDF 文本搜索的交互要求，包括重新搜索和精确定位。

## Impact

- Rust PDFium 命令与返回数据类型：`src-tauri/src/pdf/mod.rs`、`src-tauri/src/lib.rs`。
- 前端 PDF 共享工具、三栏预览和全屏视图：`src/lib/utils/pdf-shared.ts`、`PdfPreviewPanel.svelte`、`FullscreenPdfViewer.svelte`。
- PDF 页面文本数据的按需加载、缓存和失效处理；不要求新增依赖。
