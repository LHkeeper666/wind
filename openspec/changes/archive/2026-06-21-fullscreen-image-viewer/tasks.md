## 1. Layout Store 扩展

- [x] 1.1 在 `layout.ts` 的 `LayoutState` 接口中添加 `fullscreenImageViewerOpen: boolean` 字段
- [x] 1.2 在 `initialState` 中设置 `fullscreenImageViewerOpen: false`
- [x] 1.3 添加 `openFullscreenImageViewer()` 和 `closeFullscreenImageViewer()` 方法

## 2. DirectoryPanel 图片列表暴露

- [x] 2.1 在 `DirectoryPanel.svelte` 中添加 `IMAGE_EXTENSIONS` 常量
- [x] 2.2 添加 `export function getImageFiles(): FileEntry[]` 方法，过滤非目录且扩展名为图片的文件

## 3. FullscreenImageViewer 组件

- [x] 3.1 创建 `FullscreenImageViewer.svelte` 骨架：overlay + viewport + footer 结构
- [x] 3.2 实现 props：`imageList`、`currentIndex`、`onClose`、`onNavigate`
- [x] 3.3 实现图片加载逻辑：`$effect` 监听 `currentIndex` 变化，调用 `invoke('read_binary_file')` 加载原图为 blob URL
- [x] 3.4 实现 fit-to-screen 初始缩放：`Math.min(viewportW/imgW, viewportH/imgH, 1)`
- [x] 3.5 实现 `h/l` 缩放：`scale = Math.max(0.1, scale ± 0.25)`，`transform-origin: center center`
- [x] 3.6 实现 `Ctrl+j/k/h/l` 平移：`translateX/Y ±= 100px`
- [x] 3.7 实现 `j/k` 切换图片：循环增减 `currentIndex`，调用 `onNavigate`
- [x] 3.8 实现 `Esc/q` 关闭：调用 `onClose()`
- [x] 3.9 实现 footer 信息栏：文件名、原始尺寸、文件大小、位置 (current/total)
- [x] 3.10 处理 blob URL 生命周期：切换图片和关闭时 `revokeObjectURL`
- [x] 3.11 处理加载状态和错误状态

## 4. PanelLayout 集成

- [x] 4.1 import `FullscreenImageViewer` 组件
- [x] 4.2 添加状态变量：`fullscreenImageList`、`fullscreenImageIndex`
- [x] 4.3 添加 `isImageFile()` 工具函数
- [x] 4.4 修改 `handleFullscreenEditor()`：图片文件走 viewer，文本文件走 editor
- [x] 4.5 添加 `handleCloseImageViewer()` 和 `handleImageViewerNavigate()` 函数
- [x] 4.6 在模板中渲染 `{#if $layout.fullscreenImageViewerOpen}` 条件块
- [x] 4.7 在 `handleGlobalKeydown` 中拦截 viewer 打开时的 `:` 快捷键

## 5. PreviewEditor E 键分支

- [x] 5.1 修改 `PreviewEditor.svelte` 的 `handleKeydown` 中 `E` 键处理：图片文件走 `onFullscreen()`（不需要 toast 提示）

## 6. 验证

- [x] 6.1 运行 `npx svelte-check` 确认类型检查通过
- [x] 6.2 运行 `cargo check` 确认 Rust 侧无影响
