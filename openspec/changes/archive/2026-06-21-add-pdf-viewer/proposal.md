## Why

Wind 目前支持图片、文本、Markdown、JSON 等文件的预览，但不支持 PDF。PDF 是文档工作中最常见的格式之一，缺少 PDF 查看能力使得用户必须跳出应用使用外部查看器，破坏了 vim-driven file workstation 的工作流连续性。

## What Changes

- 新增 Rust 后端 PDF 渲染能力，基于 pdfium-render 静态链接 pdfium，提供页面渲染、元数据提取、文本搜索三个 Tauri 命令
- 新增 `PdfPreviewer` 预览器，在预览面板中显示 PDF 当前页，支持 J/K 翻页
- 新增 `FullscreenPdfViewer` 全屏查看器，支持翻页、缩放、平移、文本搜索与高亮
- 将 `.pdf` 扩展名注册到 PreviewRouter 路由链中
- 在 PreviewEditor 中增加 PDF 文件类型识别和专用加载逻辑
- 在 layout store 中增加 `fullscreenPdfViewerOpen` 状态

## Capabilities

### New Capabilities
- `pdf-rendering`: PDF 页面渲染、元数据提取、文本搜索的后端能力
- `pdf-preview`: 预览面板中的 PDF 单页显示与翻页
- `pdf-fullscreen-viewer`: 全屏 PDF 查看器，支持缩放、平移、文本搜索高亮

### Modified Capabilities
- `file-preview`: 将 PDF 加入支持的文件类型，PreviewRouter 路由链新增 PdfPreviewer

## Impact

- **新增依赖**: Cargo.toml 添加 `pdfium-render`，需要引入 pdfium 预编译二进制（+3-5MB 包体积）
- **修改文件**: `PreviewRouter.ts`, `PreviewEditor.svelte`, `PanelLayout.svelte`, `layout.ts`, `lib.rs`
- **新增文件**: `PdfPreviewer.ts`, `FullscreenPdfViewer.svelte`, `src-tauri/src/pdf/mod.rs`
