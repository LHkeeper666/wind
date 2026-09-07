## 1. 抽取共享 PDF 工具函数

- [x] 1.1 创建 `src/lib/utils/pdf-shared.ts`，从 `FullscreenPdfViewer.svelte` 抽取 canvas 渲染逻辑（PNG → Image → drawImage）
- [x] 1.2 抽取搜索高亮绘制逻辑（drawSearchHighlights）
- [x] 1.3 抽取预加载缓存逻辑（pageCache + preloadPages）
- [x] 1.4 抽取搜索结果导航逻辑（navigateMatch）

## 2. 创建 PdfPreviewPanel 组件

- [x] 2.1 创建 `src/lib/components/PdfPreviewPanel.svelte`，使用 $state 管理缩放/平移/搜索状态
- [x] 2.2 实现 canvas 渲染（调用共享工具函数）
- [x] 2.3 实现 fit-to-panel 初始缩放和翻页重置
- [x] 2.4 实现键盘缩放（h/l），以面板中心为原点
- [x] 2.5 实现键盘平移（Ctrl+hjkl）
- [x] 2.6 实现鼠标滚轮缩放（以鼠标位置为原点，仅焦点时生效）
- [x] 2.7 实现鼠标拖拽平移
- [x] 2.8 实现 vim 风格搜索（/ 打开、n/N 跳转、Escape 关闭、搜索栏 UI）
- [x] 2.9 实现 canvas 搜索高亮绘制
- [x] 2.10 实现预加载缓存（当前页 ±2 页）
- [x] 2.11 实现底部信息栏（页码、缩放比例、文件大小、快捷键提示）

## 3. 集成到 PreviewEditor

- [x] 3.1 在 `PreviewEditor.svelte` 中导入 PdfPreviewPanel 组件
- [x] 3.2 PDF 文件类型时条件渲染 `<PdfPreviewPanel>` 替代 PreviewRouter 渲染
- [x] 3.3 传递 pdfPath、currentPage、pageCount、fileSize 等 props
- [x] 3.4 处理 pageChange 事件（更新 pdfCurrentPage 状态）
- [x] 3.5 处理 fullscreen 事件（打开 FullscreenPdfViewer）
- [x] 3.6 修改 j/k 快捷键：PDF 模式下 j/k 触发翻页而非切换文件
- [x] 3.7 移除 `addPdfInfoBar()` 调用（信息栏已内置于 PdfPreviewPanel）
- [x] 3.8 处理焦点管理：点击 PDF 面板时获取焦点，焦点在目录面板时 j/k 恢复为文件切换

## 4. 更新 PreviewRouter

- [x] 4.1 从 PreviewRouter 中移除 PdfPreviewer 注册（PDF 不再走 PreviewRouter）
- [x] 4.2 删除 `PdfPreviewer.ts`

## 5. 更新全屏查看器（可选）

- [x] 5.1 重构 `FullscreenPdfViewer.svelte` 使用 `pdf-shared.ts` 共享函数，减少代码重复

## 6. 性能优化：渐进式渲染与 JPEG 编码

- [x] 6.1 后端 `render_pdf_page` 改用 turbojpeg JPEG 编码（quality 92），替代 PNG
- [x] 6.2 JPEG 编码移出 Mutex：Phase 1（Mutex 内）提取 RGBA，Phase 2（Mutex 外）编码+base64
- [x] 6.3 前端渐进式渲染：低清（scale 0.5）先显示，再升级到高清（scale 2.0）
- [x] 6.4 `drawCanvasIntoSlot` 支持动态 MIME 类型（jpeg/png）
- [x] 6.5 低清预加载范围 ±15 页，高清预加载范围 ±5 页

## 7. 连续滚动与缩放

- [x] 7.1 连续滚动模式：所有页面平铺，scroll 定位到具体页
- [x] 7.2 Ctrl+=/- 和鼠标滚轮缩放，保持滚动中心位置
- [x] 7.3 缩放时 canvas/wrapper 尺寸同步更新（$effect 响应式方案）
- [x] 7.4 缩放时 page-slot 高度与 canvas CSS 同一帧更新，避免脱节

## 8. PDF 大纲目录（TOC）

- [x] 8.1 后端 `get_pdf_outline` 提取 PDF 书签树（仅顶层节点，避免重复）
- [x] 8.2 `PdfTocSidebar.svelte` 组件：树形导航、搜索、键盘操作
- [x] 8.3 TOC 切换按钮（☰），与 Markdown 预览风格一致
- [x] 8.4 修复 TOC 二级及以后条目重复显示（`iter()` → `root()` + `next_sibling()`）
- [x] 8.5 搜索跳转后 `rerenderVisiblePages` 正确触发高清升级

## 9. 链接注解

- [x] 9.1 后端 `get_pdf_page_links` 提取页面链接注解
- [x] 9.2 前端链接覆盖层（overlay），支持内部跳转和外部链接
- [x] 9.3 链接定位使用 `pdf-page-wrapper` 相对定位，避免偏移

## 10. 预加载优先级

- [x] 10.1 预加载延迟 100ms，避免与可见页面高清请求竞争 Mutex

## 11. 测试与验证

- [x] 11.1 运行 `npx svelte-check` 验证类型安全
- [x] 11.2 运行 `cargo check` 验证 Rust 编译
