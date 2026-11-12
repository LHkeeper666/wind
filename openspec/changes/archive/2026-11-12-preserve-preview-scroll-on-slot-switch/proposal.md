## Why

在 Markdown TOC 获得焦点时往返切换预览槽，正文可能回到顶部。2026-11-12 19:18 的日志显示：目标槽先以来源槽的旧内容启动渲染，随后缓存恢复又触发正确内容的渲染；第一次渲染消费了待恢复的滚动位置，第二次替换内容后无法再恢复。

## What Changes

- 切槽期间只允许与目标槽 ID、文件路径和已就绪内容一致的快照进入预览渲染。
- 缓存命中时保留目标槽已有的预览 DOM 和阅读位置；确需重新渲染时，在最终有效渲染后恢复阅读位置。
- 覆盖 TOC 聚焦、快速往返切槽、缓存命中及缓存失效的回归场景。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `workspace-preview-slots`: 明确从 Markdown 大纲切槽后，正文滚动位置和大纲焦点均须恢复，快速往返也不例外。
- `atomic-preview-swap`: 明确切槽时不得以来源槽的内容渲染目标槽，且过期渲染不得消费目标槽的滚动恢复状态。

## Impact

涉及 `PreviewEditor.svelte` 的槽切换加载与缓存恢复、`PreviewPane.svelte` 的自动渲染与滚动恢复，以及相关预览槽状态测试。不涉及后端接口或新增依赖。
