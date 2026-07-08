## Why

Markdown 预览的首次加载被 KaTeX 同步渲染阻塞（公式多的文档首次绘制延迟 100-200ms），且每次 tab 切换都从 markdown 源码重新解析渲染，浪费了已经完成的渲染结果。

## What Changes

- **KaTeX 异步渲染**：markdown-it 渲染时跳过 KaTeX，输出占位符；首次绘制后在 `requestAnimationFrame` 中分批异步渲染公式，替换占位符。首次可交互时间从 ~200ms 降到 ~50ms。
- **预览 DOM 缓存**：在 `PreviewEditor` 中按 filePath 缓存渲染完成的 DOM 节点（含 KaTeX/Shiki/Mermaid 后处理结果）。切换 tab 时 detach 当前 DOM 存入缓存，从缓存取出目标 tab 的 DOM 直接 attach，跳过整个 markdown→HTML 渲染管线。LRU 淘汰，上限 5 个。
- **Tab 切换复用缓存**：`loadFile()` 检测到 tabEditorCache 命中 + 预览 DOM 缓存命中时，跳过 `read_file` invoke 和 markdown 渲染，直接 restore DOM。

## Capabilities

### New Capabilities
<!-- None — all changes modify existing capabilities -->

### Modified Capabilities
- `markdown-render-perf`: 新增 KaTeX 异步渲染和预览 DOM 缓存两个 requirement
- `tab-state-persistence`: 新增预览 DOM 作为 tab 状态的一部分被保存/恢复的 requirement

## Impact

- `src/lib/previewers/MarkdownPreviewer.ts` — KaTeX 插件改为输出占位符，新增异步替换方法
- `src/lib/components/PreviewEditor.svelte` — Preview 缓存 Map，detach/attach 逻辑，LRU 淘汰
