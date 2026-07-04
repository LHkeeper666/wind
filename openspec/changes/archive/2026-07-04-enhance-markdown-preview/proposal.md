## Why

MarkdownPreviewer 当前只有基本的 markdown-it 渲染和 Shiki 代码高亮，缺少学术/技术写作常用的 LaTeX 公式、Mermaid 图表、以及 Obsidian 风格的图片语法支持。`katex` 和 `mermaid` 已在 package.json 中但完全未接入。此外，标准 markdown 的本地图片路径在 Tauri 环境下无法正确加载（缺少 `convertFileSrc` 转换）。

## What Changes

- 接入 `markdown-it-texmath` + `katex`，支持 `$...$`、`$$...$$`、`\(...\)`、`\[...\]` 四种 LaTeX 公式语法
- 接入 `mermaid`，后处理 `language-mermaid` 代码块为 Mermaid 图表
- 自定义 markdown-it 插件解析 Obsidian `![[image.png]]`、`![[image.png|200]]`、`![[image.png|200x100]]` 语法
- 修正标准 markdown 图片 `![](path)` 和 Obsidian 图片的本地路径，使用 Tauri `convertFileSrc` 转换为可加载的 `asset://` URL
- 添加 LaTeX 公式和 Mermaid 图表的 CSS 样式

## Capabilities

### New Capabilities
- `latex-math`: LaTeX 数学公式渲染，支持行内和块级公式，基于 katex
- `mermaid-diagrams`: Mermaid 图表渲染，将代码块转换为 SVG 图表
- `obsidian-images`: Obsidian 风格 wikilink 图片语法解析和路径解析
- `local-image-urls`: 本地文件图片路径转换，使用 Tauri convertFileSrc 协议

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- **依赖**: 新增 `markdown-it-texmath`；`katex` 和 `mermaid` 已有
- **代码**: 主要修改 `src/lib/previewers/MarkdownPreviewer.ts`，新增 CSS 样式
- **Tauri 配置**: 可能需要在 `tauri.conf.json` 中启用 `asset` 协议的 CSP 配置
