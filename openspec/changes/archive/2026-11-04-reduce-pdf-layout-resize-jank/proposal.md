## Why

2026-11-04 10:42 左右调整列宽时出现严重卡顿。日志显示短时间内密集的 PDF 瓦片渲染、重复布局响应和 ResizeObserver 循环错误；当前拖动事件逐次更新布局，PDF 预览随尺寸变化反复调度渲染。

## What Changes

- 合并列宽拖动事件，使每帧最多提交一次布局更新，结束拖动时保留最终列宽。
- PDF 预览在连续尺寸变化期间保持已有画面，尺寸稳定后再更新可见瓦片；忽略宽度未变化的观察回调。
- 拖动期间暂停 PDF 预取，优先处理稳定视口的可见瓦片，清除尚未开始的过期请求。
- 增加针对连续拖动、PDF 显示和渲染请求量的验证。

## Capabilities

### New Capabilities

- `panel-resize-responsiveness`: 面板列宽拖动的更新频率、最终宽度和响应性。

### Modified Capabilities

- `pdf-preview-resource-management`: 连续改变预览视口大小时的瓦片调度和预取策略。

## Impact

主要涉及 `PanelLayout.svelte`、`PdfPreviewPanel.svelte` 和 PDF 渲染调度相关工具。无外部 API、数据格式或依赖变更。
