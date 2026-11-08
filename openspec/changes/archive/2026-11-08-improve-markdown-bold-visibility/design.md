## Context

Markdown 预览由 `MarkdownPreviewer` 生成 HTML，`PreviewEditor.svelte` 统一设置 `.preview-markdown` 样式。正文和 `strong` 目前都使用 `--text-primary`；加粗仅依赖 700 字重。深色主题的 `--warning` 对比度充足，但浅色主题的同名颜色对比度不足。

## Goals / Non-Goals

**Goals:**
- 在正文中使加粗文字明显区别于相邻普通文字。
- 使用现有主题变量，让深浅主题均保持一致的强调语义。

**Non-Goals:**
- 不改变 Markdown 解析规则、编辑器语法高亮或全局字体。
- 不引入新的颜色值、字体文件或依赖。

## Decisions

- 在 `.preview-markdown` 范围内为 `strong` 和 `b` 统一设置 700 字重及局部 `--md-bold-color`。加粗色由 60% `--text-primary` 与 40% 主题色混合：深色主题使用 `--warning`，浅色主题使用 `--accent`。这样正文加粗保持可辨，同时不会压过四级标题。`strong` 来自 Markdown 加粗语法，`b` 覆盖内嵌 HTML。
- 四级标题采用完整的 `--accent`，字号为 `1.15em`，保持标题与正文加粗的层级差异。三级标题维持 `1.25em`，并通过宽 `2em`、高 `2px` 的短下划线与四级标题区分。
- 浅色主题下，表格行悬停背景与 `--accent` 的对比度不足；该状态中的加粗文字改用 `--text-primary`，保留 700 字重。标题、链接、引用和表格均检查组合显示效果。

## Risks / Trade-offs

- [同一段落中加粗过多时强调色过于突出] → 使用已有的主题色，并以包含多处加粗的示例检查视觉密度。
- [加粗嵌套在链接或标题中时颜色层级不清] → 在两种主题下检查组合内容，并限定例外选择器的作用范围。
