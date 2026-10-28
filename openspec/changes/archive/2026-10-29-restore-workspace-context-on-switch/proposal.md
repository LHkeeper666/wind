## Why

切换工作区 tab、预览槽或离开并返回 Wind 后，画面和焦点有时不能恢复到离开前的可操作状态：选中预览层会盖住已聚焦的预览槽，大纲焦点等细节也会丢失。用户需要切回时直接继续原来的阅读或编辑，不再手动点击预览槽。

## What Changes

- 为工作区和预览槽明确保存与恢复的状态：活动面板、面板内焦点目标、编辑模式、光标与选区、未保存内容、滚动位置，以及 PDF 页码和大纲状态。
- 工作区 tab、预览槽、应用窗口的切出与切入使用一致的恢复语义；保留从目录面板键盘切槽时目录焦点和选中预览层的既有行为。
- 当目标状态是聚焦预览槽时，显示该槽的内容并隐藏遮挡它的选中预览层；当目标状态是目录焦点时，按原状态保留选中预览层。
- 保留有效缓存时优先立即显示已有画面；缓存失效或资源已回收时安全重建并在完成后恢复位置，不承诺无条件零延迟。

## Capabilities

### New Capabilities

- `workspace-context-restoration`: 工作区、预览槽和应用焦点切换时的状态快照、恢复顺序及失效处理。

### Modified Capabilities

- `selection-preview-overlay`: 明确预览层与目标焦点的一致性，补充 tab 切换和应用回焦的场景。
- `workspace-preview-slots`: 扩展切槽恢复要求，覆盖槽内焦点、选区和大纲位置。

## Impact

- 前端状态与焦点协调：`src/lib/components/PanelLayout.svelte`、`PreviewEditor.svelte`、`TextEditorHost.svelte`、预览与大纲组件，以及 `src/lib/stores/tabs.ts`。
- 不涉及后端 API、数据库结构或新增依赖。
