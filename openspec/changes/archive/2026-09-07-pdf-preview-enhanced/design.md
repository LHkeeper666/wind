## Context

Wind 的 PDF 预览有两套实现：

1. **预览面板** (`PdfPreviewer.ts`)：class-based，实现 `Previewer` 接口，`render()` 方法把后端渲染好的 PNG 放入 `<img>` 标签。翻页由 `PreviewEditor.svelte` 的 `loadPdfPage()` 控制，Shift+J/K 触发。
2. **全屏查看器** (`FullscreenPdfViewer.svelte`)：Svelte 组件，canvas 渲染 + CSS transform 缩放平移，支持 j/k 翻页、h/l 缩放、Ctrl+hjkl 平移、`/` 搜索、预加载缓存。

两者核心渲染逻辑相同（调用 `render_pdf_page` 获取 PNG → canvas 绘制），但交互层完全不同。

## Goals / Non-Goals

**Goals:**
- 预览面板内支持缩放（键盘 h/l + 鼠标滚轮）
- 预览面板内支持平移（键盘 Ctrl+hjkl + 鼠标拖拽）
- 预览面板内支持 vim 风格文本搜索（`/`、`n/N`、高亮）
- j/k 改为翻页（PDF 模式下）
- 焦点隔离：滚轮缩放仅在 PDF 面板聚焦时生效
- 保留 E 键进入全屏查看器

**Non-Goals:**
- 合并全屏查看器和预览面板为同一组件（保持两个独立入口）
- PDF 编辑/注释
- 文本选择与复制

## Decisions

### 1. 组件化：PdfPreviewPanel.svelte

**选择**: 创建 Svelte 组件 `PdfPreviewPanel.svelte`，替换 class-based 的 `PdfPreviewer.ts`。

**替代方案**: 在 `PdfPreviewer.ts` 内用手动 DOM 操作实现 canvas + 缩放平移。

**理由**: 缩放、平移、搜索的交互状态复杂（scale、translateX、translateY、searchQuery、searchResults、currentMatchIndex...），Svelte 的 `$state` 响应式管理远比手动 DOM 操作清晰。且与 `FullscreenPdfViewer.svelte` 可以抽取共享逻辑。

### 2. 集成方式：PreviewEditor 条件渲染

**选择**: 在 `PreviewEditor.svelte` 中，当文件是 PDF 时直接渲染 `<PdfPreviewPanel>` 组件，绕过 PreviewRouter。

**替代方案**: 修改 Previewer 接口支持 Svelte 组件渲染。

**理由**: PreviewEditor 已经对 PDF 有大量特殊处理（`loadPdfPage()`、`pdfCurrentPage`、`pdfPageCount`、`addPdfInfoBar()`），PDF 本质上不走 PreviewRouter 的通用流程。再加一步条件渲染并不增加架构复杂度。

### 3. 滚轮缩放焦点隔离

**选择**: PdfPreviewPanel 内部监听 `wheel` 事件，通过 `stopPropagation()` 阻止冒泡。组件获得焦点时（`tabindex` + `focus`）才处理滚轮缩放，失焦时滚轮事件正常冒泡到全局。

**实现**:
```
PdfPreviewPanel 容器：
  - tabindex="0" 使其可获得焦点
  - on:focus → 标记 hasFocus = true
  - on:blur → 标记 hasFocus = false
  - on:wheel → if (hasFocus) { e.preventDefault(); e.stopPropagation(); zoom(delta) }
```

### 4. 快捷键映射

PDF 预览面板（global-normal 模式下，焦点在 preview 列）：

| 按键 | 行为 |
|------|------|
| j/k | 翻页（上/下一页） |
| h/l | 缩放（缩小/放大） |
| Ctrl+j/k | 平移（上/下） |
| Ctrl+h/l | 平移（左/右） |
| g/G | 跳首/末页 |
| / | 打开搜索栏 |
| n/N | 上/下一个搜索匹配 |
| E | 进入全屏查看器 |
| 鼠标滚轮 | 缩放（仅聚焦时） |
| 鼠标拖拽 | 平移 |

**注意**: j/k 原来用于切换目录中的文件。PDF 模式下改为翻页，与全屏查看器一致。非 PDF 文件的 j/k 行为不变。

### 5. 共享逻辑抽取

`PdfPreviewPanel` 和 `FullscreenPdfViewer` 共享以下逻辑：
- canvas 渲染（PNG → Image → drawImage）
- 搜索高亮绘制（drawSearchHighlights）
- 预加载缓存（pageCache + preloadPages）
- 搜索结果导航（navigateMatch）

抽取为 `src/lib/utils/pdf-shared.ts` 工具函数，两个组件各自调用。

### 6. 缩放行为细节

- 初始缩放：fit-to-panel（页面自适应面板大小）
- 缩放中心：鼠标位置（滚轮缩放时）或面板中心（键盘缩放时）
- 缩放步长：0.25（键盘），滚轮按 deltaY 线性映射
- 最小缩放：0.1x
- 最大缩放：5.0x
- 翻页时重置缩放到 fit-to-panel

## Risks / Trade-offs

**[j/k 语义变更]** → PDF 模式下 j/k 从"切换文件"改为"翻页"。这与图片预览的行为不一致（图片预览中 j/k 仍切换文件）。但与全屏查看器一致，且 PDF 翻页是更常用的操作。用户如果想用 j/k 切换文件，可以先按 Escape 回到目录面板。

**[PreviewEditor 复杂度]** → PreviewEditor.svelte 已经很庞大（2000+ 行），再加 PDF 条件渲染逻辑会更复杂。但 PDF 的特殊处理已经存在，只是把 img 渲染换成 Svelte 组件渲染，增量不大。

**[两个组件功能重叠]** → PdfPreviewPanel 和 FullscreenPdfViewer 功能高度重叠。但保持两个入口有价值：面板适合快速浏览，全屏适合沉浸式阅读。共享逻辑抽取可以减少代码重复。
