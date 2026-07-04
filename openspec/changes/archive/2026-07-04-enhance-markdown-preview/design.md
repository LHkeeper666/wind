## Context

MarkdownPreviewer (`src/lib/previewers/MarkdownPreviewer.ts`) 当前使用 `markdown-it` 做基础渲染，`Shiki` 做代码高亮。`katex` 和 `mermaid` 已在 package.json 但未接入。Tauri 应用中本地文件图片需要通过 `convertFileSrc` 转换为 `asset://` 协议 URL 才能加载。

## Goals / Non-Goals

**Goals:**
- 支持 LaTeX 数学公式（行内 `$...$` / `\(...\)` 和块级 `$$...$$` / `\[...\]`）
- 支持 Mermaid 图表渲染（` ```mermaid ` 代码块）
- 支持 Obsidian wikilink 图片语法 `![[image.png]]`
- 修正所有本地图片路径为 Tauri asset URL

**Non-Goals:**
- 不做 Obsidian vault 全局图片搜索（只在当前文件相对路径查找）
- 不做 Obsidian 内部链接 `[[page]]` 的解析（只处理图片）
- 不做 LaTeX 自定义宏或包的支持

## Decisions

### 1. LaTeX: markdown-it-texmath + katex

选择 `markdown-it-texmath` 而非 `markdown-it-katex`，原因：
- 支持四种分隔符：`$...$`、`$$...$$`、`\(...\)`、`\[...\]`
- 活跃维护，与 katex 0.x/1.x 兼容
- `markdown-it-katex` 已停止维护

渲染流程：texmath 插件在 markdown-it 的 token 层面处理公式，直接输出 katex HTML，无需后处理。

### 2. Mermaid: 后处理方案

不使用 markdown-it 插件，而是在 Shiki 后处理阶段识别 `language-mermaid` 代码块，替换为 `mermaid.render()` 生成的 SVG。

原因：mermaid 的渲染是异步的且需要 DOM 容器，不适合在 markdown-it 的同步 token 流中处理。与 Shiki 代码高亮的后处理模式一致。

### 3. Obsidian 图片: 自定义 inline rule

编写一个轻量的 markdown-it inline rule，在 token 阶段将 `![[file.png]]` 解析为标准 img token。解析规则：
- `![[image.png]]` → `<img src="image.png">`
- `![[image.png|200]]` → `<img src="image.png" width="200">`
- `![[image.png|200x100]]` → `<img src="image.png" width="200" height="100">`

### 4. 本地图片路径: 后处理 DOM

在所有渲染完成后，遍历 container 中所有 `<img>` 元素：
- 相对路径（不以 `http`/`https`/`data`/`asset` 开头）→ 使用 `convertFileSrc` 转换
- Obsidian wikilink 图片路径相对于当前文件目录解析
- 标准 markdown 图片路径相对于当前文件目录解析

需要从 render() 的调用方传入当前文件路径信息。查看 PreviewRouter，需要确认如何传递文件路径上下文。

### 5. CSS 样式

在 PreviewEditor.svelte 的 `:global(.preview-markdown)` 样式块中添加：
- `.katex-display` 块级公式居中
- `.mermaid-container` 图表容器样式
- mermaid 错误时的 fallback 样式

## Risks / Trade-offs

- [katex 包体积] katex 约 300KB，首次加载会影响预览速度 → 使用 `throwOnError: false` 降级显示原始文本，避免阻塞
- [mermaid 异步渲染] 图表渲染是异步的，可能出现先显示代码块再替换为图表的闪烁 → 先隐藏代码块，渲染完成后替换
- [图片路径歧义] Obsidian 的 `![[file]]` 在 vault 中全局搜索，我们的实现只在当前目录查找 → 这是设计决策，非 bug
- [CSP 策略] Tauri asset 协议可能需要 CSP 配置 → 需要检查 tauri.conf.json 的 security.csp 设置
