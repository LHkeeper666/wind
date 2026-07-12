## Why

当前 tab 切换时，preview DOM 通过 removeChild → Map 缓存 → appendChild 的方式在"文档 attached"和"缓存 detached"之间反复移动。每次切回有大量 DOM 节点的 tab（如大 md 文件），浏览器需要重新 layout 整棵 DOM 树（数万个元素），耗时 50-200ms，造成可感知的切换延迟。

同时，频繁的 DOM 移动导致图片 blob URL 在重新挂载时需要重新解码，偶尔出现图片显示异常。

## What Changes

将 DOM 缓存策略从"detach + Map 存储"改为"per-tab 常驻容器 + display 切换"。每个 tab 的 preview DOM 始终 attached 在文档中（隐藏在 `display:none` 的容器里），tab 切换只需切换 `display` 属性，不触发浏览器 layout。

- PreviewEditor 为每个 tabId 维护独立的预览容器（per-tab slot）
- 所有 slot 始终存在于 DOM 中，仅活跃 tab 的 slot 可见
- 移除 `previewDomCache`（不再需要将 DOM 移入 Map 缓存）
- 简化 `cacheTabState`（不再处理 DOM 移动）
- 修复 `renderPreview` 中 stale DOM 导致的缓存恢复失败（line 936 early return bug）
- 避免 `{#if filePath}` 导致的 container 重建（改用 CSS 显隐）

## Capabilities

### New Capabilities
- `per-tab-container`: 每个 tab 维护独立的预览 DOM 容器，切换 tab 仅切换 display 属性，不移动 DOM 节点

### Modified Capabilities
- `tab-state-persistence`: DOM 缓存机制从 removeChild+Map 改为常驻 DOM+display 切换

## Impact

- `src/lib/components/PreviewEditor.svelte` — 核心改动：per-tab slot 管理、移除 previewDomCache、修复 stale DOM bug、避免 container 重建
- `src/lib/components/PanelLayout.svelte` — 可能需配合调整（如 focusPanel 中的 previewContainer 引用）
