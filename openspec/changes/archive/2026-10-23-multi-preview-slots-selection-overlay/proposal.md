## Why

当前工作区 tab 只能保留一个打开文件，目录选中预览又与该文件共用同一预览状态。在同一项目中查看多个文件时，用户需要反复重开文件，且选中预览可能干扰编辑状态。

## What Changes

- 每个工作区 tab 支持多个由用户管理的预览位，始终至少保留一个；可新建空位、切换、关闭。
- 目录选中项进入独立、只读的选中预览层，不修改任何预览位；预览层覆盖内容区域，顶部预览位标签栏保持可见。
- `l` 在当前预览位打开选中文件，`L` 在新预览位打开；点击标签切换预览位并收起选中预览层。
- 顶部标题栏改为预览位标签栏，保留文件名、修改状态、编辑模式和现有布局及大纲操作的可见入口。
- 为每个预览位独立保存编辑、滚动与 PDF 状态，并在关闭或替换有未保存修改的预览位时执行保存、放弃或取消流程。

## Capabilities

### New Capabilities

- `selection-preview-overlay`: 目录选中项的独立只读预览层及其显示、隐藏和异步加载行为。
- `workspace-preview-slots`: 工作区内多个手动预览位的创建、打开、切换、关闭、状态与快捷键。

### Modified Capabilities

- `editor-unsaved-prompt`: 目录选中预览不再触发编辑器文件切换；替换或关闭有未保存修改的预览位时继续保护修改。
- `per-tab-container`: 在现有顶层 tab 预览容器内支持多个预览位，并维持切换和清理时的资源隔离。

## Impact

- 前端状态与交互：`tabs.ts`、`PanelLayout.svelte`、`DirectoryPanel.svelte`、`PreviewEditor.svelte`、`PreviewPane.svelte`、`TextEditorHost.svelte`、键位与帮助说明。
- 预览资源：PDF、图片、视频与文件监听的生命周期和缓存键；需要保持现有资源预算与过期结果防护。
- 不涉及数据库结构、外部 API 或新增依赖。
