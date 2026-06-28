## Why

当前图片只能在预览区以缩略图方式查看，无法查看原图细节。用户需要一个全屏查看器来浏览原图，并支持在同目录图片间快速切换、缩放和平移。

## What Changes

- 新增 `FullscreenImageViewer.svelte` 全屏图片查看器组件
- 复用 `E` 快捷键：当目标文件是图片时，打开全屏查看器而非文本编辑器
- 支持 `j/k` 在当前目录的纯图片列表中循环切换
- 支持 `h/l` 缩小/放大图片（默认 fit-to-screen，最小 0.1）
- 支持 `Ctrl+j/k/h/l` 平移图片焦点
- `Esc/q` 关闭查看器，恢复之前焦点
- 新增 layout store 状态 `fullscreenImageViewerOpen`
- `DirectoryPanel` 新增 `getImageFiles()` 方法暴露图片列表
- `PanelLayout` 中 `E` 键路由逻辑改为：图片 → ImageViewer，文本 → Editor

## Capabilities

### New Capabilities
- `fullscreen-image-viewer`: 全屏图片查看器，支持缩放、平移、图片切换

### Modified Capabilities
(无现有 spec 需要修改)

## Impact

- 新增文件：`src/lib/components/FullscreenImageViewer.svelte`
- 修改文件：`src/lib/stores/layout.ts`、`src/lib/components/PanelLayout.svelte`、`src/lib/components/DirectoryPanel.svelte`、`src/lib/components/PreviewEditor.svelte`
- 无新依赖，复用现有 Tauri `read_binary_file` 命令
