## Why

切回已打开的 PDF tab 时，预览组件重新创建并清空瓦片，用户会看到从视口中心向外逐块重建的过程。首次打开 PDF 时，高清瓦片优先且逐块显示，也造成明显的拼图感。

## What Changes

- 为已打开的 PDF tab 保留当前视口的画面；切回时立即显示，并在确认文件变更或视口参数变化后按需更新。
- 首次渲染及缓存失效时，优先生成可见区域的低清瓦片，成批呈现完整视口，再逐步替换成高清瓦片。
- 明确 tab 关闭、文件替换、缩放、滚动及缓存预算下的资源清理和失效行为。

## Capabilities

### New Capabilities

- `pdf-progressive-presentation`: 定义低清视口的批量呈现和高清渐进替换行为。

### Modified Capabilities

- `per-tab-container`: 已打开 PDF 的 tab 切换也应复用其预览画面。
- `pdf-preview-resource-management`: 保留画面须遵守有界内存预算，且可见低清任务优先于高清及预取任务。

## Impact

涉及 `PreviewEditor.svelte`、`PdfPreviewPanel.svelte`、`pdf-shared.ts` 和 tab 生命周期管理。增加按路径失效后端 PDF 文档缓存的 Tauri 命令；无需新增依赖。
