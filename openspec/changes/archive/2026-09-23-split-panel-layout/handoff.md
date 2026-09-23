# Handoff: split-panel-layout

## 变更概述

将 `PanelLayout.svelte`（3098 行）拆分为 5 个独立模块 + 1 个子组件，降低单文件复杂度，实现关注点分离。

## Artifacts 位置

| 文件 | 路径 |
|------|------|
| Proposal | `openspec/changes/split-panel-layout/proposal.md` |
| Design | `openspec/changes/split-panel-layout/design.md` |
| Tasks | `openspec/changes/split-panel-layout/tasks.md` |

## 职责分布分析

### PanelLayout.svelte 当前职责（3098 行）

| 职责 | 行数 | 占比 | 说明 |
|------|------|------|------|
| 键盘路由 | ~500 | 16% | `handleGlobalKeydown` 365 行 + 前缀状态 + 模式检查 |
| 命令面板 | ~700 | 23% | 命令定义 + Tab 补全 + `handleCommandKeydown` 359 行 + 模板 + 样式 |
| 剪贴板/粘贴 | ~400 | 13% | `handlePaste` 5 种路径 + `scanConflicts` + FTP 工具 |
| Tab 管理 | ~245 | 8% | 新建/关闭/切换 + 状态保存/恢复 + MRU 切换器 |
| 全屏查看器 | ~100 | 3% | 图片/PDF/视频/编辑器全屏状态 |
| 对话框状态 | ~120 | 4% | 压缩/冲突/未保存 4 种对话框 |
| 焦点管理 | ~80 | 3% | focusPanel + 窗口焦点恢复 |
| 导航 | ~80 | 3% | handleNavigate + 目录监听 |
| 拖拽分隔条 | ~35 | 1% | 列宽调整 |
| 模板 | ~361 | 12% | 三列布局 + 覆盖层 + 状态栏 |
| 样式 | ~287 | 9% | 所有 CSS |
| 其他 | ~190 | 6% | imports + state 声明 + onMount/onDestroy + 工具函数 |

### $state 变量清单（40+ 个）

```
导航: currentPath, selectedFile, selectedFileIsDir
命令面板: showCommandPalette, commandQuery, commandInput
文件搜索: showFileSearch, fileSearchHelpDir
帮助: showHelp
传输: showTransfer
缩放: zoomLevel
编辑器引用: previewEditor, fullscreenEditor, floatingTerminal
面板引用: parentDirectoryPanel, currentDirectoryPanel, previewPanel, recycleBinPanel
目录监听: refreshCoordinator, directoryWatchUnlisten, directoryWatchTimer
批量重命名: batchRenameTempPath, batchRenameFileEntries
压缩对话框: compressDialogVisible, compressDialogValue, compressMarkPaths
粘贴冲突: showConfirmModal, confirmFileName, pasteResolve
流式冲突: streamConflictVisible, streamConflictName, streamConflictResolve, scanningConflicts
未保存更改: showUnsavedConfirm, pendingActionPath, pendingAction
Ctrl+W 前缀: waitingForWindowKey, windowKeyTimeout
g 前缀: waitingForGKey, gKeyTimeout
Alt+Tab 切换器: altHeld, switcherActive, switcherSelectionId, switcherOriginTabId, switcherMruIds, switcherPhysicalIds
全屏查看器: fullscreenImageList/Index, fullscreenPdfPath/Page/PageCount/FileSize, fullscreenVideoPlayerPath/FileSize
焦点: windowReady, focusUnlisten, preFullscreenColumn, editorInitialLine
Toast: toastMessage, toastTimeout
回收站: recycleBinPreviewItem
拖拽: isDragging, dragStartX, dragStartRatios
Tab 补全: completions, completionIndex, completionPrefix, completionDir
```

## 架构总览

### 拆分后的组件/模块树

```
PanelLayout.svelte (~1200 行)
  ├── composables/keyboard-shortcuts.svelte.ts (~450 行)
  │     └── 前缀状态 + Alt+Tab 切换器 + handleGlobalKeydown/up/wheel + setup/teardown
  ├── composables/fullscreen-manager.svelte.ts (~100 行)
  │     └── 图片/PDF/视频/编辑器全屏状态 + 处理函数
  ├── composables/dialog-state.svelte.ts (~120 行)
  │     └── 压缩/冲突/未保存对话框状态 + prompt 函数 + 处理函数
  ├── utils/clipboard-operations.ts (~350 行)
  │     └── handlePaste + scanConflicts + FTP 工具函数
  ├── <CommandPalette> (~500 行)
  │     └── 命令定义 + Tab 补全 + 命令解析 + 模板 + 样式
  ├── <DirectoryPanel> (已有, 不变)
  ├── <PreviewEditor> (已有, 不变)
  ├── <FullscreenEditor> (已有, 不变)
  ├── <FullscreenImageViewer> (已有, 不变)
  ├── <FullscreenPdfViewer> (已有, 不变)
  ├── <FullscreenVideoPlayer> (已有, 不变)
  ├── <FloatingTerminal> (已有, 不变)
  ├── <TabBar> (已有, 不变)
  └── <TransferManager> (已有, 不变)
```

### PanelLayout 拆分后保留的职责

- 导航状态（`currentPath`、`selectedFile`、`leftPanelPath`）及处理函数
- 焦点管理（`focusPanel`/`focusPanelNow`、`handleAppFocusIn`、`handleWindowFocusChanged`）
- 目录监听 + 刷新协调（`refreshCoordinator`、`directoryWatchUnlisten`）
- Tab 切换核心逻辑（`handleTabSwitch`、`restoreTabContent`、`saveCurrentTabState`、`handleTabNew`、`handleTabClose`）
- 拖拽分隔条（`startResize`/`handleResize`/`stopResize`）
- Toast 通知
- `onMount`/`onDestroy` 生命周期
- 模板（三列布局 + 覆盖层 + 状态栏）+ 样式

## 关键接口设计

### CommandPalette.svelte Props

```typescript
interface CommandPaletteProps {
  visible: boolean;                    // $bindable
  currentPath: string;
  activeColumn: string;
  previewMode: string;
  onNavigate: (path: string) => void;
  onLeftNavigate: (path: string) => void;
  onSelect: (path: string) => void;
  onShowToast: (msg: string) => void;
  onToggleTerminal: () => void;
  onToggleTransfer: () => void;
  onToggleHelp: () => void;
  onToggleDetach: () => void;
  onToggleProjectTree: () => Promise<void>;
  onRefreshDirectory: () => void;
  onTogglePdfToc: () => void;
  onJumpToPdfPage: (page: number) => void;
  getPreviewEditor: () => any;
  getCurrentDirectoryPanel: () => any;
  resolvePath: (input: string) => string;
}
```

### keyboard-shortcuts.svelte.ts 返回值

```typescript
interface KeyboardShortcutsReturn {
  // Reactive state (for template binding)
  readonly waitingForWindowKey: boolean;
  readonly waitingForGKey: boolean;
  readonly altHeld: boolean;
  readonly switcherActive: boolean;
  readonly switcherSelectionId: number;
  readonly switcherOriginTabId: number;
  readonly switcherMruIds: number[];
  readonly switcherPhysicalIds: number[];
  // Operations
  startSwitcher: (mode: 'mru' | 'physical', direction?: 1 | -1) => void;
  commitSwitcher: () => void;
  // Lifecycle
  setup: () => void;
  teardown: () => void;
}
```

### clipboard-operations.ts 接口

```typescript
interface PasteDependencies {
  currentPath: string;
  getOperationDirectory: () => string;
  showToast: (msg: string) => void;
  refreshPanels: (paths: string[]) => Promise<void>;
  onOpenTransfer: () => void;
}

// 导出函数
handlePaste(deps: PasteDependencies, force?: boolean): Promise<void>
scanConflicts(startScan, promptConflict): Promise<ScanResult | null>
isFtpPath(p: string): boolean
getFtpConnName(p: string): string
getFtpRemotePath(p: string): string
getFtpDestPath(dirPath: string, name: string): string
```

## 实施顺序

1. **Phase 1**（状态模块，无模板改动）→ 最简单，可独立验证
   - Task 1.1: `fullscreen-manager.svelte.ts`
   - Task 1.2: `dialog-state.svelte.ts`
2. **Phase 2**（纯函数提取）→ 无 UI 依赖，可单元测试
   - Task 2.1: `clipboard-operations.ts`
3. **Phase 3**（组件提取）→ 涉及模板迁移
   - Task 3.1: `CommandPalette.svelte`
4. **Phase 4**（键盘处理）→ 最复杂，与 Tab 切换器交互
   - Task 4.1: `keyboard-shortcuts.svelte.ts`
5. **Phase 5**（验证）→ 全面测试
   - Task 5.1: 编译验证
   - Task 5.2: 功能验证
   - Task 5.3: 行数统计

## 受影响文件清单

| 操作 | 文件 |
|------|------|
| 新建 | `src/lib/components/CommandPalette.svelte` |
| 新建 | `src/lib/composables/keyboard-shortcuts.svelte.ts` |
| 新建 | `src/lib/utils/clipboard-operations.ts` |
| 新建 | `src/lib/composables/fullscreen-manager.svelte.ts` |
| 新建 | `src/lib/composables/dialog-state.svelte.ts` |
| 修改 | `src/lib/components/PanelLayout.svelte`（3098 → ~1200 行） |

## 预期效果

- PanelLayout.svelte: 3098 行 → ~1200 行（-61%）
- 最大单文件职责: 键盘路由 500 行 → 0（移入 composable）
- 命令面板: 内联 700 行 → 独立组件 500 行
- 剪贴板逻辑: 内联 400 行 → 独立模块 350 行
- $state 变量: 40+ → ~20（移出一半到 composables）

## 注意事项

1. **Svelte 5 runes 限制**: `$state`/`$derived`/`$effect` 只能在 `.svelte` 或 `.svelte.ts` 文件中使用。纯逻辑模块（`clipboard-operations.ts`）使用普通 TypeScript。
2. **Tab 切换器状态共享**: `switcherActive` 等状态被键盘处理和 `restoreTabContent` 共用。保留在 `keyboard-shortcuts.svelte.ts` 中，通过 getter 暴露。
3. **回调地狱风险**: `keyboard-shortcuts.svelte.ts` 需要 ~15 个回调。考虑使用对象接口而非独立参数。
4. **模板绑定**: `CommandPalette.svelte` 的 `visible` 需要用 `$bindable` 实现双向绑定。
5. **不要在此次拆分中改变任何功能行为**。纯重构，所有快捷键、命令、对话框行为保持不变。

## 下一步

运行 `/opsx:apply` 开始实施任务。