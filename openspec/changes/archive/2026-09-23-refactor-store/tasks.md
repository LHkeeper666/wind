# Tasks: refactor-store

## Task 1: 添加 FullscreenViewer 类型和新方法

**文件**: `src/lib/stores/layout.ts`

- [x] 添加 `FullscreenViewer` 类型定义
- [x] 在 `LayoutState` 中添加 `fullscreenViewer: FullscreenViewer` 字段
- [x] 在 `initialState` 中设置 `fullscreenViewer: 'none'`
- [x] 添加 `setFullscreenViewer(viewer: FullscreenViewer)` 方法
- [x] 添加 `closeFullscreenViewer()` 方法（等同于 `setFullscreenViewer('none')`）
- [x] 添加 derived store `isFullscreen`

## Task 2: 更新 fullscreen-manager.svelte.ts

**文件**: `src/lib/composables/fullscreen-manager.svelte.ts`

- [x] 将 `layout.openFullscreenImageViewer()` 替换为 `layout.setFullscreenViewer('image')`
- [x] 将 `layout.openFullscreenPdfViewer()` 替换为 `layout.setFullscreenViewer('pdf')`
- [x] 将 `layout.openFullscreenVideoPlayer()` 替换为 `layout.setFullscreenViewer('video')`
- [x] 将 `layout.openFullscreenEditor()` 替换为 `layout.setFullscreenViewer('editor')`
- [x] 将 `layout.closeFullscreenEditor()` 替换为 `layout.closeFullscreenViewer()`
- [x] 将 `layout.closeFullscreenImageViewer()` 替换为 `layout.closeFullscreenViewer()`
- [x] 将 `layout.closeFullscreenPdfViewer()` 替换为 `layout.closeFullscreenViewer()`
- [x] 将 `layout.closeFullscreenVideoPlayer()` 替换为 `layout.closeFullscreenViewer()`

## Task 3: 更新 PanelLayout.svelte 模板

**文件**: `src/lib/components/PanelLayout.svelte`

- [x] 将 `{#if $layout.fullscreenEditorOpen}` 替换为 `{#if $layout.fullscreenViewer === 'editor'}`
- [x] 将 `{#if $layout.fullscreenImageViewerOpen}` 替换为 `{#if $layout.fullscreenViewer === 'image'}`
- [x] 将 `{#if $layout.fullscreenPdfViewerOpen}` 替换为 `{#if $layout.fullscreenViewer === 'pdf'}`
- [x] 将 `{#if $layout.fullscreenVideoPlayerOpen}` 替换为 `{#if $layout.fullscreenViewer === 'video'}`
- [x] 更新 import，移除旧的 derived store（如 `isFullscreenEditor`）

## Task 4: 删除旧的布尔字段和方法

**文件**: `src/lib/stores/layout.ts`

- [x] 从 `LayoutState` 中删除 4 个布尔字段（保留 `fullscreenTerminalOpen`）
- [x] 从 `initialState` 中删除对应的初始值
- [x] 删除 8 个旧方法：
  - `toggleFullscreenEditor`
  - `openFullscreenEditor` / `closeFullscreenEditor`
  - `openFullscreenImageViewer` / `closeFullscreenImageViewer`
  - `openFullscreenPdfViewer` / `closeFullscreenPdfViewer`
  - `openFullscreenVideoPlayer` / `closeFullscreenVideoPlayer`
- [x] 更新 `isFullscreenEditor` derived store 使用新字段

## Task 5: 验证

- [x] 运行 `npx svelte-check` 确认无类型错误
- [ ] 运行 `npm run tauri dev` 验证功能正常（需手动测试）
- [ ] 测试所有全屏查看器（需手动测试）：
  - 图片全屏查看
  - PDF 全屏查看
  - 视频全屏播放
  - 编辑器全屏
  - 终端全屏（Ctrl+`）