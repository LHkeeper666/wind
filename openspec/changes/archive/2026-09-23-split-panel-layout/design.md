## 架构总览

### 拆分后的依赖关系

```
PanelLayout.svelte (~1200 行)
  ├── imports keyboard-shortcuts.svelte.ts
  │     └── 返回 reactive state + setup()/teardown()
  ├── imports fullscreen-manager.svelte.ts
  │     └── 返回 state + handler 函数
  ├── imports dialog-state.svelte.ts
  │     └── 返回 state + handler 函数
  ├── imports clipboard-operations.ts
  │     └── 纯函数，通过参数注入依赖
  ├── <CommandPalette> 子组件
  │     └── 命令定义 + Tab 补全 + 命令解析 + 模板
  ├── <DirectoryPanel> (已有)
  ├── <PreviewEditor> (已有)
  ├── <FullscreenEditor> (已有)
  ├── <FullscreenImageViewer> (已有)
  ├── <FullscreenPdfViewer> (已有)
  ├── <FullscreenVideoPlayer> (已有)
  ├── <FloatingTerminal> (已有)
  ├── <TabBar> (已有)
  └── <TransferManager> (已有)
```

### PanelLayout 拆分后保留的职责

- 导航状态（`currentPath`、`selectedFile`、`leftPanelPath`）
- 焦点管理（`focusPanel`/`focusPanelNow`、`handleAppFocusIn`、`handleWindowFocusChanged`）
- 目录监听 + 刷新协调（`refreshCoordinator`、`directoryWatchUnlisten`）
- Tab 切换核心逻辑（`handleTabSwitch`、`restoreTabContent`、`saveCurrentTabState`）
- 拖拽分隔条（`startResize`/`handleResize`/`stopResize`）
- 模板 + 样式
- `onMount`/`onDestroy` 生命周期
- Toast 通知

## 关键接口设计

### 1. CommandPalette.svelte

```svelte
<!-- CommandPalette.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { layout } from '$lib/stores/layout';
  import { tabs } from '$lib/stores/tabs';
  import { clipboard } from '$lib/stores/clipboard';

  let {
    visible = $bindable(false),
    currentPath,
    activeColumn,
    previewMode,
    onNavigate,
    onLeftNavigate,
    onSelect,
    onShowToast,
    onToggleTerminal,
    onToggleTransfer,
    onToggleHelp,
    onToggleDetach,
    onToggleProjectTree,
    onRefreshDirectory,
    onTogglePdfToc,
    onJumpToPdfPage,
    getPreviewEditor,
    getCurrentDirectoryPanel,
    resolvePath,
  }: {
    visible: boolean;
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
  } = $props();

  // Tab completion state
  let completions = $state<{ name: string; is_dir: boolean }[]>([]);
  let completionIndex = $state(-1);
  let completionPrefix = $state('');
  let completionDir = $state('');
  let commandQuery = $state('');
  let commandInput = $state<HTMLInputElement>();

  // Commands array
  const commands = [
    { name: 'Open File', action: () => layout.setActiveColumn('current') },
    { name: 'Open Terminal', action: () => onToggleTerminal() },
    // ... existing commands
  ];

  let filteredCommands = $derived(/* ... */);

  // Tab completion
  async function triggerCompletion() { /* moved from PanelLayout */ }
  function resetCompletion() { /* moved from PanelLayout */ }

  // Command execution
  function handleCommandKeydown(event: KeyboardEvent) { /* moved from PanelLayout */ }
  function executeCommand(cmd: any) { /* moved from PanelLayout */ }
</script>

{#if visible}
  <div class="command-palette-overlay" onclick={() => { visible = false; }}>
    <div class="command-palette" onclick={(e) => e.stopPropagation()}>
      <input bind:value={commandQuery} bind:this={commandInput} onkeydown={handleCommandKeydown} />
      <div class="command-list">
        {#each filteredCommands as cmd}
          <div class="command-item" onclick={() => executeCommand(cmd)}>{cmd.name}</div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  /* moved from PanelLayout command palette styles */
</style>
```

### 2. keyboard-shortcuts.svelte.ts

```typescript
// src/lib/composables/keyboard-shortcuts.svelte.ts
import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';
import { tabs } from '$lib/stores/tabs';

export interface KeyboardShortcutCallbacks {
  onTabNew: () => void;
  onTabClose: () => void;
  onTabSwitchByIndex: (index: number) => void;
  onStartSwitcher: (mode: 'mru' | 'physical', direction?: 1 | -1) => void;
  onPaste: (force?: boolean) => void;
  onToggleTerminal: () => void;
  onToggleFullscreenTerminal: () => void;
  onToggleTransfer: () => void;
  onToggleHelp: () => void;
  onToggleProjectTree: () => Promise<void>;
  onTogglePreviewLayout: () => void;
  onSwitchPanel: (dir: 'left' | 'right') => void;
  onOpenCommandPalette: () => void;
  onOpenFileSearch: () => Promise<void>;
  onApplyZoom: (level: number) => void;
  onRecycleBinToggle: () => void;
  onToggleDetach: () => void;
  focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void;
  showToast: (msg: string) => void;
  getPreviewEditor: () => any;
  getFullscreenEditor: () => any;
  getFloatingTerminal: () => any;
}

export function useKeyboardShortcuts(callbacks: KeyboardShortcutCallbacks) {
  // Ctrl+W prefix state
  let waitingForWindowKey = $state(false);
  let windowKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  // g prefix state
  let waitingForGKey = $state(false);
  let gKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  // Alt+Tab switcher state
  let altHeld = $state(false);
  let switcherActive = $state(false);
  let switcherSelectionId = $state(-1);
  let switcherOriginTabId = $state(-1);
  let switcherMruIds = $state<number[]>([]);
  let switcherPhysicalIds = $state<number[]>([]);

  const supportedAltTabCodes = new Set([
    'KeyN', 'KeyM', 'KeyU', 'KeyR', 'KeyH', 'KeyL', 'Comma', 'Period', 'KeyD',
    'Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5', 'Digit6', 'Digit7', 'Digit8', 'Digit9',
  ]);

  function isSupportedAltTabCode(code: string): boolean {
    return supportedAltTabCodes.has(code);
  }

  function isTerminalInputTarget(target: EventTarget | null): boolean {
    return target instanceof Element && target.closest('.terminal-containers') !== null;
  }

  // Mode check helpers
  function getCanOpenCommandPalette(): boolean { /* ... */ }
  function getCanUseTabShortcuts(): boolean { /* ... */ }
  function getCanUseGlobalFileOperations(): boolean { /* ... */ }

  // Switcher operations
  function startSwitcher(mode: 'mru' | 'physical', direction: 1 | -1 = 1) { /* ... */ }
  function moveSwitcherIn(ids: number[], direction: 1 | -1) { /* ... */ }
  function commitSwitcher() { /* ... */ }

  // Main keydown handler
  async function handleGlobalKeydown(event: KeyboardEvent) { /* ... */ }
  function handleGlobalKeyup(event: KeyboardEvent) { /* ... */ }
  function handleGlobalWheel(event: WheelEvent) { /* ... */ }

  function setup() {
    window.addEventListener('keydown', handleGlobalKeydown, true);
    window.addEventListener('keyup', handleGlobalKeyup, true);
    window.addEventListener('wheel', handleGlobalWheel, { passive: false, capture: true });
  }

  function teardown() {
    window.removeEventListener('keydown', handleGlobalKeydown, true);
    window.removeEventListener('keyup', handleGlobalKeyup, true);
    window.removeEventListener('wheel', handleGlobalWheel, { capture: true } as any);
  }

  return {
    // Reactive state (for template binding)
    get waitingForWindowKey() { return waitingForWindowKey; },
    get waitingForGKey() { return waitingForGKey; },
    get altHeld() { return altHeld; },
    get switcherActive() { return switcherActive; },
    get switcherSelectionId() { return switcherSelectionId; },
    get switcherOriginTabId() { return switcherOriginTabId; },
    get switcherMruIds() { return switcherMruIds; },
    get switcherPhysicalIds() { return switcherPhysicalIds; },
    // Operations
    startSwitcher,
    commitSwitcher,
    // Lifecycle
    setup,
    teardown,
  };
}
```

### 3. clipboard-operations.ts

```typescript
// src/lib/utils/clipboard-operations.ts
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';
import { clipboard } from '$lib/stores/clipboard';
import { transfer } from '$lib/stores/transfer';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';

export interface PasteDependencies {
  currentPath: string;
  getOperationDirectory: () => string;
  showToast: (msg: string) => void;
  refreshPanels: (paths: string[]) => Promise<void>;
  onOpenTransfer: () => void;
}

export function isFtpPath(p: string): boolean { return p.startsWith('ftp://'); }
export function getFtpConnName(p: string): string { /* ... */ }
export function getFtpRemotePath(p: string): string { /* ... */ }
export function getFtpDestPath(dirPath: string, name: string): string { /* ... */ }

export async function handlePaste(deps: PasteDependencies, force: boolean = false): Promise<void> {
  /* moved from PanelLayout - handles all 5 paste paths */
}

export async function scanConflicts(
  startScan: () => Promise<unknown>,
  promptConflict: (name: string) => Promise<'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort'>
): Promise<{ dirSkipMap: Map<string, string[]>; fileSkipSet: Set<string> } | null> {
  /* moved from PanelLayout */
}
```

### 4. fullscreen-manager.svelte.ts

```typescript
// src/lib/composables/fullscreen-manager.svelte.ts
import { isImageFile, isPdfFile, isVideoFile } from '$lib/utils/file-types';
import { layout } from '$lib/stores/layout';

export function useFullscreenManager() {
  // Fullscreen image viewer state
  let fullscreenImageList = $state<{ name: string; path: string }[]>([]);
  let fullscreenImageIndex = $state(0);

  // Fullscreen PDF viewer state
  let fullscreenPdfPath = $state('');
  let fullscreenPdfPage = $state(0);
  let fullscreenPdfPageCount = $state(0);
  let fullscreenPdfFileSize = $state(0);

  // Fullscreen video player state
  let fullscreenVideoPlayerPath = $state('');
  let fullscreenVideoPlayerFileSize = $state(0);

  // Track active column before fullscreen for restoration
  let preFullscreenColumn = $state<'parent' | 'current' | 'preview' | 'terminal' | null>(null);

  // Editor initial line
  let editorInitialLine = $state(0);

  function handleFullscreenEditor(selectedFile: string | null, currentDirectoryPanel: any, previewEditor: any) { /* ... */ }
  function handleCloseFullscreen(focusPanel: (p: string) => void) { /* ... */ }
  function handleSaveFullscreen(content: string, previewEditor: any) { /* ... */ }
  function handleCloseImageViewer(focusPanel: (p: string) => void) { /* ... */ }
  function handleClosePdfViewer(focusPanel: (p: string) => void) { /* ... */ }
  function handleCloseVideoPlayer(focusPanel: (p: string) => void) { /* ... */ }
  function handleImageViewerNavigate(index: number) { /* ... */ }

  return {
    // State
    get fullscreenImageList() { return fullscreenImageList; },
    get fullscreenImageIndex() { return fullscreenImageIndex; },
    get fullscreenPdfPath() { return fullscreenPdfPath; },
    get fullscreenPdfPage() { return fullscreenPdfPage; },
    get fullscreenPdfPageCount() { return fullscreenPdfPageCount; },
    get fullscreenPdfFileSize() { return fullscreenPdfFileSize; },
    get fullscreenVideoPlayerPath() { return fullscreenVideoPlayerPath; },
    get fullscreenVideoPlayerFileSize() { return fullscreenVideoPlayerFileSize; },
    get preFullscreenColumn() { return preFullscreenColumn; },
    get editorInitialLine() { return editorInitialLine; },
    set editorInitialLine(v: number) { editorInitialLine = v; },
    // Handlers
    handleFullscreenEditor,
    handleCloseFullscreen,
    handleSaveFullscreen,
    handleCloseImageViewer,
    handleClosePdfViewer,
    handleCloseVideoPlayer,
    handleImageViewerNavigate,
  };
}
```

### 5. dialog-state.svelte.ts

```typescript
// src/lib/composables/dialog-state.svelte.ts
import { invoke } from '@tauri-apps/api/core';
import { layout } from '$lib/stores/layout';

export function useDialogState() {
  // Compress dialog
  let compressDialogVisible = $state(false);
  let compressDialogValue = $state('');
  let compressMarkPaths = $state<string[]>([]);

  // Paste conflict
  let showConfirmModal = $state(false);
  let confirmFileName = $state('');
  let pasteResolve = $state<((choice: 'overwrite' | 'skip' | 'abort') => void) | null>(null);

  // Streaming conflict
  let streamConflictVisible = $state(false);
  let streamConflictName = $state('');
  let streamConflictResolve = $state<((choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') => void) | null>(null);
  let scanningConflicts = $state(false);

  // Unsaved changes
  let showUnsavedConfirm = $state(false);
  let pendingActionPath = $state('');
  let pendingAction = $state<'navigate' | 'activate'>('activate');

  // Prompt functions
  function promptConflict(fileName: string): Promise<'overwrite' | 'skip' | 'abort'> { /* ... */ }
  function promptConflictStream(fileName: string): Promise<'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort'> { /* ... */ }
  function resolveStreamConflict(choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') { /* ... */ }

  // Confirm handlers
  function handleConfirmOverwrite() { /* ... */ }
  function handleConfirmSkip() { /* ... */ }
  function handleConfirmAbort() { /* ... */ }

  // Unsaved handlers
  function handleUnsavedSave(/* deps */) { /* ... */ }
  function handleUnsavedDiscard(/* deps */) { /* ... */ }
  function handleUnsavedCancel(focusPanel: (p: string) => void) { /* ... */ }

  // Compress handlers
  function handleCompressDialogConfirm(value: string, currentPath: string, showToast: (m: string) => void, refresh: () => void) { /* ... */ }
  function handleCompressDialogCancel() { /* ... */ }

  return {
    // State
    compressDialogVisible, compressDialogValue, compressMarkPaths,
    showConfirmModal, confirmFileName,
    streamConflictVisible, streamConflictName, scanningConflicts,
    showUnsavedConfirm, pendingActionPath, pendingAction,
    // Prompt functions
    promptConflict, promptConflictStream, resolveStreamConflict,
    // Handlers
    handleConfirmOverwrite, handleConfirmSkip, handleConfirmAbort,
    handleUnsavedSave, handleUnsavedDiscard, handleUnsavedCancel,
    handleCompressDialogConfirm, handleCompressDialogCancel,
  };
}
```

## 模板拆分细节

### 从 PanelLayout 模板移入 CommandPalette.svelte 的部分

```svelte
<!-- 移出: 命令面板覆盖层 (~25 行) -->
{#if showCommandPalette}
  <div class="command-palette-overlay" ...>
    <div class="command-palette" ...>
      <input ... />
      <div class="command-list">
        {#each filteredCommands as cmd}
          <div class="command-item" ...>{cmd.name}</div>
        {/each}
      </div>
    </div>
  </div>
{/if}
```

### 从 PanelLayout 样式移入 CommandPalette.svelte 的部分

```css
/* 移出: 命令面板样式 (~35 行) */
.command-palette-overlay { ... }
.command-palette { ... }
.command-input { ... }
.command-list { ... }
.command-item { ... }
.command-item:hover { ... }
```

## 实施顺序

1. **Phase 1**: `fullscreen-manager.svelte.ts` + `dialog-state.svelte.ts` — 最简单，纯状态提取，无模板改动
2. **Phase 2**: `clipboard-operations.ts` — 纯函数提取，需仔细处理依赖注入
3. **Phase 3**: `CommandPalette.svelte` — 组件提取，涉及模板和样式迁移
4. **Phase 4**: `keyboard-shortcuts.svelte.ts` — 最复杂，需处理与 Tab 切换器的交互
5. **Phase 5**: 验证 — `svelte-check` + `cargo check` + 手动测试

## Tab 切换器特殊处理

Alt+Tab 切换器的状态（`switcherActive`、`switcherSelectionId` 等）被多个地方使用：
- `handleGlobalKeydown` 中的 Alt+Tab 处理
- `handleGlobalKeyup` 中的 Alt 释放检测
- `restoreTabContent` 中的 `switcherActive` 检查
- 模板中 `TabBar` 的 `switcherActive`/`switcherSelectionId` props
- `PreviewEditor` 的 `previewTabId` prop

方案：将切换器状态保留在 `keyboard-shortcuts.svelte.ts` 中，通过 getter 暴露给 PanelLayout，PanelLayout 传递给 `TabBar` 和 `PreviewEditor`。

## 受影响文件清单

| 操作 | 文件 |
|------|------|
| 新建 | `src/lib/components/CommandPalette.svelte` |
| 新建 | `src/lib/composables/keyboard-shortcuts.svelte.ts` |
| 新建 | `src/lib/utils/clipboard-operations.ts` |
| 新建 | `src/lib/composables/fullscreen-manager.svelte.ts` |
| 新建 | `src/lib/composables/dialog-state.svelte.ts` |
| 修改 | `src/lib/components/PanelLayout.svelte` |