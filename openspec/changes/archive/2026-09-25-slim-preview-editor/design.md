## Context

PreviewEditor.svelte 是 Wind 文件管理器的预览/编辑面板，当前 1518 行、37 个 `$state`、41 个函数。前一轮拆分已提取 TextEditorHost（528 行）、VimOverlay（465 行）、PreviewPane（519 行）三个子组件。剩余职责混合了 PDF 预览状态（10 变量）、Markdown TOC 状态（4 变量）、文件监听（Tauri 事件）、Tab 缓存序列化和 `loadFile` 函数（~350 行，处理 7 种文件类型）。

项目已有 composable 模式：`createDirectorySortFilter`、`createProjectTree`、`createDirectoryDialogs` 使用工厂函数 + DI + `$state` in closure 模式。本次拆分沿用该模式。

## Goals / Non-Goals

**Goals:**
- 将 PreviewEditor 从 1518 行缩减到 ~700 行。
- 提取 PDF 状态为独立 composable，使 PDF 逻辑可独立理解和修改。
- 提取 Markdown TOC 状态为独立 composable，与 PreviewPane 的 TOC 渲染解耦。
- 将文件监听逻辑封装为 composable，与预览逻辑解耦。
- 将 `loadFile` 拆分为 per-type 子加载器，使文件类型路由清晰可测试。
- 不改变任何用户可见行为。

**Non-Goals:**
- 不提取 mode `$effect`（编排层，依赖 15+ 变量，提取无收益）。
- 不提取 keyboard handler（~30 行，中央路由）。
- 不提取 saveFile（~30 行）。
- 不提取 CSS（Svelte scoped CSS 有防泄漏价值）。
- 不改变 TabCacheManager 类本身。
- 不涉及 Rust 后端改动。

## Decisions

### 1. PDF 状态提取为 `createPdfState`

PreviewEditor 中 PDF 相关的 10 个 `$state` 变量自成一体：`pdfPageCount`、`pdfCurrentPage`、`pdfFileSize`、`pdfTitle`、`pdfPageDimensions`、`pdfOutline`、`pdfTocOpen`、`pdfTocFocused`、`pdfTocSidebar`（ref）、`pdfPreviewPanel`（ref）。

composable 负责：
- 状态管理（getter/setter）
- `togglePdfToc()`、`jumpToPdfPage()`、`focusPanel()`、`focusToc()`
- `loadPdfInfo(path, gen)` — 从 `loadFile` PDF 分支提取
- Tab 缓存集成：`toCacheSnapshot()` / `fromCacheSnapshot()`

DI 依赖：`getFilePath()`、`onToast()`。

`bind:this` 在 Svelte 5 中可用于 composable 属性——`pdfState.pdfPreviewPanel` 作为 ref 绑定到 `PdfPreviewPanel` 组件。

### 2. Markdown TOC 提取为 `createMarkdownTocState`

4 个变量（`tocHeadings`、`tocActiveLine`、`tocFocused`、`tocOpen`）与 PreviewPane 的 TocSidebar 交互。composable 负责：
- 状态管理
- `handleTocFocusChange(focused)` — 从 PreviewEditor 移入
- `onHeadingsChange(headings)` / `onActiveLineChange(line)` — PreviewPane 回调
- Tab 缓存集成

DI 依赖：`getPanelElement()`（用于 `tocFocused=false` 时 refocus 面板）。

### 3. 文件监听提取为 `createFileWatcher`

`startWatching`、`stopWatching`、`handleFileChanged` 和 `listen('file-changed', ...)` 事件订阅构成独立关注点。

composable 负责：
- `startWatching(path)` / `stopWatching()` — 调用 Tauri invoke
- `setup()` — 注册 `listen('file-changed', ...)` 事件
- `teardown()` — 清理事件监听
- `currentFileMtime` 状态

DI 依赖：`getFilePath()`、`getMode()`、`getRenderTabId()`、`onFileChanged(path)` 回调。

### 4. `loadFile` 拆分为 per-type 子加载器

当前 `loadFile` 350 行中有 7 条文件类型路径，每条路径异步读取数据并写入不同 `$state` 变量。拆分方案：

1. 定义 `LoadContext`（gen、loadTabId、filePath、archiveState、selectedEntryIsDir）和 `LoadResult`（content、binaryContent、readyTextContent、mode、isDirectory + 可选 pdfInfo/thumbnailMeta/videoMeta/originalFileSize/savedContent/currentFileMtime/editorInitInFlight/archiveEditPath）
2. 每种文件类型一个纯异步函数：`loadArchiveDirectory`、`loadArchiveFile`、`loadDirectory`、`loadImage`、`loadPdf`、`loadArchive`、`loadVideo`、`loadTextOrBinary`
3. 函数返回 `LoadResult`，不直接修改 `$state`
4. PreviewEditor 的 `loadFile` 变为编排器：缓存检查 → reset → 路由 → `applyLoadResult()`
5. `applyLoadResult()` 负责将结果写入 `$state` 变量

关键：每个子加载器在每次 `await` 后检查 `gen !== ctx.gen`，如果 generation 变化则返回 `{ aborted: true }` 哨兵值。编排器收到后直接 return。

### 5. 不提取 mode effect

mode `$effect`（~70 行）编排编辑器会话激活、滚动同步和焦点管理，依赖 `mode`、`editorView`、`textEditorHost`、`vimOverlay`、`previewPane`、`readyTextContent`、`editorTargetLine`、`activeColumn`、`codeFileDirectEdit`、`panelElement`、`tocFocused`、`renderTabId`。提取为 composable 需传递 ~15 个依赖——收益递减。保留为 PreviewEditor 的编排层。

### 6. 不提取 CSS

600+ 行预览专用 CSS（Markdown、ipynb、JSON、image、PDF）在 `<style>` 块中。Svelte scoped CSS 有防泄漏价值。CSS 提取不影响逻辑复杂度。保留内联。

## Risks / Trade-offs

- **`bind:this` 与 composable 属性** → Svelte 5 的 `$state` rune 在 composable closure 中与模板 `bind:this` 兼容。如果遇到问题，可改用 callback ref 模式（`onRef={el => pdfPreviewPanel = el}`）。
- **Tab 缓存快照完整性** → `toCacheSnapshot()` 必须覆盖 `TabEditorCache` 接口的所有字段。通过交叉检查 `tab-cache.ts` 接口确保完整。
- **generation check 在子加载器中** → 每个 loader 在每次 await 后检查 gen。遗漏会导致过期数据写入 `$state`。编排器中的 `applyLoadResult` 也做最终检查。
- **loadFile 重入** → `loadGeneration` 计数器已处理此问题。编排器中保留该机制。