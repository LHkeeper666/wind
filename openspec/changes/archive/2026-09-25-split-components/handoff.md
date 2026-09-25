# Handoff: split-components

## 变更概述

拆分 PreviewEditor.svelte (2344行) 和 DirectoryPanel.svelte (2152行) 两个大组件为更小的子组件。

## Artifacts 位置

| 文件 | 路径 |
|------|------|
| Proposal | `openspec/changes/split-components/proposal.md` |
| Design | `openspec/changes/split-components/design.md` |
| Specs | `openspec/changes/split-components/specs/*/spec.md` |
| Tasks | `openspec/changes/split-components/tasks.md` |

## 关键发现

1. **已有未使用的子组件**: PreviewEditor 已经 import 了 `TextEditorHost`、`VimOverlay`、`PreviewPane` 但未在模板中使用。这是之前重构的残留，本次拆分将完成这个迁移。

2. **Tab 缓存必须留在 PreviewEditor**: Tab 缓存需要访问所有子组件的状态（编辑器、预览、PDF），是 PreviewEditor 与 PanelLayout 之间的协调机制。子组件通过暴露 snapshot 方法支持缓存。

3. **handleKeydown 不拆到子组件**: DirectoryPanel 的 425 行 `handleKeydown` 涉及所有状态，拆到子组件会导致事件冒泡复杂化。改为内部函数拆分。

4. **项目树已有 ProjectTreePanel**: 项目树的渲染已在 `ProjectTreePanel.svelte` 中，只需将交互逻辑（展开/折叠/选择）迁入。

## 拆分架构

### PreviewEditor → 3 个子组件

```
PreviewEditor.svelte (~500 行)
├── Tab 缓存 + 文件加载/监听 + global-normal 按键
├── TextEditorHost.svelte (~400 行)
│   └── CodeMirror EditorSession 管理
├── VimOverlay.svelte (~350 行)
│   └── Vim 按键路由 + 命令行 + shell 输出
├── PreviewPane.svelte (~300 行)
│   └── 预览渲染 + TOC + 滚动
├── PdfPreviewPanel.svelte (已有)
├── PdfTocSidebar.svelte (已有)
└── TocSidebar.svelte (已有)
```

### DirectoryPanel → 2 个子组件 + 内部拆分

```
DirectoryPanel.svelte (~800 行)
├── 共享逻辑：加载、排序、过滤、搜索、对话框
├── FileListPanel.svelte (~200 行, 新建)
│   └── 普通文件列表渲染
├── ProjectTreePanel.svelte (已有, 扩展)
│   └── 项目树渲染 + 展开/折叠
└── handleKeydown → 5 个辅助函数（内部拆分）
```

## 子组件接口设计

### TextEditorHost

```typescript
// Props
{
  filePath: string | null,
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',
  content: string,
  savedContent: string,
  isModified: boolean,
  renderTabId: number,
  onEditorViewChange: (view: EditorView | undefined) => void,
  onClipboardBridgeChange: (bridge: ClipboardBridge | null) => void,
  onEditorFilePathChange: (path: string | null) => void,
  onModifiedChange: (modified: boolean) => void,
  onContentChange: (content: string) => void,
}

// 暴露方法
export function createSession(tabId: number, path: string, text: string): void;
export function activateSession(tabId: number, path: string): boolean;
export function destroySession(tabId: number): void;
export function hideAll(): void;
export function focus(mode: 'editor-normal' | 'editor-insert'): void;
export function getView(): EditorView | undefined;
```

### VimOverlay

```typescript
// Props
{
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',
  editorView: EditorView | undefined,
  clipboardBridge: ClipboardBridge | null,
  filePath: string | null,
  content: string,
  savedContent: string,
  isModified: boolean,
  batchRenameTempPath: string | null,
  onModeChange: (mode: ...) => void,
  onContentChange: (content: string) => void,
  onSavedContentChange: (content: string) => void,
  onModifiedChange: (modified: boolean) => void,
  onSaveFile: () => void,
  onToast: (msg: string) => void,
  onBatchRenameSave: (content: string) => void,
  onBatchRenameCancel: () => void,
}

// 暴露方法
export function handleKeydown(event: KeyboardEvent): boolean;
export function focus(): void;
```

### PreviewPane

```typescript
// Props
{
  filePath: string | null,
  content: string,
  binaryContent: ArrayBuffer | null,
  originalFileSize: number,
  thumbnailMeta: object | null,
  videoMeta: VideoMeta | null,
  isMarkdown: boolean,
  renderTabId: number,
  mode: 'global-normal' | 'editor-normal' | 'editor-insert',
  tocHeadings: TocHeading[],
  tocActiveLine: number,
  tocOpen: boolean,
  tocFocused: boolean,
  onTocJump: (line: number) => void,
  onTocFocusChange: (focused: boolean) => void,
}

// 暴露方法
export function render(): void;
export function scroll(deltaY: number, deltaX?: number): void;
export function getVisibleLine(): number;
export function focusToc(): void;
export function focusContent(): void;
export function isTocVisible(): boolean;
export function isTocFocused(): boolean;
```

### FileListPanel (新建)

```typescript
// Props
{
  files: FileEntry[],
  selectedIndex: number,
  cutPaths: Set<string>,
  type: 'parent' | 'current',
  selectionState: SelectionState,
  isArchiveMode: boolean,
  onSelect: (index: number) => void,
  onDblClick: (entry: FileEntry, index: number, event: MouseEvent) => void,
}
```

## 父子通信方式

| 通信方向 | 机制 | 示例 |
|----------|------|------|
| 父→子 | `$props()` | PreviewEditor 传递 `filePath` 给 VimOverlay |
| 子→父 | Callback | VimOverlay 调用 `onContentChange(content)` |
| 父调子方法 | `bind:this` + `export function` | PreviewEditor 调用 `textEditorHost.createSession()` |
| 子组件间 | **不直接通信**，通过父组件协调 | VimOverlay 不直接调用 TextEditorHost |

## 实施顺序

| 阶段 | 任务 | 依赖 | 预估行数变化 |
|------|------|------|-------------|
| 1.1 | TextEditorHost 迁移 | 无 | PE -350, TEH +350 |
| 1.2 | VimOverlay 迁移 | 1.1 | PE -400, VO +350 |
| 1.3 | PreviewPane 迁移 | 1.2 | PE -300, PP +300 |
| 1.4 | PreviewEditor 清理 | 1.1-1.3 | PE -200 |
| 2.1 | FileListPanel 提取 | 无 | DP -200, FLP +200 |
| 2.2 | ProjectTreePanel 扩展 | 2.1 | DP -100, PTP +100 |
| 2.3 | handleKeydown 拆分 | 2.1-2.2 | DP ±0 (内部重组) |
| 3.1 | 编译验证 | 全部 | - |
| 3.2 | 功能回归测试 | 3.1 | - |

最终目标：PreviewEditor ~500 行，DirectoryPanel ~800 行。

## 风险

1. **CodeMirror session 泄漏** — 迁移后必须检查所有 destroy 路径
2. **Vim 按键事件截断** — 在 PreviewEditor 层捕获 keydown，分派给 VimOverlay
3. **Tab 缓存不一致** — 保持缓存在 PreviewEditor 中，子组件暴露 snapshot
4. **文件监听丢失** — 保持 file-changed 监听在 PreviewEditor 中

## 下一步

运行 `/opsx:apply` 开始实施 Task 1.1（TextEditorHost 迁移）。