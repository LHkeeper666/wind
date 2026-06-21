## Context

Wind 的预览系统由 `PreviewRouter` 调度多个 `Previewer` 实现，切换文件时通过 `dispose()` + `render()` 完成内容替换。当前实现在三个环节产生闪烁：异步数据未就绪时触发预览、dispose 清空 DOM 后等待 render、图片 blob URL 解码延迟。

涉及文件：
- `src/lib/components/PreviewEditor.svelte` — 文件加载和预览触发
- `src/lib/previewers/PreviewRouter.ts` — 预览路由和生命周期管理
- `src/lib/previewers/ImagePreviewer.ts` — 图片预览实现
- `src/lib/previewers/TextPreviewer.ts` — 文本预览实现
- `src/lib/previewers/MarkdownPreviewer.ts` — Markdown 预览实现
- `src/lib/previewers/JsonPreviewer.ts` — JSON 预览实现

## Goals / Non-Goals

**Goals:**
- 消除文件切换时的闪烁现象（空白帧和旧内容残留）
- 图片预览在图片解码完成后才显示，避免布局抖动
- 保持现有 Previewer 接口不变，改动集中在内部实现

**Non-Goals:**
- 不添加加载动画/骨架屏（后续可独立做）
- 不改变预览器的匹配优先级或支持的文件类型
- 不优化首次加载性能（只解决切换闪烁）

## Decisions

### Decision 1: loadFile 先加载后触发

**选择**: 将 `mode = 'global-normal'` 移到 `await invoke(...)` 之后

**理由**: 当前 `loadFile` 在函数开头设置 `mode`，触发 `$effect` 时异步数据还没加载完，导致用旧内容渲染。改为先加载数据再更新状态，确保 effect 触发时数据已就绪。

**替代方案**: 在 effect 中检查数据是否就绪并跳过渲染 — 增加了状态判断复杂度，不如从源头保证时序。

### Decision 2: PreviewRouter 原子替换

**选择**: 用 `DocumentFragment` 实现原子替换 — 先渲染到临时容器，再一次性替换到 DOM

**理由**: 当前 `dispose()` 清空 `innerHTML = ''` 后，下一帧才 `render()` 写入新内容。用 DocumentFragment 可以在内存中完成渲染，然后一次性替换，浏览器只重绘一次。

**替代方案**: CSS opacity 过渡遮盖 — 只是视觉欺骗，不解决根本问题。

### Decision 3: 图片预加载

**选择**: `ImagePreviewer.render()` 内部使用 `new Image()` + `onload` 回调

**理由**: 直接写入 `<img>` 标签后，浏览器异步解码 blob URL，期间图片区域可能显示为空白或占位符。预加载确保图片解码完成后再写入 DOM。

**替代方案**: 使用 `img.decode()` Promise — 兼容性略差，且需要先创建 img 元素。

## Risks / Trade-offs

- **原子替换期间旧内容多停留一帧** — 用户可能短暂看到旧文件内容。Mitigation: 时间极短（<16ms），视觉上比空白帧好得多。
- **图片预加载增加显示延迟** — 必须等图片完全解码才显示。Mitigation: 图片通常很小（本地文件），延迟可忽略；大图可考虑渐进式显示（future work）。
- **Previewer 接口中 render 的返回类型** — 当前 `ImagePreviewer.render()` 是同步的，改为预加载后变为异步。需要检查所有调用点是否已用 `await`。Mitigation: `PreviewRouter.preview()` 已经用了 `await`，接口类型声明中 render 返回 `void | Promise<void>` 即可。
