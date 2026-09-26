# PDF 连续滚动 - 设计文档

## 架构总览

```
┌─────────────────────────────────────────────────────────┐
│  PdfPreviewPanel (连续滚动模式)                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │  scroll-container (overflow-y: auto)              │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │  page-slot[0] (高度 = pointH[0] * scale)    │  │  │
│  │  │  ┌───────────────────────────────────────┐  │  │  │
│  │  │  │  canvas (RENDER_SCALE 渲染)           │  │  │  │
│  │  │  │  link-overlay (透明链接注解层)         │  │  │  │
│  │  │  └───────────────────────────────────────┘  │  │  │
│  │  ├─────────────────────────────────────────────┤  │  │
│  │  │  page-slot[1] (占位 div，未渲染)            │  │  │
│  │  ├─────────────────────────────────────────────┤  │  │
│  │  │  page-slot[2] (canvas + overlay)            │  │  │
│  │  ├─────────────────────────────────────────────┤  │  │
│  │  │  ...                                        │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌──────────────────┐  ┌────────────────────────────┐   │
│  │  search bar      │  │  info bar (页码/缩放/大小)  │   │
│  └──────────────────┘  └────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
         右侧 ──────────────────────────────
         ┌──────────────────────────────┐
         │  PdfTocSidebar               │
         │  (内置书签树 + 点击跳转)      │
         └──────────────────────────────┘
```

## 1. 后端 API 扩展

### 1.1 扩展 `get_pdf_info` — 返回每页 point 尺寸

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfPageDimensions {
    width: f32,   // PDF points (1/72 inch)
    height: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfInfo {
    page_count: u32,
    title: Option<String>,
    author: Option<String>,
    file_size: u64,
    page_dimensions: Vec<PdfPageDimensions>,  // 新增
}
```

前端在打开 PDF 时调用一次，获取所有页尺寸用于占位 div 高度计算。加载 PDFium 文档后遍历 `document.pages()` 取每页的 `width().value` 和 `height().value`。

### 1.2 新增 `get_pdf_outline` — 提取内置书签

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfOutlineItem {
    title: String,
    page: u32,           // 目标页码 (0-based)
    x: f32,              // 目标 x 坐标 (PDF points)
    y: f32,              // 目标 y 坐标 (PDF points, 从页面底部算)
    children: Vec<PdfOutlineItem>,
}

#[tauri::command]
pub fn get_pdf_outline(path: String, app_handle: tauri::AppHandle) -> Result<Vec<PdfOutlineItem>, String>
```

使用 PDFium 的 `document.outline()` 遍历书签树。每个 `PdfBookmark` 提供 `title()`、`get_destination()` (含页码和坐标)、`children()`。

### 1.3 新增 `get_pdf_page_links` — 提取链接注解

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfLinkAnnotation {
    rect: [f32; 4],      // [left, bottom, right, top] in page points
    target_page: u32,     // 目标页码 (0-based)
    target_x: f32,        // 目标 x 坐标
    target_y: f32,        // 目标 y 坐标
    is_external: bool,    // 是否外部 URL
    url: Option<String>,  // 外部 URL (is_external=true 时有值)
}

#[tauri::command]
pub fn get_pdf_page_links(path: String, page: u32, app_handle: tauri::AppHandle) -> Result<Vec<PdfLinkAnnotation>, String>
```

使用 PDFium 的 `page.annotations()` 遍历注解，过滤 `PdfAnnotation::Link` 类型。提取 `get_link_annotation()` 的目标信息和矩形区域。

## 2. 前端连续滚动架构

### 2.1 数据流

```
打开 PDF
  │
  ├─ get_pdf_info(path) → { pageCount, pageDimensions[], fileSize }
  │
  ├─ get_pdf_outline(path) → outline tree
  │
  └─ 初始化 PdfPreviewPanel:
       ├─ 计算每页占位高度: pageSlotHeight[i] = pageDimensions[i].height * scale
       ├─ 创建 N 个占位 div (只设置高度，不渲染)
       └─ IntersectionObserver 监听 → 进入缓冲区的页触发渲染
```

### 2.2 虚拟化渲染

**策略**：IntersectionObserver 监听每个 `page-slot` div。当 slot 进入视口上下各 `BUFFER_PAGES` 页的范围时，触发该页的 canvas 渲染；离开范围后销毁 canvas（保留占位 div）。

```
BUFFER_PAGES = 2

视口可见: page 5
已渲染:   page 3, 4, 5, 6, 7
占位:     page 0, 1, 2, 8, 9, ... N

用户滚动到 page 6:
已渲染:   page 4, 5, 6, 7, 8
销毁:     page 3 的 canvas
创建:     page 8 的 canvas
```

**canvas 创建流程**：
1. `fetchPdfPage(path, pageNum, cache)` 获取 PNG data（PdfPageCache 复用）
2. `drawPageWithHighlights(canvas, data, searchState, pageNum)` 绘制到 canvas
3. canvas 插入 page-slot div 中，用 CSS `width: 100%; height: auto` 自适应 slot 宽度

**canvas 销毁**：
- 从 DOM 移除 canvas 元素
- PdfPageCache 中保留 PNG data（滚回来时直接重建，无需重新 fetch）

### 2.3 缩放

**交互**：
- `Ctrl + =` / `Ctrl + -`：放大/缩小（拦截，不触发全局缩放）
- `Ctrl + 鼠标滚轮`：缩放

**实现**：
1. 更新 `scale` 状态
2. 立即更新所有 page-slot 的高度: `slot.style.height = pointHeight * scale + 'px'`
3. 调整 `scrollTop` 保持视口中心对应内容不变:
   ```
   const centerOffset = scrollTop + viewportHeight / 2;
   const contentRatio = centerOffset / oldTotalHeight;
   scrollTop = contentRatio * newTotalHeight - viewportHeight / 2;
   ```
4. 已渲染的 canvas 用 CSS `transform: scale(newScale/oldScale)` 临时缩放（保持清晰度可接受）
5. 异步重新渲染可见页的 canvas（用新 scale 调用 `render_pdf_page`），渲染完后替换并移除 CSS transform

**注意**：`render_pdf_page` 的 scale 参数影响渲染分辨率。当前 RENDER_SCALE=3.0 是固定的，canvas 尺寸始终是 3x point 尺寸。缩放只影响 CSS 显示大小，不影响渲染分辨率。因此不需要因缩放重新渲染——canvas 始终是高清的，CSS scale 只是缩小显示。

这意味着**缩放只需要步骤 1-3**，不需要重新渲染 canvas。

### 2.4 当前页码追踪

监听 scroll 容器的 `scroll` 事件：

```typescript
function updateCurrentPage() {
  const center = scrollTop + viewportHeight / 2;
  let accHeight = 0;
  for (let i = 0; i < totalPages; i++) {
    accHeight += pageSlotHeight[i];
    if (accHeight >= center) {
      currentPage = i;
      break;
    }
  }
}
```

## 3. TocSidebar 适配

复用现有 `TocSidebar.svelte` 组件，适配 PDF 书签数据：

### 3.1 数据映射

PDF 的 `PdfOutlineItem` 树映射为 `TocHeading` 树：

```typescript
function outlineToHeadings(items: PdfOutlineItem[]): TocHeading[] {
  return items.map(item => ({
    text: item.title,
    line: item.page,           // 用 page 作为 line（TocSidebar 用 line 做跳转定位）
    expanded: true,            // 默认展开
    children: outlineToHeadings(item.children),
  }));
}
```

### 3.2 跳转行为

TocSidebar 的 `onJump(line)` 回调中，`line` 实际是页码：

```typescript
function handleTocJump(page: number) {
  scrollToPage(page);  // 滚动到目标页顶部
}
```

如需精确位置（y 坐标），需要额外传递。可扩展 `onJump` 为 `onJump(line, metadata)` 或用单独的映射表存储每条书签的精确坐标。

### 3.3 当前页高亮

`activeLine` 传入 `currentPage`，TocSidebar 会自动高亮当前页对应的书签条目。

### 3.4 位置

右侧，与 Markdown TOC 一致。在 PdfPreviewPanel 的模板中条件渲染：

```svelte
{#if showToc && outlineItems.length > 0}
  <PdfTocSidebar
    headings={outlineItems}
    activeLine={currentPage}
    onJump={handleTocJump}
    onFocusChange={(f) => tocFocused = f}
  />
{/if}
```

## 4. 链接注解覆盖层

### 4.1 数据获取

虚拟化渲染 canvas 时，同时调用 `get_pdf_page_links(path, pageNum)` 获取该页的链接注解。结果缓存在 `Map<number, PdfLinkAnnotation[]>` 中。

### 4.2 覆盖层 DOM

每个 page-slot 内，canvas 上方叠加一个绝对定位的容器：

```html
<div class="page-slot" style="height: {pointH * scale}px; position: relative;">
  <canvas ... style="width: 100%; height: auto;" />
  <div class="link-overlay">
    <!-- 每个链接注解一个透明 div -->
    <div class="link-rect" style="
      left: {rect.left / pointW * 100}%;
      bottom: {rect.bottom / pointH * 100}%;
      width: {(rect.right - rect.left) / pointW * 100}%;
      height: {(rect.top - rect.bottom) / pointH * 100}%;
    " onclick={() => jumpToLink(link)} />
  </div>
</div>
```

### 4.3 坐标转换

PDF 坐标系：原点在左下，y 向上。
CSS 坐标系：原点在左上，y 向下。

```
cssLeft   = rect.left / pagePointWidth * 100%
cssBottom = rect.bottom / pagePointHeight * 100%  (CSS bottom)
cssWidth  = (rect.right - rect.left) / pagePointWidth * 100%
cssHeight = (rect.top - rect.bottom) / pagePointHeight * 100%
```

### 4.4 跳转行为

点击链接注解：滚动到 `targetPage`，然后偏移 `targetY`：

```typescript
function jumpToLink(link: PdfLinkAnnotation) {
  const pageOffset = cumHeight[link.target_page];  // 目标页之前的累计高度
  const yOffset = (pageDimensions[link.target_page].height - link.target_y) * scale;
  scrollContainer.scrollTop = pageOffset + yOffset;
}
```

外部 URL：`window.open(link.url)`。

## 5. 命令与快捷键

### 5.1 命令面板扩展

在 `PanelLayout.svelte` 的 `handleCommandKeydown` 中添加：

```typescript
// :toc — 切换 PDF 目录侧边栏
if (q === 'toc') {
  previewEditor?.togglePdfToc();
  showCommandPalette = false;
  return;
}

// :数字 — 跳转到指定页
const pageNum = parseInt(q);
if (!isNaN(pageNum) && pageNum >= 1) {
  previewEditor?.jumpToPdfPage(pageNum - 1);  // 1-based → 0-based
  showCommandPalette = false;
  return;
}
```

### 5.2 快捷键变更

PdfPreviewPanel 的 `handleKeydown`：

| 键 | 行为 | 条件 |
|---|---|---|
| j | 向下滚动 40px | 页面高度未超出视口时 |
| k | 向上滚动 40px | 同上 |
| h | 向左滚动 40px | 页面宽度超出视口时 |
| l | 向右滚动 40px | 同上 |
| g | 滚动到顶部 | |
| G | 滚动到底部 | |
| Ctrl+= | 放大 | 拦截，阻止全局缩放 |
| Ctrl+- | 缩小 | 拦截，阻止全局缩放 |
| / | 搜索 | 保留 |
| n | 下一个搜索匹配 | 保留 |
| N | 上一个搜索匹配 | 保留 |
| 滚轮 | 垂直滚动 | 原生行为 |
| Ctrl+滚轮 | 缩放 | 拦截 |

移除：Shift+hjkl（平移）、h/l 缩放（改为 Ctrl+=/-）

### 5.3 焦点隔离

与当前实现一致：PanelLayout 全局 keydown/wheel 在 capture 阶段，检测 `.pdf-preview-panel` 时跳过。

需要新增：拦截 `Ctrl+=` 和 `Ctrl+-`，在 PdfPreviewPanel 内消费，不传播到全局缩放。

## 6. 搜索

连续滚动模式下搜索逻辑不变：
- `searchPdfText(path, query)` 搜索所有页
- 结果存入 `searchState`
- 渲染时 `drawSearchHighlights` 在每页 canvas 上绘制高亮
- n/N 跳转时 `scrollToPage(targetPage)` 滚动到目标页

## 7. PreviewEditor 集成

### 7.1 新增 props/exports

PreviewEditor 需要暴露：
- `togglePdfToc()`：切换 PDF TOC 侧边栏
- `jumpToPdfPage(page)`：跳转到指定页

### 7.2 模板结构

```svelte
{#if filePath && isPdfFile(filePath) && mode === 'global-normal'}
  <div class="pdf-container">
    <PdfPreviewPanel ... />
    {#if pdfTocVisible && pdfOutline.length > 0}
      <PdfTocSidebar ... />
    {/if}
  </div>
{/if}
```

### 7.3 TOC 命令传递

PanelLayout 通过 `previewEditor.togglePdfToc()` 调用，PreviewEditor 切换 `pdfTocVisible` 状态。PdfPreviewPanel 不需要知道 TOC 的存在——它只负责渲染和滚动。

## 8. 关键文件变更

| 文件 | 变更类型 | 说明 |
|---|---|---|
| `src-tauri/src/pdf/mod.rs` | 修改 | 扩展 PdfInfo，新增 get_pdf_outline、get_pdf_page_links |
| `src/lib/utils/pdf-shared.ts` | 修改 | 新增 outline/link 接口，扩展 fetchPdfPage 等 |
| `src/lib/components/PdfPreviewPanel.svelte` | 重写 | 单页 → 连续滚动 + 虚拟化 |
| `src/lib/components/PdfTocSidebar.svelte` | 新建 | 复用 TocSidebar 模式，适配 PDF 书签 |
| `src/lib/components/PreviewEditor.svelte` | 修改 | 集成 TOC 侧边栏，暴露 togglePdfToc/jumpToPdfPage |
| `src/lib/components/PanelLayout.svelte` | 修改 | 添加 :toc 和 :数字 命令，拦截 Ctrl+=/- |
