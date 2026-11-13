## Context

应用主题由根元素 `data-theme` 控制，缺省为深色。`MarkdownPreviewer.renderMermaidBlock()` 当前读取 `dark` class，因而总是选择 Mermaid `default` 主题。Mermaid SVG 缓存在预览器中只按图表源码索引，预览组件还会保留各标签的 DOM；两者都可能在主题切换后保留旧配色。

## Goals / Non-Goals

**Goals:**
- 首次渲染、增量渲染和主题切换后，Mermaid 图表颜色与应用主题一致。
- 更新已显示和保留在标签槽中的 Mermaid 图表，并保持预览位置。
- 继续复用同一主题的 SVG 缓存。

**Non-Goals:**
- 改变普通 Markdown、代码高亮或用户在 Mermaid 源码中明确指定的样式。
- 引入新主题库或修改 Mermaid 源码语法。

## Decisions

1. 以 `document.documentElement.dataset.theme` 为主题来源；仅 `light` 使用 Mermaid `default`，其余状态使用 `dark`。这与应用缺省深色行为一致。不会再依赖未设置的 `dark` class。
2. 将主题纳入 Mermaid 缓存键，并保留图块的原始 Mermaid 源码（可用 DOM 关联数据或预览器维护的映射）。缓存命中时仍能在下一次主题切换后重新生成 SVG。相较于缓存失效后整篇重绘，逐块更新可保留 Markdown DOM、滚动位置及其他异步预览状态。
3. 在预览组件监听根元素 `data-theme` 变化，遍历当前及保留标签槽中的 Mermaid 图块，按新主题重新渲染并替换 SVG。组件销毁时解除监听；异步完成前确认目标仍存在且主题未再次变化，避免旧结果覆盖新结果。
4. Mermaid 的 `initialize` 是模块级配置。若多个图块并行渲染或主题快速切换，应保证配置与相应 `render` 调用配对，避免跨主题竞争；可对 Mermaid 渲染任务串行化。其他 Markdown 后处理仍保持并行。

## Risks / Trade-offs

- [主题切换时大量图块重新渲染造成短暂延迟] → 仅处理 Mermaid 图块并保留原 SVG 直到新 SVG 就绪。
- [异步渲染完成顺序与主题切换顺序不一致] → 替换前检查主题版本和目标节点连接状态。
- [源码存放在 DOM 属性时增加 DOM 大小] → 优先采用映射或轻量关联数据，并在预览清理时释放。
