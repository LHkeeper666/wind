## Context

Wind 项目前端两个核心组件的行数远超合理范围：

| 组件 | 行数 | 函数数 | 职责数 |
|------|------|--------|--------|
| PreviewEditor.svelte | 2481 | ~86 | 8 |
| DirectoryPanel.svelte | 2238 | ~70 | 7 |

### PreviewEditor 职责分析

| 职责 | 行数范围 | 核心状态 | 核心函数 |
|------|----------|----------|----------|
| Tab 缓存 | 382-494, 410-460 | `tabEditorCache`, `tabSlots`, `editorSessions` | `cacheTabState`, `clearTabCache`, `getOrCreateSlot`, `showTabSlot` |
| CodeMirror 编辑器 | 205-314, 1346-1485 | `editorView`, `editorSessions`, `EditorSession` | `initEditor`, `destroyEditorSession`, `activateEditorSession`, `focusActiveEditor` |
| Vim overlay 键处理 | 1487-1742 | `overlayCmdBuf`, `overlayCmdActive`, `searchActive`, `clipboardBridge` | `handleOverlayKeydown`, `processOverlayCommand`, `codeToVimKey`, `executeSearch` |
| 文件加载 | 847-1169 | `content`, `binaryContent`, `readyTextContent`, `loadGeneration` | `loadFile`（300 行分发函数） |
| 预览渲染 | 709-720, 1183-1268, 1326-1344 | `previewRouter`, `renderInFlight`, `renderQueued` | `renderPreview`, `renderPreviewOnce`, `renderSimpleCodePreview` |
| TOC/PDF 状态 | 342-380, 836-845 | `tocHeadings`, `pdfOutline`, `pdfTocOpen` | `setupScrollObserver`, `handleTocJump`, `focusToc` |
| 模式管理 | 628-692 | `mode` (global-normal/editor-normal/editor-insert) | `$effect` 中的模式切换逻辑 |
| Shell 输出 | 1526-1638 | `outputVisible`, `outputText`, `outputExitCode` | `executeShellCommand`, `closeOutputPanel` |

### DirectoryPanel 职责分析

| 职责 | 行数范围 | 核心状态 | 核心函数 |
|------|----------|----------|----------|
| 文件列表 | 95-200 | `files`, `displayFiles`, `normalDisplayFiles` | `loadDirectory`, `selectInitialEntry` |
| 项目树 | 128-130, 372-398, 633-750 | `projectMode`, `projectRoot`, `TreeNode` | `setProjectMode`, `loadTreeChildren`, `expandTreeNode`, `refreshProjectTree` |
| 压缩包浏览 | 1340-1458 | `archiveState`, `isArchiveMode` | `enterArchive`, `handleArchiveUp`, `handleArchiveExtract`, `handleArchiveDelete` |
| 多选状态 | 110-113, 1281-1338 | `selectedPaths`, `selectedTreeRoots`, `deselectedTreePaths` | `togglePathSelection`, `selectTreeSubtree`, `deselectTreeSubtree`, `isTreeNodeSelected` |
| 排序/过滤 | 118-160 | `sortBy`, `sortReverse`, `dirFirst`, `filterPattern`, `filterMode` | `setSort`, `toggleDirFirst`, `startFilter`, `clearFilter` |
| 键盘处理 | 1459-1846 | `lastKey`, `lastKeyTime`, `sortPrefixPending` | `handleKeydown`（380 行 switch） |
| 文件操作 | 837-1088 | `inputVisible`, `inputValue`, `inputMode`, `showDeleteConfirm` | `startRename`, `handleInputConfirm`, `handleDelete`, `handleCompress` |

## Goals / Non-Goals

**Goals:**
- 将 PreviewEditor 从 ~2500 行拆分到 ~800 行（父组件只保留协调逻辑）
- 将 DirectoryPanel 从 ~2238 行拆分到 ~1200 行
- 保持所有现有功能行为不变（纯重构）
- 保持 Tab 缓存机制的一致性（不破坏 tab 切换性能）
- 保持 PanelLayout 对这两个组件的调用接口不变

**Non-Goals:**
- 不改变 PreviewRouter/TextPreviewer 等 previewer 架构
- 不改变 vim-commands.ts 等已有工具模块
- 不重构 PanelLayout 的整体布局逻辑
- 不改变任何键盘快捷键的行为

## Decisions

### 1. PreviewEditor：提取子组件 vs 提取工具模块

**决策**：混合策略 — 有 DOM 模板的部分提取为 Svelte 子组件，纯逻辑部分提取为 `.ts` 工具模块。

**理由**：
- TextEditorHost 需要管理 editorContainer DOM 元素和 CodeMirror 实例 → 子组件
- VimOverlay 需要 overlayElement DOM 和键盘事件处理 → 子组件
- PreviewPane 需要 previewArea DOM 和预览 slot 管理 → 子组件
- file-loader.ts 是纯函数分发逻辑 → 工具模块
- tab-cache.ts 是纯数据结构管理 → 工具模块

**替代方案**：全部提取为工具模块 → 不可行，因为 CodeMirror 需要 DOM 容器引用

### 2. Tab 缓存保留在父组件

**决策**：`tabEditorCache`、`tabSlots`、`editorSessions` 三个 Map 保留在 PreviewEditor 父组件中。

**理由**：
- Tab 缓存是跨子组件的共享状态（TextEditorHost 的 session、PreviewPane 的 slot、文件加载的 content 都需要读写）
- 如果放到子组件中，需要通过 props/callbacks 传递大量状态，增加耦合
- 缓存的 `cacheTabState()` 需要从所有子组件收集状态，放在父组件中协调最自然

**实现**：
- 父组件持有 `tabEditorCache` Map 和 `tabSlots` Map
- 通过 `$effect` 将当前 tab 的缓存数据传递给子组件
- 子组件通过 `onUpdate` 回调通知父组件更新缓存

### 3. DirectoryPanel：ProjectTreePanel 作为子组件

**决策**：项目树模式提取为 `ProjectTreePanel` 子组件，与普通文件列表模式在 DirectoryPanel 中条件渲染。

**理由**：
- 项目树模式有独立的数据结构（TreeNode）、独立的渲染逻辑（缩进+toggle）、独立的键盘行为（h/l 展开折叠）
- 普通文件列表和项目树共享的只有 `selectByIndex`、排序、`onSelect` 回调
- 条件渲染 `{#if projectMode} <ProjectTreePanel> {:else} <FileList> {/if}` 是自然的拆分边界

**替代方案**：保持单一组件，仅提取工具函数 → 不能解决模板过长的问题

### 4. selection-manager 提取为工具模块

**决策**：多选状态管理（`selectedPaths`、`selectedTreeRoots`、`deselectedTreePaths` 及 10+ 个操作函数）提取为 `selection-manager.ts`。

**理由**：
- 多选逻辑是纯数据操作，不依赖 DOM
- 普通模式和项目树模式都使用多选，提取后两者都能复用
- 约 200 行逻辑代码，提取后 DirectoryPanel 显著瘦身

### 5. ArchiveBrowser 提取为工具模块

**决策**：压缩包浏览逻辑提取为 `archive-browser.ts`，提供 `ArchiveBrowserState` 类和操作方法。

**理由**：
- 压缩包浏览有独立的状态（`archiveState`）和独立的操作（进入/退出/导航/解压/删除/重命名）
- 约 120 行逻辑代码
- 键盘处理中的压缩包分支（h/l/Enter/D/E/e/X 等）仍然留在 DirectoryPanel 的 handleKeydown 中，但调用 archive-browser 的方法

### 6. 子组件通信方式

**决策**：使用 Svelte 5 的 `$props()` + callbacks 模式。

**接口设计原则**：
- 数据向下：通过 `$props()` 传递只读数据
- 事件向上：通过 callback props 通知父组件
- 子组件不直接修改父组件状态
- 子组件可以有内部状态（`$state`），但关键状态提升到父组件

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| Tab 缓存一致性：子组件状态更新不同步 | 父组件通过 $effect 统一收集子组件状态更新 |
| 性能回退：额外的组件层级增加渲染开销 | Svelte 5 编译时优化，子组件不会触发不必要的重渲染 |
| 拆分后代码导航：需要跳转多个文件理解完整逻辑 | 每个子组件/模块有清晰的职责命名，handoff 文档记录依赖关系 |
| 大范围重构引入 bug | 逐个提取，每步运行 svelte-check + 手动验证 |