## Context

Wind 的预览系统基于 `PreviewRouter` 路由链，每个预览器实现 `match/render/dispose` 接口。图片预览使用 `ImagePreviewer`（面板内显示）+ `FullscreenImageViewer`（全屏查看器）的双层架构。PDF 目前完全不支持。

当前架构约束：
- 预览器通过 `render(content, container)` 接收内容，content 是 `string | ArrayBuffer`
- 全屏查看器是独立的 Svelte 组件，通过 layout store 的状态控制显隐
- PreviewEditor 负责文件类型判断和内容加载，不经过 PreviewRouter 处理的类型会显示 "unsupported"

## Goals / Non-Goals

**Goals:**
- 在预览面板中显示 PDF 页面，支持翻页
- 全屏查看器支持翻页、缩放、平移、文本搜索与高亮
- 快捷键逻辑与图片查看器保持一致
- 按需渲染，大 PDF 不卡顿

**Non-Goals:**
- PDF 编辑/注释/填写表单
- PDF 文本选择与复制（后续可扩展）
- PDF 打印功能
- 加密/受密码保护的 PDF

## Decisions

### 1. 渲染引擎：pdfium-render

**选择**: 使用 `pdfium-render` crate，静态链接 Google 的 pdfium 库。

**替代方案**:
- `pdf` crate：纯 Rust，包体积小（+200KB），但渲染质量和功能不如 pdfium
- PDF.js（纯前端）：功能全，但无法与 Rust 后端的缩略图/搜索架构统一

**理由**: pdfium 渲染质量最高，同时支持文本提取（搜索用），且与现有的"后端渲染 → 前端展示图片"架构完全一致。

### 2. Rust API 设计：三个 Tauri 命令

```
get_pdf_info(path: String) → PdfInfo
  - page_count: u32
  - title: Option<String>
  - author: Option<String>
  - file_size: u64

render_pdf_page(path: String, page: u32, scale: f64) → PdfPageResult
  - data: Vec<u8>        // PNG bytes
  - width: u32
  - height: u32

search_pdf_text(path: String, query: String) → Vec<PdfSearchResult>
  - PdfSearchResult { page: u32, matches: Vec<TextMatch> }
  - TextMatch { x: f64, y: f64, width: f64, height: f64 }
```

**理由**: 分离关注点 — 预览面板只用 info + render（轻量），搜索按需调用（用户按 `/` 时才触发）。

### 3. 预览面板：单页显示 + 翻页

预览面板显示一页 PDF，底部信息栏显示文件名、页码（如 "3/45"）、文件大小。

**快捷键**:
- `j/k` — 切换目录里的文件（与图片一致，不用于 PDF 翻页）
- `J/K` — PDF 翻页（上/下一页）
- `E` — 进入全屏查看器

**理由**: 保持 j/k 的语义一致性（切换文件），PDF 翻页用大写 J/K 区分。

### 4. 全屏查看器：Canvas 渲染 + 搜索高亮

使用 `<canvas>` 元素而非 `<img>`，支持在同一层上绘制搜索高亮矩形。

**渲染流程**:
1. `render_pdf_page` 获取 PNG bytes
2. 创建 `Image` 对象加载 PNG
3. `ctx.drawImage(img)` 绘制页面
4. 搜索命中时 `ctx.fillStyle = rgba(255,255,0,0.3)` + `ctx.fillRect()` 绘制高亮

**预加载**: 当前页 ±2 页，翻页时零延迟。

### 5. 搜索架构：按需触发

- 预览面板不触发搜索
- 全屏查看器中用户按 `/` 时调用 `search_pdf_text`
- 返回所有页的匹配坐标，前端跳到第一个匹配页
- `n/N` 在匹配间跳转，跨页时自动渲染目标页

### 6. pdfium 预编译二进制引入

使用 `pdfium-render` 的 `pdfium_private` feature，通过 build script 引入预编译的 pdfium 静态库。从 [pdfium-binaries](https://github.com/nicedoc/nicedoc/pdfium-binaries) 下载对应平台的二进制。

## Risks / Trade-offs

**[包体积增加 3-5MB]** → 使用静态链接，不依赖用户系统环境。对于桌面应用可接受。

**[大 PDF 搜索延迟]** → 按需触发（用户按 `/` 才搜索），不预加载。搜索期间显示 loading 状态。后续可优化为增量搜索（先搜当前页 ±N 页）。

**[内存占用]** → 默认 scale=1.5 渲染，预加载 ±2 页（最多 5 页在内存中）。单页 PNG 约 1-3MB，总占用 5-15MB，可接受。

**[跨平台 pdfium 二进制]** → 需要为 Windows/macOS/Linux 分别引入预编译二进制。当前项目只针对 Windows，优先处理 Windows 平台。
