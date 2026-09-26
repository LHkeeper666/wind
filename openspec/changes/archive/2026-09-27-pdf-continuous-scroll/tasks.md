## 1. 后端 API 扩展

- [x] 1.1 修改 `PdfInfo` 结构体，新增 `page_dimensions: Vec<PdfPageDimensions>` 字段
- [x] 1.2 修改 `get_pdf_info`，遍历所有页提取 `width().value` 和 `height().value` 填入 `page_dimensions`
- [x] 1.3 新增 `PdfOutlineItem` 结构体（title, page, x, y, children）
- [x] 1.4 实现 `get_pdf_outline` 命令，使用 PDFium `document.outline()` 遍历书签树
- [x] 1.5 新增 `PdfLinkAnnotation` 结构体（rect, target_page, target_x, target_y, is_external, url）
- [x] 1.6 实现 `get_pdf_page_links` 命令，使用 PDFium `page.annotations()` 提取链接注解
- [x] 1.7 在 `lib.rs` 中注册新命令

## 2. 前端类型与工具扩展

- [x] 2.1 在 `pdf-shared.ts` 中新增 `PdfPageDimensions`、`PdfOutlineItem`、`PdfLinkAnnotation` 接口
- [x] 2.2 新增 `fetchPdfOutline(path)` 和 `fetchPdfPageLinks(path, page)` 工具函数
- [x] 2.3 新增 `cumulativeHeight(pageDimensions, scale)` 工具函数，计算每页的累计高度偏移

## 3. PdfPreviewPanel 重写为连续滚动

- [x] 3.1 重写组件结构：单页 canvas → 可滚动容器 + N 个 page-slot 占位 div
- [x] 3.2 实现 page-slot 高度计算：`height = pointHeight * scale`
- [x] 3.3 实现 IntersectionObserver 虚拟化：进入缓冲区 ±2 页时创建 canvas，离开时销毁
- [x] 3.4 实现 canvas 渲染流程：fetchPdfPage → drawPageWithHighlights → 插入 page-slot
- [x] 3.5 实现 canvas 销毁流程：从 DOM 移除 canvas，保留 PdfPageCache 中的数据
- [x] 3.6 实现缩放：Ctrl+=/- 和 Ctrl+滚轮，更新所有 slot 高度，调整 scrollTop 保持视口中心
- [x] 3.7 实现当前页码追踪：监听 scroll 事件，计算视口中心所在页码
- [x] 3.8 实现搜索高亮：每页 canvas 绘制时叠加 drawSearchHighlights
- [x] 3.9 移除旧的单页渲染逻辑（goToPage、fitToPanel、canvasStyle transform 等）

## 4. 快捷键重写

- [x] 4.1 j/k：向下/上滚动 40px
- [x] 4.2 h/l：向左/右滚动 40px（页面宽度超出视口时生效）
- [x] 4.3 g/G：滚动到顶部/底部
- [x] 4.4 Ctrl+=/-：缩放（拦截 preventDefault + stopPropagation，阻止全局缩放）
- [x] 4.5 /、n/N：搜索保留
- [x] 4.6 滚轮：原生垂直滚动（移除自定义 pan 逻辑）
- [x] 4.7 Ctrl+滚轮：缩放（移除普通滚轮缩放）
- [x] 4.8 移除 Shift+hjkl 平移、h/l 缩放

## 5. 链接注解覆盖层

- [x] 5.1 虚拟化渲染 canvas 时，同时调用 `fetchPdfPageLinks` 获取链接注解
- [x] 5.2 实现链接注解缓存：`Map<number, PdfLinkAnnotation[]>`
- [x] 5.3 在 page-slot 中叠加绝对定位的 link-overlay 容器
- [x] 5.4 实现坐标转换：PDF 坐标（左下原点）→ CSS 百分比定位
- [x] 5.5 实现点击跳转：内部链接滚动到目标页+坐标，外部链接 window.open
- [x] 5.6 实现 cursor: pointer hover 效果

## 6. TocSidebar 集成

- [x] 6.1 创建 `PdfTocSidebar.svelte`，复用 TocSidebar 的树形展示逻辑
- [x] 6.2 实现 `outlineToHeadings()` 数据映射函数
- [x] 6.3 实现点击跳转：`scrollToPage(page)` + 精确 y 坐标偏移
- [x] 6.4 实现当前页高亮：`activeLine` 传入 `currentPage`
- [x] 6.5 在 PreviewEditor 中集成 PdfTocSidebar，右侧显示
- [x] 6.6 实现 `togglePdfToc()` 和 `jumpToPdfPage()` exports

## 7. 命令面板扩展

- [x] 7.1 在 PanelLayout 的 `handleCommandKeydown` 中添加 `:toc` 命令
- [x] 7.2 添加 `:数字` 命令，跳转到指定页码（1-based 输入，0-based 内部）
- [x] 7.3 PanelLayout 拦截 Ctrl+=/-，当焦点在 PDF 面板时不触发全局缩放

## 8. 清理与集成

- [x] 8.1 更新 PdfPreviewPanel 的 props 接口（移除 initialPage、onPageChange 等单页相关 props）
- [x] 8.2 更新 PreviewEditor 中 PdfPreviewPanel 的集成代码
- [x] 8.3 更新 info bar 提示文字
- [x] 8.4 运行 `npx svelte-check` 验证类型安全
- [x] 8.5 运行 `cargo check` 验证 Rust 编译
