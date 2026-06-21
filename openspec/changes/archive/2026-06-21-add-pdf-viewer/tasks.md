## 1. Rust 后端：PDF 依赖与基础模块

- [x] 1.1 在 Cargo.toml 中添加 pdfium-render 依赖，配置 pdfium 预编译二进制引入
- [x] 1.2 创建 `src-tauri/src/pdf/mod.rs`，实现 `get_pdf_info` 命令
- [x] 1.3 实现 `render_pdf_page` 命令，返回 PNG bytes 和尺寸
- [x] 1.4 实现 `search_pdf_text` 命令，返回每页匹配坐标
- [x] 1.5 在 `lib.rs` 中注册三个新 Tauri 命令

## 2. 前端：PdfPreviewer 预览器

- [x] 2.1 创建 `src/lib/previewers/PdfPreviewer.ts`，实现 match/render/dispose 接口
- [x] 2.2 在 `PreviewRouter.ts` 中注册 PdfPreviewer（在 ImagePreviewer 之前）
- [x] 2.3 在 `PreviewEditor.svelte` 的 `loadFile` 中添加 PDF 类型识别和加载逻辑

## 3. 前端：预览面板 PDF 翻页

- [x] 3.1 在 `PreviewEditor.svelte` 中添加 PDF 页码状态管理
- [x] 3.2 实现 J/K 翻页快捷键处理
- [x] 3.3 预览面板底部信息栏显示 PDF 页码和文件大小

## 4. 前端：FullscreenPdfViewer 全屏查看器

- [x] 4.1 创建 `FullscreenPdfViewer.svelte` 组件，实现 canvas 渲染和自适应窗口
- [x] 4.2 实现 j/k 翻页、g/G 首末页跳转
- [x] 4.3 实现 h/l 缩放和 Ctrl+hjkl 平移
- [x] 4.4 实现预加载逻辑（当前页 ±2 页）
- [x] 4.5 实现底部状态栏（文件名、页码、缩放比例、文件大小、快捷键提示）

## 5. 前端：全屏 PDF 文本搜索

- [x] 5.1 实现 `/` 打开搜索框、Escape 关闭搜索框
- [x] 5.2 实现搜索调用 `search_pdf_text` 并跳转到第一个匹配页
- [x] 5.3 实现 canvas 上的半透明矩形高亮绘制
- [x] 5.4 实现 n/N 跳转上一个/下一个匹配（支持跨页）

## 6. 集成：Layout 与 PanelLayout

- [x] 6.1 在 `layout.ts` 中添加 `fullscreenPdfViewerOpen` 状态和相关方法
- [x] 6.2 在 `PanelLayout.svelte` 中集成 FullscreenPdfViewer 组件
- [x] 6.3 在 `PreviewEditor.svelte` 的 E 键处理中添加 PDF 全屏查看器入口
- [x] 6.4 实现 PDF 文件列表收集（`getPdfFiles` 类似 `getImageFiles`）

## 7. 测试与验证

- [ ] 7.1 测试 PDF 预览：选中 PDF 文件后预览面板正确显示第 1 页
- [ ] 7.2 测试 J/K 翻页和页码更新
- [ ] 7.3 测试全屏查看器：翻页、缩放、平移功能
- [ ] 7.4 测试文本搜索：搜索、高亮、n/N 跳转
- [ ] 7.5 测试大 PDF（100+ 页）的加载性能和预加载效果
- [ ] 7.6 运行 `npm run tauri dev` 验证完整流程
