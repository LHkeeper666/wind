# Design: 拆分 PreviewEditor 和 DirectoryPanel

## 一、PreviewEditor 拆分方案

### 现状分析

PreviewEditor 已有 3 个未使用的子组件 import：
- `TextEditorHost.svelte` — 已有 CodeMirror 初始化逻辑
- `VimOverlay.svelte` — 已有 Vim 按键路由和命令行处理
- `PreviewPane.svelte` — 已有预览渲染和 TOC 管理

拆分策略：**完成已有迁移**，将内联逻辑迁移到这些已有组件中。

### 目标架构

```
PreviewEditor.svelte (~500 行)
├── 状态协调 + Tab 缓存 + 文件加载/监听
├── TextEditorHost.svelte (~400 行)
│   └── CodeMirror EditorSession 管理（创建/销毁/激活/布局稳定性）
├── VimOverlay.svelte (~350 行)
│   └── Vim 按键路由 + 命令行 + 搜索 + shell 输出
├── PreviewPane.svelte (~300 行)
│   └── 预览渲染 + TOC 侧边栏 + 滚动管理
├── PdfPreviewPanel.svelte (已有)
├── PdfTocSidebar.svelte (已有)
└── TocSidebar.svelte (已有)
```

### 各子组件职责边界

#### TextEditorHost
**职责**: CodeMirror EditorView 的完整生命周期管理

从 PreviewEditor 迁入：
- `EditorSession` 接口和 `editorSessions` Map
- `destroyEditorSession` / `activateEditorSession` / `hideEditorSessions`
- `initEditor` (CodeMirror 配置、Vim、主题、ResizeObserver、剪贴板桥接)
- `nextAnimationFrame` / `isVisibleBox` / `clampEditorScroll`
- `hasStableEditorLayout` / `waitForStableEditorLayout` / `refreshEditorLayoutAfterPaint`
- `isActiveEditorSession` / `getSessionForView`
- `focusActiveEditor`

Props 接口：
```typescript
let {
  // 输入
  filePath: string | null,
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',
  content: string,
  savedContent: string,
  isModified: boolean,
  renderTabId: number,

  // 输出回调
  onEditorViewChange: (view: EditorView | undefined) => void,
  onClipboardBridgeChange: (bridge: ClipboardBridge | null) => void,
  onEditorFilePathChange: (path: string | null) => void,
} = $props();

// 暴露给父组件的方法
export function createSession(tabId: number, path: string, text: string): void;
export function activateSession(tabId: number, path: string): boolean;
export function destroySession(tabId: number): void;
export function hideAll(): void;
export function focus(): void;
export function getView(): EditorView | undefined;
```

#### VimOverlay
**职责**: editor-normal 模式下的所有按键处理和命令行交互

从 PreviewEditor 迁入：
- `codeToVimKey` — 键盘事件→Vim 按键转换
- `handleOverlayKeydown` — editor-normal 模式按键路由
- `processOverlayCommand` — 命令行处理 (`:w`/`:q`/`:wq`/`:reg`/`:!`)
- `triggerFileCompletion` / `resetCompletion` — Tab 补全
- `executeSearch` / `highlightSMatches` — 搜索和替换高亮
- `executeShellCommand` / `closeOutputPanel` / `handleOutputKeydown` — Shell 输出
- overlay 状态变量 (`overlayCmdBuf`, `overlayCmdActive`, `searchActive`, `searchBuf`, `outputVisible`, `outputText`, `outputExitCode`)

Props 接口：
```typescript
let {
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',
  editorView: EditorView | undefined,
  clipboardBridge: ClipboardBridge | null,
  filePath: string | null,
  content: string,
  savedContent: string,
  isModified: boolean,
  batchRenameTempPath: string | null,

  // 模式/内容变更回调
  onModeChange: (mode: ...) => void,
  onContentChange: (content: string) => void,
  onSavedContentChange: (content: string) => void,
  onModifiedChange: (modified: boolean) => void,
  onSaveFile: () => void,
  onToast: (msg: string) => void,
  onBatchRenameSave: (content: string) => void,
  onBatchRenameCancel: () => void,
} = $props();

// 暴露给父组件的方法
export function handleKeydown(event: KeyboardEvent): boolean;  // 返回是否已处理
export function focus(): void;
```

#### PreviewPane
**职责**: 非编辑模式下的预览渲染（文本/图片/视频/目录/压缩包/Markdown+TOC）

从 PreviewEditor 迁入：
- `getPreviewRouter` / `getDirectoryPreviewer` — 懒初始化
- `renderPreview` / `renderPreviewOnce` — 串行化渲染队列
- `renderSimpleCodePreview` / `renderDirectoryPreview` / `renderArchivePreview`
- `setupScrollObserver` — Markdown TOC 联动
- `handleTocJump` / `handleTocFocusChange`
- `scrollPreview` / `getVisibleLine`

Props 接口：
```typescript
let {
  filePath: string | null,
  content: string,
  binaryContent: ArrayBuffer | null,
  originalFileSize: number,
  thumbnailMeta: object | null,
  videoMeta: VideoMeta | null,
  isMarkdown: boolean,
  renderTabId: number,
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',

  // TOC 状态（双向）
  tocHeadings: TocHeading[],
  tocActiveLine: number,
  tocOpen: boolean,
  tocFocused: boolean,

  // 回调
  onTocJump: (line: number) => void,
  onTocFocusChange: (focused: boolean) => void,
} = $props();

// 暴露给父组件的方法
export function render(): void;
export function scroll(deltaY: number, deltaX?: number): void;
export function getVisibleLine(): number;
export function focusToc(): void;
export function focusContent(): void;
export function isTocVisible(): boolean;
export function isTocFocused(): boolean;
```

### Tab 缓存保留在 PreviewEditor

Tab 缓存 (`TabCacheManager`, `cacheTabState`, `clearTabCache`, `requestTabRender`) **不迁移到子组件**，因为：
1. Tab 缓存需要访问所有子组件的状态（编辑器、预览、PDF）
2. 它是 PreviewEditor 与 PanelLayout 之间的协调机制
3. 子组件通过暴露 snapshot 方法来支持缓存

### 通信模式

```
PanelLayout
  └── PreviewEditor (协调者 + Tab 缓存)
        ├── TextEditorHost  ← 通过 ref 调用 createSession/activateSession
        ├── VimOverlay      ← 通过 ref 调用 handleKeydown/focus
        └── PreviewPane     ← 通过 ref 调用 render/scroll
```

- 父→子: Props 传递 + `bind:this` 调用方法
- 子→父: Callback 回调
- 子组件不直接通信，所有协调逻辑在 PreviewEditor 中

---

## 二、DirectoryPanel 拆分方案

### 现状分析

DirectoryPanel 通过 `projectMode` 在模板中切换两种完全不同的渲染模式：
- 普通模式：`{#each displayFiles}` 文件列表
- 项目树模式：`<ProjectTreePanel>` 子组件

但两种模式共享大量逻辑（键盘处理、选择、排序、归档操作等）。

### 目标架构

```
DirectoryPanel.svelte (~800 行)
├── 共享逻辑：加载、排序、过滤、键盘、搜索、对话框、归档操作
├── FileListPanel.svelte (~200 行)
│   └── 普通文件列表渲染 + 点击/双击
├── ProjectTreePanel.svelte (已有，需扩展)
│   └── 项目树渲染 + 展开/折叠 + 树节点选择
└── ArchiveToolbar.svelte (~80 行)
    └── 归档模式专用工具栏（路径、提取、删除）
```

### 各子组件职责边界

#### FileListPanel (新建)
**职责**: 普通模式下的文件列表渲染

从 DirectoryPanel 迁入：
- 文件列表的 `{#each}` 渲染逻辑
- 文件项的 class 绑定（selected/multi-selected/cut-marked/directory/hidden）
- cut-marker 和 file-name 的渲染
- `handleItemClick` / `handleItemDblClick` 中普通模式的逻辑

Props 接口：
```typescript
let {
  files: FileEntry[],
  selectedIndex: number,
  cutPaths: Set<string>,
  showHidden: boolean,
  type: 'parent' | 'current',
  selectionState: SelectionState,
  isArchiveMode: boolean,

  onSelect: (index: number) => void,
  onDblClick: (entry: FileEntry, index: number) => void,
} = $props();
```

#### ProjectTreePanel (已有，扩展)
**职责**: 项目树模式下的渲染和交互

当前已有的渲染逻辑保持不变。从 DirectoryPanel 迁入：
- `toggleTreeNode` / `collapseTreeNode`
- `ensureSelectionVisibleAfterCollapse`
- `restoreSelectedTreeNodeFocusAfterRender`
- `isTreeToggleHit`
- `isTreeNodeSelected` 判断逻辑

需要增加的 props：
```typescript
// 已有 props 保持不变
// 新增：
isTreeNodeSelected: (node: TreeNode) => boolean,
onCollapse: (node: TreeNode) => void,
```

#### ArchiveToolbar (新建，可选)
**职责**: 归档模式下的专用 UI

从 DirectoryPanel 模板迁入：
- 归档路径和内部路径的显示
- 归档操作按钮（提取、删除、重命名）

> 注：此组件较简单（~80 行），可作为第二阶段优化。

### 键盘处理保留在 DirectoryPanel

`handleKeydown`（425 行）**不拆分到子组件**，因为：
1. 键盘快捷键需要访问所有状态（选择、排序、归档、项目树）
2. 普通模式和项目树模式的快捷键有大量重叠
3. 拆分会导致键盘事件冒泡处理复杂化

但可以在 DirectoryPanel 内部将 `handleKeydown` 拆分为多个辅助函数：
- `handleNavigationKey` — j/k/gg/G/h/l/Enter
- `handleOperationKey` — d/D/y/x/p/r 等文件操作
- `handleSortFilterKey` — s/S/o 等排序过滤
- `handleTreeKey` — 项目树专用按键

### 通信模式

```
PanelLayout
  └── DirectoryPanel (协调者 + 键盘处理)
        ├── FileListPanel    ← props: files, selectedIndex, cutPaths
        ├── ProjectTreePanel ← props: visibleNodes, expanded state
        └── ArchiveToolbar   ← props: archiveState (可选)
```

---

## 三、迁移顺序

### Phase 1: PreviewEditor 拆分（风险较高，先做）

1. **TextEditorHost 迁移** — 将 `initEditor` 和 session 管理迁入
2. **VimOverlay 迁移** — 将按键处理和命令行迁入
3. **PreviewPane 迁移** — 将预览渲染和 TOC 迁入
4. **清理 PreviewEditor** — 移除内联代码，改为调用子组件

### Phase 2: DirectoryPanel 拆分（风险较低）

5. **FileListPanel 提取** — 将文件列表渲染提取为新组件
6. **ProjectTreePanel 扩展** — 迁入树交互逻辑
7. **handleKeydown 拆分** — 内部函数拆分（不涉及组件边界）

### Phase 3: 验证

8. **svelte-check** — 类型检查
9. **dev server** — 手动验证所有功能
10. **回归测试** — Tab 切换、文件预览、编辑、保存、PDF、压缩包

---

## 四、风险和缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| CodeMirror session 生命周期在迁移后出现泄漏 | 高 | 迁移后检查所有 EditorSession 的 destroy 路径 |
| VimOverlay 的按键事件被子组件边界截断 | 高 | 使用 `onkeydown` 在 PreviewEditor 层捕获，分派给 VimOverlay |
| Tab 缓存在子组件边界不一致 | 高 | 保持 Tab 缓存在 PreviewEditor 中，子组件暴露 snapshot 方法 |
| ProjectTreePanel 扩展后与 DirectoryPanel 状态不同步 | 中 | 使用 $effect 同步关键状态 |
| 文件监听事件在组件重新挂载后丢失 | 中 | 保持 file-changed 监听在 PreviewEditor 层 |