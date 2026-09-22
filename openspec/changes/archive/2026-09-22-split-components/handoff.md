# Handoff: split-components

## 变更概述

将 PreviewEditor.svelte（2481 行）和 DirectoryPanel.svelte（2238 行）拆分为更小的子组件和工具模块，降低单文件复杂度。

## Artifacts 位置

| 文件 | 路径 |
|------|------|
| Proposal | `openspec/changes/split-components/proposal.md` |
| Design | `openspec/changes/split-components/design.md` |
| Specs | `openspec/changes/split-components/specs/*/spec.md` |
| Tasks | `openspec/changes/split-components/tasks.md` |

## 架构总览

### PreviewEditor 拆分后的组件树

```
PanelLayout
  └── PreviewEditor (父组件, ~800 行)
        ├── VimOverlay (子组件)
        │     └── overlayElement + 命令行 + 搜索 + shell 输出
        ├── TextEditorHost (子组件)
        │     └── CodeMirror EditorView + EditorSession + 主题
        └── PreviewPane (子组件)
              ├── tabSlots (预览 DOM 容器)
              ├── TocSidebar (已有子组件)
              └── PreviewRouter (已有工具类)
```

### DirectoryPanel 拆分后的组件树

```
PanelLayout
  └── DirectoryPanel (父组件, ~1200 行)
        ├── ProjectTreePanel (子组件, projectMode=true 时)
        │     └── TreeNode 渲染 + 缩进线 + toggle
        ├── [普通文件列表] (projectMode=false 时, 保留在父组件)
        ├── SearchModal (已有子组件)
        ├── InputDialog (已有子组件)
        ├── ConfirmModal (已有子组件)
        └── FileInfoPanel (已有子组件)
```

### 工具模块

```
src/lib/utils/
  ├── tab-cache.ts          — TabEditorCache Map + 渲染版本管理
  ├── file-loader.ts        — 文件类型分发加载逻辑
  ├── selection-manager.ts  — 多选状态纯函数
  └── archive-browser.ts    — 压缩包操作函数
```

## 关键接口设计

### PreviewEditor 父组件 → 子组件通信

```typescript
// PreviewEditor 父组件持有
let tabCache = new TabCacheManager();
let tabSlots = new Map<number, HTMLDivElement>();
let editorSessions = new Map<number, EditorSession>();

// 向 PreviewPane 传递
<PreviewPane
  filePath={filePath}
  content={content}
  binaryContent={binaryContent}
  tabId={renderTabId}
  mode={mode}
  isMarkdown={isMarkdown}
  tocHeadings={tocHeadings}
  tocOpen={tocOpen}
  onTocHeadingsChange={(h) => { tocHeadings = h; }}
  onTocActiveLineChange={(l) => { tocActiveLine = l; }}
  onTocFocusChange={(f) => { tocFocused = f; }}
/>

// 向 TextEditorHost 传递
<TextEditorHost
  container={editorContainer}
  tabId={renderTabId}
  filePath={filePath}
  content={content}
  savedContent={savedContent}
  mode={mode === 'editor-normal' || mode === 'editor-insert' ? mode : 'editor-normal'}
  isActive={mode === 'editor-normal' || mode === 'editor-insert'}
  onContentChange={(c) => { content = c; }}
  onModeChange={(m) => { mode = m; }}
  onModifiedChange={(m) => { isModified = m; }}
  onSave={() => saveFile()}
  onQuit={() => { if (!codeFileDirectEdit) mode = 'global-normal'; }}
/>

// 向 VimOverlay 传递
<VimOverlay
  mode={mode}
  editorView={editorView}
  clipboardBridge={clipboardBridge}
  activeColumn={activeColumn}
  filePath={filePath}
  content={content}
  onModeChange={(m) => { mode = m; }}
  onContentChange={(c) => { content = c; }}
  onSave={() => saveFile()}
  onToast={onToast}
/>
```

### DirectoryPanel 父组件 → 子组件通信

```typescript
// 使用 selection-manager 纯函数
import { createSelectionState, togglePathSelection, isTreeNodeSelected, ... } from '$lib/utils/selection-manager';

let selection = $state<SelectionState>(createSelectionState());

// 向 ProjectTreePanel 传递
<ProjectTreePanel
  root={projectRoot}
  visibleNodes={treeVisibleNodes}
  selectedIndex={selectedIndex}
  selectedFile={selectedFile}
  showHidden={showHidden}
  panelType={type}
  onSelect={(index) => selectByIndex(index)}
  onActivate={(path) => onActivate(path)}
  onExpand={(node) => void expandTreeNode(node)}
  onCollapse={(node) => collapseTreeNode(node)}
  onToggleSelection={(node) => { selection = toggleTreeSelection(selection, node); }}
/>

// 使用 archive-browser 函数
import { enterArchive, handleArchiveUp, archiveExtract, ... } from '$lib/utils/archive-browser';

// 键盘处理中调用
case 'KeyL':
  if (isArchiveMode && archiveState) {
    enterArchive(entry.path, layout);
  }
```

## Tab 缓存一致性保障

拆分后 Tab 缓存机制的核心流程保持不变：

1. **缓存收集**：`PreviewEditor.cacheTabState(tabId)` 调用各子组件的方法收集状态
   - `previewPane.getActiveSlot().scrollTop` → previewScrollTop
   - `textEditorHost.getSession()?.view.state.selection.main.head` → editorCursorPos
   - `tocSidebar.getSelectedIndex()` → tocSelectedIndex

2. **缓存恢复**：`loadFile()` 检测到缓存命中时
   - 从 `tabCache.get(tabId)` 恢复所有状态
   - 传递给各子组件的 props 触发恢复

3. **缓存失效**：文件修改或手动刷新时
   - `tabCache.delete(tabId)` + 清除对应 slot 的 `dataset.rendered`
   - 子组件收到新 props 后重新渲染

## 实施顺序

1. Phase 1（工具模块）→ 可独立验证，不影响模板
2. Phase 2（DirectoryPanel 子组件）→ 先处理较简单的组件
3. Phase 3（PreviewEditor 子组件）→ 最复杂的部分
4. Phase 4（验证）→ 全面测试

## 受影响文件清单

| 操作 | 文件 |
|------|------|
| 新建 | `src/lib/utils/tab-cache.ts` |
| 新建 | `src/lib/utils/file-loader.ts` |
| 新建 | `src/lib/utils/selection-manager.ts` |
| 新建 | `src/lib/utils/archive-browser.ts` |
| 新建 | `src/lib/components/TextEditorHost.svelte` |
| 新建 | `src/lib/components/VimOverlay.svelte` |
| 新建 | `src/lib/components/PreviewPane.svelte` |
| 新建 | `src/lib/components/ProjectTreePanel.svelte` |
| 修改 | `src/lib/components/PreviewEditor.svelte` |
| 修改 | `src/lib/components/DirectoryPanel.svelte` |

## 下一步

运行 `/opsx:apply` 开始实施任务。