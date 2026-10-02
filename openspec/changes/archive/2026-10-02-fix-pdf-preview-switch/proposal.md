## Why

在包含多个 PDF 文件的文件夹中，用户选中不同 PDF 文件时，预览面板内容不更新。根本原因是 `PdfPreviewPanel.svelte` 的路径切换 `$effect` 只清理了组件内部状态（tileCache、renderedPages、renderedTileData），但没有清理 `scrollEl` 中已有的 tile canvas DOM 元素。由于新旧 PDF 的 pageCount 可能相同，Svelte 的 `{#each}` 不会重建 page-slot，旧 tile canvas 留在 DOM 中，新 tile 追加在旁边，导致视觉上预览不变。

## What Changes

- 在 `PdfPreviewPanel.svelte` 的路径切换 `$effect` 中，当 `pdfPath` 变化时，增加 `scrollEl.innerHTML = ''` 来清理 DOM 中的旧 tile canvas 元素
- 确保新 PDF 渲染时从干净的 DOM 状态开始，避免旧内容残留

## Capabilities

### New Capabilities

_(无新能力，纯 bug 修复)_

### Modified Capabilities

_(无需求变更，纯实现修复)_

## Impact

- 影响文件：`src/lib/components/PdfPreviewPanel.svelte`
- 影响范围：PDF 预览面板的路径切换逻辑
- 无 API 变更，无依赖变更，无 breaking change