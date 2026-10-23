## Why

当前预览槽栏同时承担文件切换、模式显示及槽位增删操作，视觉上比终端和 Neovim 的 tabline 更拥挤；模式信息也与底部状态栏分散。需要让预览槽标题更紧凑，并将预览模式统一放到固定的底部状态位置。

## What Changes

- 顶部各栏的数量和高度保持现状，不合并或移除窗口标题栏、工作区标签栏和预览槽栏。
- 在底部状态栏设置固定宽度的预览模式区域，准确显示 `PREVIEW`、`TOC`、`NORMAL`、`INSERT`，避免模式变化使其他信息移位。
- 从预览槽栏移除模式文字，以及每个预览槽的 `×` 按钮和末尾的 `+` 按钮；保留现有键盘新建、关闭和切换操作。
- 将预览槽标签改为与现有工作区标签栏一致的样式，并保留未保存修改标记。

## Capabilities

### New Capabilities

- `preview-mode-statusbar`: 底部状态栏在固定位置展示当前预览槽的模式，包括大纲焦点状态。

### Modified Capabilities

- `workspace-preview-slots`: 调整预览槽栏的可见信息及操作入口，保留标签切换、修改状态、适用的大纲和布局控制，以及现有键盘操作。

## Impact

主要涉及 `src/lib/components/PanelLayout.svelte` 的预览槽栏与底部状态栏，以及 `src/lib/components/PreviewEditor.svelte` 提供的模式信息。无需变更 Rust 后端、持久化格式或依赖。
