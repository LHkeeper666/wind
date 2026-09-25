## 1. 提取 `createPdfState` composable

- [x] 1.1 创建 `src/lib/composables/pdf-state.svelte.ts`，包含 10 个 PDF `$state` 变量（pdfPageCount、pdfCurrentPage、pdfFileSize、pdfTitle、pdfPageDimensions、pdfOutline、pdfTocOpen、pdfTocFocused、pdfTocSidebar、pdfPreviewPanel）
- [x] 1.2 实现 `PdfStateAPI` 接口：getter/setter、`togglePdfToc()`、`jumpToPdfPage()`、`focusPanel()`、`focusToc()`、`reset()`
- [x] 1.3 实现 `loadPdfInfo(path, gen)` — 从 `loadFile` PDF 分支提取（`invoke('get_pdf_info')` + `fetchPdfOutline`）
- [x] 1.4 实现 `toCacheSnapshot()` / `fromCacheSnapshot()` — 对接 `TabEditorCache` 的 PDF 字段
- [x] 1.5 在 PreviewEditor 中实例化 `createPdfState`，移除 10 个 PDF `$state` 声明
- [x] 1.6 更新 `loadFile` PDF 分支调用 `pdfState.loadPdfInfo(path, gen)`
- [x] 1.7 更新模板 PDF 区域使用 `pdfState` getter 和 `bind:this={pdfState.pdfPreviewPanel}`
- [x] 1.8 更新 `cacheTabState` / 缓存命中路径使用 `pdfState.toCacheSnapshot()` / `fromCacheSnapshot()`
- [x] 1.9 更新导出函数 `getPdfInfo`、`togglePdfToc`、`jumpToPdfPage`、`isTocVisible`、`focusToc`、`focusContent` 委托到 composable

## 2. 提取 `createMarkdownTocState` composable

- [x] 2.1 创建 `src/lib/composables/markdown-toc-state.svelte.ts`，包含 4 个 TOC `$state` 变量（tocHeadings、tocActiveLine、tocFocused、tocOpen）
- [x] 2.2 实现 `MarkdownTocAPI` 接口：getter/setter、`handleTocFocusChange()`、`onHeadingsChange()`、`onActiveLineChange()`、`reset()`
- [x] 2.3 实现 `toCacheSnapshot(getTocSelectedIndex)` / `fromCacheSnapshot()` — 对接 `TabEditorCache` 的 TOC 字段
- [x] 2.4 在 PreviewEditor 中实例化 `createMarkdownTocState`，移除 4 个 TOC `$state` 声明
- [x] 2.5 更新模板传递 `tocState` getter 到 PreviewPane 的 props
- [x] 2.6 更新 `cacheTabState` / 缓存命中路径使用 composable 快照方法
- [x] 2.7 更新 `isTocVisible`、`focusToc`、`focusContent` 委托到 composable

## 3. 提取 `createFileWatcher` composable

- [x] 3.1 创建 `src/lib/composables/file-watcher.svelte.ts`，包含 `currentFileMtime` 和 `fileChangedUnlisten`
- [x] 3.2 实现 `startWatching(path)`、`stopWatching()` — 调用 Tauri `invoke('start_watch_file')` / `invoke('stop_watch_file')`
- [x] 3.3 实现 `setup()` — 注册 `listen('file-changed', ...)` 事件，实现 `handleFileChanged` 逻辑
- [x] 3.4 实现 `teardown()` — 清理事件监听
- [x] 3.5 在 PreviewEditor 中实例化 `createFileWatcher`，移除 `currentFileMtime`、`fileChangedUnlisten`、`startWatching`、`stopWatching`、`handleFileChanged`
- [x] 3.6 更新 `loadFile` 中所有 `startWatching` / `stopWatching` / `currentFileMtime` 引用
- [x] 3.7 更新 `onDestroy` 清理逻辑调用 `fileWatcher.teardown()`
- [x] 3.8 将 `listen('file-changed', ...)` 从顶层移到 `fileWatcher.setup()`

## 4. 拆分 `loadFile` 为 per-type 子加载器

- [x] 4.1 创建 `src/lib/utils/file-loaders.ts`，定义 `LoadContext` 和 `LoadResult` 接口
- [x] 4.2 提取 `loadArchiveDirectory(ctx)` — 读取压缩包目录条目，返回 HTML 渲染结果
- [x] 4.3 提取 `loadArchiveFile(ctx)` — 读取压缩包文件字节，检测二进制/文本
- [x] 4.4 提取 `loadDirectory(ctx)` — 调用 `invoke('read_directory')`
- [x] 4.5 提取 `loadImage(ctx)` — 读取图片（含缩略图和 GIF 支持）
- [x] 4.6 提取 `loadPdf(ctx)` — 调用 `get_pdf_info` + `fetchPdfOutline`（委托给 `pdfState.loadPdfInfo`）
- [x] 4.7 提取 `loadArchive(ctx)` — 渲染压缩包预览
- [x] 4.8 提取 `loadVideo(ctx)` — 调用 `get_video_thumbnail`
- [x] 4.9 提取 `loadTextOrBinary(ctx)` — 读取文本/二进制（含 partial 和 null-byte 检测）
- [x] 4.10 实现 `applyLoadResult(result, gen, loadTabId, path)` — 将 `LoadResult` 写入 `$state`（内联到 loadFile 编排器）
- [x] 4.11 实现 `applyCacheResult(cached, gen, loadTabId, path)` — 缓存命中路径（保留原有逻辑）
- [x] 4.12 实现 `resetForNewLoad()` — 新加载前重置 `$state`（内联到 loadFile 编排器）
- [x] 4.13 重构 `loadFile` 为编排器：缓存检查 → reset → 路由到子加载器 → applyLoadResult

## 5. 清理与验证

- [x] 5.1 复核所有导出函数是否正确委托到 composable
- [x] 5.2 移除因提取而产生的死代码路径（invokeArchiveWithOptionalPassword、formatSize 未使用导入）
- [x] 5.3 运行 `npx svelte-check` 确认无类型错误（0 errors, 92 warnings — 全部为已有警告）
- [x] 5.4 运行 `cargo check` 确认 Rust 后端无影响（通过，2 个已有 warning）
- [x] 5.5 修复 PDF 滚动恢复：保存实际 scrollTop 到缓存（而非 pdfCurrentPage），用 double-rAF 等待 initScale 后恢复
- [x] 5.6 修复 MD 滚动恢复：以退出编辑模式时编辑器的 scrollRatio 为准（而非进入时的预览位置），通过 `pendingRestoreScrollRatio` prop 传给 PreviewPane 在异步渲染完成后恢复
- [x] 5.7 清理 debug console.log（PreviewPane、PdfPreviewPanel）
- [x] 5.8 运行 `npm run tauri dev` 桌面验证：PDF 预览/TOC/全屏、Markdown TOC 侧边栏、文件监听自动刷新、Tab 切换缓存恢复、各种文件类型加载、编辑模式切换