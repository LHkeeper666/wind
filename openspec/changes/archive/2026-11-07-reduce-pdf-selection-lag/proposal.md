## Why

在含多个 PDF 的目录中连续按 j/k 时，每次短暂选中都会立即启动 PDF 信息提取和页面预览。过期请求仍占用 PDFium 锁，页数多时还会创建大量页面节点，导致目录导航卡顿。

## What Changes

- 目录选中高亮和预览目标立即更新；对选中预览中的 PDF 加载作短暂合并，只加载停留的目标。
- 在异步边界前校验目标身份，避免为已过期的选中项发起 PDF 请求；已返回的过期结果不得挂载或继续预取。
- PDF 预览仅挂载视口附近的页面节点，使页面 DOM 数量不随文档总页数增长，同时保持完整滚动范围与页码定位。
- 增加针对连续选择、长 PDF 和请求耗时的验证与性能记录。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `selection-preview-overlay`: PDF 快速切换时合并加载并抑制过期工作，保持目录导航即时响应。
- `pdf-preview-resource-management`: 长 PDF 的页面节点数量有界，并保留正确滚动与定位行为。

## Impact

涉及 `DirectoryPanel.svelte`、`SelectionPreviewOverlay.svelte`、`PdfPreviewPanel.svelte`、PDF 加载状态及相关测试。现有 Tauri 命令接口不变，无新增依赖。
