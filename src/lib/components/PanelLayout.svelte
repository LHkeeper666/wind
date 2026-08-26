<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, onDestroy, tick } from 'svelte';
  import { layout, columnWidths } from '$lib/stores/layout';
  import { theme } from '$lib/stores/theme';
  import { tabs, activeTab, type TabState } from '$lib/stores/tabs';
  import { vimOptions } from '$lib/utils/vim-options';
  import DirectoryPanel from './DirectoryPanel.svelte';
  import PreviewEditor from './PreviewEditor.svelte';
  import FullscreenEditor from './FullscreenEditor.svelte';
  import FullscreenImageViewer from './FullscreenImageViewer.svelte';
  import FullscreenVideoPlayer from './FullscreenVideoPlayer.svelte';
  import FullscreenPdfViewer from './FullscreenPdfViewer.svelte';
  import { isVideoFileExt } from '$lib/previewers';
  import FloatingTerminal from './FloatingTerminal.svelte';
  import { terminalManager } from '$lib/terminal/terminal-manager';
  import SearchModal from './SearchModal.svelte';
  import HelpOverlay from './HelpOverlay.svelte';
  import TabBar from './TabBar.svelte';
  import WindowTitlebar from './WindowTitlebar.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import TransferManager from './TransferManager.svelte';
  import { transfer, activeTransferCount } from '$lib/stores/transfer';
  import { clipboard, clipboardSummary } from '$lib/stores/clipboard';
  import { directoryCache } from '$lib/utils/directory-cache';
  import {
    adaptLegacyTransferEvent,
    DirectoryRefreshCoordinator,
    type DirectoryKey,
  } from '$lib/utils/directory-refresh';

  let currentPath: string = $state('');
  let leftPanelPath: string = $derived($layout.leftMode === 'manual' ? $layout.leftPath : $layout.parentPath);
  let selectedFile: string | null = $state(null);
  let showCommandPalette: boolean = $state(false);
  let commandQuery: string = $state('');
  let commandInput: HTMLInputElement | undefined = $state(undefined);
  let showFileSearch: boolean = $state(false);
  let showHelp: boolean = $state(false);
  let showTransfer: boolean = $state(false);
  let fileSearchHomeDir: string = $state('');
  let zoomLevel: number = $state(1);
  let previewEditor: PreviewEditor | undefined = $state(undefined);
  let fullscreenEditor: FullscreenEditor | undefined = $state(undefined);
  let editorInitialLine: number = $state(0);
  let floatingTerminal: FloatingTerminal | undefined = $state(undefined);
  let parentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let currentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let previewPanel: HTMLDivElement | undefined = $state(undefined);
  let refreshCoordinator: DirectoryRefreshCoordinator | null = null;
  let directoryWatchUnlisten: (() => void) | null = null;
  let directoryWatchTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingDirectoryChanges = new Set<string>();
  // Batch rename state
  let batchRenameTempPath: string | null = $state(null);

  // Paste conflict state
  let showConfirmModal: boolean = $state(false);
  let confirmFileName: string = $state('');
  let pasteResolve: ((choice: 'overwrite' | 'skip' | 'abort') => void) | null = null;

  // Streaming conflict prompt state
  let streamConflictVisible: boolean = $state(false);
  let streamConflictName: string = $state('');
  let streamConflictResolve: ((choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') => void) | null = null;
  let scanningConflicts: boolean = $state(false);

  // Unsaved changes confirm state
  let showUnsavedConfirm: boolean = $state(false);
  let pendingActionPath: string = $state('');
  let pendingAction: 'navigate' | 'activate' = $state('activate');

  // Ctrl+W prefix state for vim-style window navigation
  let waitingForWindowKey: boolean = $state(false);
  let windowKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  let altHeld: boolean = $state(false);
  let switcherActive: boolean = $state(false);
  let switcherSelectionId: number = $state(-1);
  let switcherOriginTabId: number = $state(-1);
  let switcherMruIds: number[] = $state([]);
  let switcherPhysicalIds: number[] = $state([]);

  // Focus restore state
  let windowReady: boolean = $state(false);
  let focusUnlisten: (() => void) | null = null;

  function tracePerformance(name: string, start: string): void {
    if (!import.meta.env.DEV || typeof performance === 'undefined') return;
    performance.mark(name);
    const measure = performance.measure(name, start, name);
    if (measure.duration > 16) console.debug(`[PanelLayout] ${name}: ${measure.duration.toFixed(1)}ms`);
  }

  // Tab completion state for command palette
  let completions: { name: string; is_dir: boolean }[] = [];
  let completionIndex: number = -1;
  let completionPrefix: string = '';
  let completionDir: string = '';

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

  // Toast notification
  let toastMessage: string = $state('');
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string) {
    toastMessage = message;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => { toastMessage = ''; }, 3000);
  }

  function applyZoom(level: number) {
    zoomLevel = Math.max(0.5, Math.min(2.0, level));
    document.documentElement.style.setProperty('--zoom-level', String(zoomLevel));
    floatingTerminal?.setZoom(zoomLevel);
    showToast(`Zoom: ${Math.round(zoomLevel * 100)}%`);
  }

  // Resolve a path argument relative to currentPath
  function resolvePath(input: string): string {
    let trimmed = input.trim();
    if (!trimmed) return '';
    // Don't convert slashes for FTP URLs
    if (trimmed.startsWith('ftp://')) return trimmed.replace(/\\/g, '/');
    trimmed = trimmed.replace(/\//g, '\\');
    // Git Bash style: /d/ → D:\, /c/Users → C:\Users, /d → D:\
    // Only single letter after \ is treated as drive letter
    if (/^\\[A-Za-z]$/.test(trimmed) || /^\\[A-Za-z]\\/.test(trimmed)) {
      const rest = trimmed.substring(2); // after \X
      trimmed = trimmed[1].toUpperCase() + ':' + (rest.startsWith('\\') ? rest : (rest ? '\\' + rest : '\\'));
    }
    // Absolute path: X:\, \ (root)
    if (/^[A-Za-z]:\\/.test(trimmed) || trimmed === '\\') {
      return trimmed;
    }
    // Relative path
    return currentPath + '\\' + trimmed;
  }

  // Reset completion state
  function resetCompletion() {
    completions = [];
    completionIndex = -1;
    completionPrefix = '';
    completionDir = '';
  }

  // Trigger tab completion for cd/e commands
  async function triggerCompletion() {
    const q = commandQuery.trim();
    // Parse: "cd path/prefix" or "e path/prefix"
    const cdMatch = q.match(/^(cd\s+)(.+)$/);
    const eMatch = q.match(/^(e\s+)(.+)$/);

    let cmdPrefix: string;
    let pathInput: string;
    let dirsOnly: boolean;

    if (cdMatch) {
      cmdPrefix = cdMatch[1];
      pathInput = cdMatch[2].replace(/\//g, '\\');
      dirsOnly = true;
    } else if (eMatch) {
      cmdPrefix = eMatch[1];
      pathInput = eMatch[2].replace(/\//g, '\\');
      dirsOnly = false;
    } else {
      return;
    }

    // Strip trailing backslash so parsing treats the last component as
    // the partial name (enables cycling through directory completions).
    if (pathInput.endsWith('\\') && pathInput !== '\\') {
      pathInput = pathInput.slice(0, -1);
    }

    // Determine parent dir and partial name
    let parentDir: string;
    let partial: string;

    if (pathInput.includes('\\')) {
      const lastSlash = pathInput.lastIndexOf('\\');
      const dirPart = pathInput.substring(0, lastSlash) || '\\';
      partial = pathInput.substring(lastSlash + 1);
      parentDir = resolvePath(dirPart);
    } else {
      partial = pathInput;
      parentDir = currentPath;
    }

    // Check if we're cycling through existing completions
    let doFetch = true;
    if (completionDir === parentDir && completions.length > 0) {
      const currentMatchIdx = completions.findIndex(c => c.name.toLowerCase() === partial.toLowerCase());
      if (currentMatchIdx >= 0) {
        // Cycle: find next completion that matches the original prefix
        doFetch = false;
        const prefix = completionPrefix.toLowerCase();
        for (let i = 1; i <= completions.length; i++) {
          const idx = (currentMatchIdx + i) % completions.length;
          if (completions[idx].name.toLowerCase().startsWith(prefix)) {
            completionIndex = idx;
            break;
          }
        }
      }
    }

    if (doFetch) {
      try {
        const entries = await invoke<{ name: string; path: string; is_dir: boolean }[]>('read_directory', { path: parentDir });
        const filtered = entries
          .filter(e => e.name.toLowerCase().startsWith(partial.toLowerCase()) && (dirsOnly ? e.is_dir : true));
        if (filtered.length === 0) { resetCompletion(); return; }
        completions = filtered;
        completionDir = parentDir;
        completionPrefix = partial;
        completionIndex = 0;
      } catch { resetCompletion(); return; }
    }

    // Apply completion
    const completed = completions[completionIndex];
    // Rebuild the path: replace the partial with the completed name
    let newPath: string;
    if (pathInput.includes('\\')) {
      const lastSlash = pathInput.lastIndexOf('\\');
      newPath = pathInput.substring(0, lastSlash + 1) + completed.name;
    } else {
      newPath = completed.name;
    }
    commandQuery = cmdPrefix + newPath;
    // Append backslash for directory completions
    if (completed.is_dir && !commandQuery.endsWith('\\')) {
      commandQuery += '\\';
    }
  }

  // Track active column before fullscreen for restoration
  let preFullscreenColumn: 'parent' | 'current' | 'preview' | 'terminal' | null = null;

  // Fullscreen image viewer state
  let fullscreenImageList: { name: string; path: string }[] = $state([]);
  let fullscreenImageIndex: number = $state(0);

  // Fullscreen PDF viewer state
  let fullscreenPdfPath: string = $state('');
  let fullscreenPdfPage: number = $state(0);
  let fullscreenPdfPageCount: number = $state(0);
  let fullscreenPdfFileSize: number = $state(0);

  // Fullscreen video player state
  let fullscreenVideoPlayerPath: string = $state('');
  let fullscreenVideoPlayerFileSize: number = $state(0);

  function isImageFile(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext);
  }

  function isPdfFile(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'pdf';
  }

  function isVideoFile(path: string): boolean {
    return isVideoFileExt(path);
  }

  // Drag state for column resizing
  let isDragging: 'first' | 'second' | null = $state(null);
  let dragStartX: number = 0;
  let dragStartRatios: [number, number, number] = [1, 1, 3];

  const commands = [
    { name: 'Open File', action: () => layout.setActiveColumn('current') },
    { name: 'Open Terminal', action: () => layout.toggleTerminal() },
    { name: 'Open Editor', action: () => layout.setActiveColumn('preview') },
    { name: 'Toggle Terminal', action: () => layout.toggleTerminal() },
    { name: 'ratio 1:1:3', action: () => layout.setRatios([1, 1, 3]) },
    { name: 'ratio 1:1:1', action: () => layout.setRatios([1, 1, 1]) },
    { name: 'Close Panel', action: () => { showCommandPalette = false; } },
    { name: 'Help', action: () => { showCommandPalette = false; showHelp = true; } },
    { name: 'Tab: New', action: () => handleTabNew() },
    { name: 'Tab: Close', action: () => handleTabClose() },
    { name: 'Tab: Swap Next', action: () => tabs.swapTab(1) },
    { name: 'Tab: Swap Prev', action: () => tabs.swapTab(-1) },
  ];

  let filteredCommands = $derived(
    commandQuery
      ? commands.filter(cmd => cmd.name.toLowerCase().includes(commandQuery.toLowerCase()))
      : commands
  );

  function getDirectoryVersion(directory: DirectoryKey): number {
    return refreshCoordinator?.getVersion(directory) ?? 0;
  }

  function handleDirectorySynchronized(panel: 'current' | 'left', _directory: DirectoryKey, version: number) {
    tabs.setActivePanelDirectoryVersion(panel, version);
  }

  function getRefreshPanels() {
    return [
      currentDirectoryPanel ? {
        getDirectoryKey: () => currentDirectoryPanel!.getDirectoryKey(),
        getObservedVersion: () => $activeTab.currentDirectoryVersion,
        synchronize: (version: number) => currentDirectoryPanel!.synchronize(version),
      } : undefined,
      parentDirectoryPanel ? {
        getDirectoryKey: () => parentDirectoryPanel!.getDirectoryKey(),
        getObservedVersion: () => $activeTab.leftDirectoryVersion,
        synchronize: (version: number) => parentDirectoryPanel!.synchronize(version),
      } : undefined,
    ];
  }

  onMount(async () => {
    refreshCoordinator = new DirectoryRefreshCoordinator({
      cache: directoryCache,
      getActivePanels: getRefreshPanels,
    });
    directoryWatchUnlisten = await listen<string[]>('directory-changed', (event) => {
      event.payload.forEach(path => pendingDirectoryChanges.add(path));
      if (directoryWatchTimer) clearTimeout(directoryWatchTimer);
      directoryWatchTimer = setTimeout(() => {
        const paths = [...pendingDirectoryChanges];
        pendingDirectoryChanges.clear();
        void currentDirectoryPanel?.refreshProjectDirectories(paths);
      }, 250);
    });

    // Pre-load vim config so options are ready before first editor init
    vimOptions.preload();

    // Register global listeners in capturing phase
    window.addEventListener('keydown', handleGlobalKeydown, true);
    window.addEventListener('keyup', handleGlobalKeyup, true);
    window.addEventListener('wheel', handleGlobalWheel, { passive: false, capture: true });

    // Register Tauri window focus listener for auto-restore
    getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      handleWindowFocusChanged(focused);
    }).then(unlisten => { focusUnlisten = unlisten; });

    // Delay windowReady to skip initial focus event
    setTimeout(() => { windowReady = true; }, 500);

    // Initialize with home directory
    try {
      const homeDir = await invoke<string>('get_home_dir');
      // Initialize the first tab with home directory
      tabs.renameTab(1, homeDir.split('\\').pop() || homeDir.split('/').pop() || 'home');
      layout.setCurrentPath(homeDir);
      currentPath = homeDir;

      // Set initial focus to current directory panel
      setTimeout(() => {
        if (currentDirectoryPanel) {
          currentDirectoryPanel.focus();
          layout.setActiveColumn('current');
        }
      }, 100);
    } catch (error) {
      console.error('Failed to get home directory:', error);
    }

    fileOpUnlistens.push(transfer.onTerminal((event, outcome) => {
      const mutation = adaptLegacyTransferEvent(event, outcome);
      refreshCoordinator?.acceptMutation(mutation);
      if (transfer.isBatchSettled(mutation.batchId)) {
        refreshCoordinator?.settleBatch(mutation.batchId);
      }
    }));

    // Listen for transfer panel open requests from child components
    window.addEventListener('transfer:open', () => { showTransfer = true; });
  });

  let fileOpUnlistens: (() => void)[] = [];

  onDestroy(() => {
    fileOpUnlistens.forEach(fn => fn());
    refreshCoordinator?.dispose();
    refreshCoordinator = null;
    window.removeEventListener('keydown', handleGlobalKeydown, true);
    window.removeEventListener('keyup', handleGlobalKeyup, true);
    window.removeEventListener('wheel', handleGlobalWheel, { capture: true } as any);
    if (focusUnlisten) { focusUnlisten(); focusUnlisten = null; }
    if (directoryWatchUnlisten) directoryWatchUnlisten();
    if (directoryWatchTimer) clearTimeout(directoryWatchTimer);
    void invoke('stop_watch_directory');
  });

  function getDirName(path: string): string {
    if (!path || path === '/' || path === '\\') return 'root';
    if (path.startsWith('ftp://')) {
      const noPrefix = path.slice(6);
      const slashPos = noPrefix.indexOf('/');
      if (slashPos < 0) return noPrefix;
      const parts = noPrefix.split('/').filter(Boolean);
      return parts[parts.length - 1] || noPrefix;
    }
    const normalized = path.replace(/\//g, '\\');
    const parts = normalized.split('\\').filter(Boolean);
    if (parts.length === 1 && /^[A-Za-z]:$/.test(parts[0])) return parts[0];
    return parts[parts.length - 1] || 'home';
  }

  // Subscribe to layout changes
  $effect(() => {
    const unsubscribe = layout.subscribe(state => {
      currentPath = state.currentPath;
      selectedFile = state.selectedFile;
      // During MRU switcher preview, layout temporarily shows another tab's
      // content without changing activeTabId — skip auto-rename so the origin
      // tab's name isn't clobbered by the previewed tab's directory/file name.
      if (switcherActive) return;
      // Auto-name active tab: file name if selected, otherwise directory name
      const projectTree = currentDirectoryPanel?.getProjectTreeState();
      const name = projectTree?.enabled && projectTree.rootPath
        ? getDirName(projectTree.rootPath)
        : state.selectedFile
        ? (state.selectedFile.split(/[/\\]/).pop() || state.selectedFile)
        : getDirName(state.currentPath);
      const tabsState = getTabsState();
      tabs.renameTab(tabsState.activeTabId, name);
    });
    return unsubscribe;
  });

  function handleNavigate(path: string) {
    if (previewEditor?.getIsModified()) {
      pendingActionPath = path;
      pendingAction = 'navigate';
      showUnsavedConfirm = true;
      return;
    }
    // Preserve slashes for FTP paths, normalize for local
    if (!path.startsWith('ftp://')) {
      path = path.replace(/\//g, '\\');
    }
    layout.setCurrentPath(path);
    currentPath = path;
  }

  // Navigate left panel (used in manual mode when left panel is focused)
  function handleLeftNavigate(path: string) {
    if (!path.startsWith('ftp://')) {
      path = path.replace(/\//g, '\\');
    }
    layout.setLeftPath(path);
  }

  function getParentPathForNavigate(dirPath: string): string {
    if (dirPath.startsWith('ftp://')) {
      const stripped = dirPath.replace(/\/$/, '');
      const lastSlash = stripped.lastIndexOf('/');
      if (lastSlash <= 6) return '\\';
      return stripped.substring(0, lastSlash);
    }
    const normalized = dirPath.replace(/\//g, '\\').replace(/\\$/, '');
    if (normalized === '\\' || /^[A-Za-z]:\\$/.test(normalized)) return '\\';
    const lastSlash = normalized.lastIndexOf('\\');
    return lastSlash > 0 ? normalized.substring(0, lastSlash) : '\\';
  }

  // Tab operations
  function saveCurrentTabState() {
    const t0 = performance.now();
    // Cache full editor state for tab restore
    const state = getTabsState();
    const snapshot = previewEditor?.getEditorStateSnapshot();
    const t1 = performance.now();
    previewEditor?.cacheTabState(state.activeTabId);
    const t2 = performance.now();

    const projectTree = currentDirectoryPanel?.getProjectTreeState();
    const t3 = performance.now();
    tabs.saveActiveTabState({
      cursorIndex: currentDirectoryPanel?.getSelectedIndex() ?? 0,
      scrollOffset: currentDirectoryPanel?.getScrollOffset() ?? 0,
      editorMode: snapshot?.mode ?? 'global-normal',
      previewScrollTop: snapshot?.previewScrollTop ?? 0,
      isModified: snapshot?.isModified ?? false,
      pdfCurrentPage: snapshot?.pdfCurrentPage ?? 0,
      tocOpen: snapshot?.tocOpen ?? true,
      projectMode: projectTree?.enabled ?? false,
      projectRootPath: projectTree?.rootPath ?? null,
      projectExpandedPaths: projectTree?.expandedPaths ?? [],
      projectSelectedPath: projectTree?.selectedPath ?? null,
      projectScrollOffset: projectTree?.scrollOffset ?? 0,
    });
    const t4 = performance.now();
    console.log(`[tab-perf] saveCurrentTabState total=${(t4-t0).toFixed(1)}ms snapshot=${(t1-t0).toFixed(1)}ms cache=${(t2-t1).toFixed(1)}ms projectTree=${(t3-t2).toFixed(1)}ms saveStore=${(t4-t3).toFixed(1)}ms`);
  }

  function handleTabNew() {
    saveCurrentTabState();
    tabs.createTab(currentPath);
    restoreTabAndFocus();
    showToast('Tab created');
  }

  function handleTabClose() {
    const tabsState = getTabsState();
    if (tabsState.tabs.length <= 1) {
      showToast('Cannot close last tab');
      return;
    }
    const closingTabId = tabsState.activeTabId;
    tabs.closeTab(closingTabId);
    terminalManager.destroy(closingTabId);
    previewEditor?.clearTabCache(closingTabId);
    restoreTabAndFocus();
    showToast('Tab closed');
  }

  function handleTabSwitch(tabId: number) {
    if (tabId === getTabsState().activeTabId) return;
    const t0 = performance.now();
    saveCurrentTabState();
    const t1 = performance.now();
    tabs.switchTab(tabId);
    const t2 = performance.now();
    restoreTabAndFocus();
    const t3 = performance.now();
    console.log(`[tab-perf] handleTabSwitch total=${(t3-t0).toFixed(1)}ms save=${(t1-t0).toFixed(1)}ms switch=${(t2-t1).toFixed(1)}ms restore=${(t3-t2).toFixed(1)}ms`);
  }

  function startSwitcher(mode: 'mru' | 'physical', direction: 1 | -1 = 1) {
    const state = getTabsState();
    if (state.tabs.length <= 1) return;
    saveCurrentTabState();
    switcherOriginTabId = state.activeTabId;
    switcherMruIds = tabs.getMruOrder().map((t: TabState) => t.id);
    switcherPhysicalIds = state.tabs.map((t: TabState) => t.id);
    switcherSelectionId = state.activeTabId;
    switcherActive = true;
    if (mode === 'physical') {
      moveSwitcherIn(switcherPhysicalIds, direction);
    } else {
      moveSwitcherIn(switcherMruIds, 1);
    }
  }

  async function synchronizeProjectTreeWatcher(): Promise<void> {
    const tree = currentDirectoryPanel?.getProjectTreeState();
    if (tree?.enabled && tree.rootPath && !tree.rootPath.startsWith('ftp://')) {
      await invoke('start_watch_directory', { path: tree.rootPath });
    } else {
      await invoke('stop_watch_directory');
    }
  }

  async function setProjectTreeRoot(path: string): Promise<void> {
    await tick();
    const enabled = await currentDirectoryPanel?.setProjectMode(true, {
      enabled: true,
      rootPath: path,
      expandedPaths: [],
      selectedPath: null,
      scrollOffset: 0,
    });
    if (enabled) {
      tabs.renameTab(getTabsState().activeTabId, getDirName(path));
      await synchronizeProjectTreeWatcher();
    }
  }

  function moveSwitcherIn(ids: number[], direction: 1 | -1) {
    const idx = ids.indexOf(switcherSelectionId);
    if (idx === -1) return;
    const newIdx = (idx + direction + ids.length) % ids.length;
    switcherSelectionId = ids[newIdx];
    const tab = getTabsState().tabs.find((t: TabState) => t.id === switcherSelectionId);
    if (tab) restoreTabContent(tab);
  }

  function commitSwitcher() {
    if (!switcherActive) return;
    const selectionId = switcherSelectionId;
    const originId = switcherOriginTabId;
    // Commit the active tab before clearing switcher state. PreviewEditor is
    // already rendering into selectionId's slot, so no delayed reload is
    // needed (and a reload can race an in-flight Markdown render).
    if (selectionId !== originId && selectionId >= 0) {
      tabs.switchTab(selectionId);
    }
    switcherActive = false;
    switcherSelectionId = -1;
    switcherOriginTabId = -1;
    switcherMruIds = [];
    switcherPhysicalIds = [];
  }

  function handleTabSwitchByIndex(index: number) {
    const state = getTabsState();
    if (index < 0 || index >= state.tabs.length) return;
    if (index === state.tabs.findIndex((t: any) => t.id === state.activeTabId)) return;
    saveCurrentTabState();
    tabs.switchTabByIndex(index);
    restoreTabAndFocus();
  }

  function restoreTabContent(tab: TabState) {
    if (!tab) return;
    const t0 = performance.now();
    // Deactivate the outgoing tab's editor before restoring the target tab's
    // selectedFile. Must run synchronously before selectedFile assignment so
    // the filePath $effect-triggered loadFile sees mode=global-normal (no stale
    // editor content during async load). Do NOT move into setTimeout/rAF.
    previewEditor?.deactivateTab();
    // Disable project mode before path change so the DirectoryPanel's $effect.pre
    // always loads the new directory in normal mode. If the target tab is project
    // mode, it will be re-enabled after the path settles in tick().then.
    currentDirectoryPanel?.setProjectMode(false);
    if (!tab.projectMode && (tab.cursorIndex > 0 || tab.scrollOffset > 0)) {
      currentDirectoryPanel?.setPendingRestore(tab.cursorIndex, tab.scrollOffset);
    }
    if (tab.projectMode) {
      currentDirectoryPanel?.setSuppressInitialSelect(true);
    }
    const t1 = performance.now();
    // Set activeColumn BEFORE restoreTabState so the editor's activeColumn
    // $effect sees the correct value when mode is restored to editor-normal,
    // preventing it from redirecting focus to the directory panel.
    const targetPanel = (tab.activeColumn === 'terminal' && tab.terminalVisible)
      ? 'terminal'
      : (tab.activeColumn !== 'terminal' ? tab.activeColumn : 'current');
    // If the tab was in editor mode AND the user hadn't explicitly switched
    // focus away (activeColumn was preview), keep focus on preview panel.
    const wasInEditor = tab.editorMode === 'editor-normal' || tab.editorMode === 'editor-insert';
    const actualPanel = wasInEditor && tab.activeColumn === 'preview'
      ? 'preview'
      : targetPanel;
    layout.setActiveColumn(actualPanel);
    if (actualPanel === 'preview') {
      previewEditor?.prepareTabFocus(tab.id, tab.selectedFile, tab.editorMode);
    }
    // Sync PanelLayout local state
    currentPath = tab.currentPath;
    selectedFile = tab.selectedFile;
    const t2 = performance.now();
    // Batch all layout store updates into one to avoid cascading reactive triggers
    layout.restoreTabState({
      columnRatios: tab.columnRatios,
      previewExpanded: tab.previewExpanded,
      originalRatios: tab.originalRatios,
      currentPath: tab.currentPath || '',
      selectedFile: tab.selectedFile,
      terminalVisible: tab.terminalVisible,
      terminalHeight: tab.terminalHeight,
      fullscreenTerminalOpen: tab.fullscreenTerminalOpen,
      leftMode: tab.leftMode || 'auto',
      leftPath: tab.leftPath || '',
    });
    // Re-assert activeColumn — restoreTabState may have triggered reactive
    // effects that changed it (e.g. tab rename callback → layout subscription)
    layout.setActiveColumn(actualPanel);
    const t3 = performance.now();
    console.log(`[tab-perf] restoreTabContent total=${(t3-t0).toFixed(1)}ms deactivate=${(t1-t0).toFixed(1)}ms setColumn=${(t2-t1).toFixed(1)}ms restoreState=${(t3-t2).toFixed(1)}ms`);
    if (import.meta.env.DEV) performance.mark('tab-focus-restore-start');
    const tickStart = performance.now();
    void tick().then(() => {
      const tickDone = performance.now();
      console.log(`[tab-perf] tick-wait=${(tickDone-tickStart).toFixed(1)}ms`);
      if (tab.projectMode) {
        void currentDirectoryPanel?.setProjectMode(true, {
          enabled: true,
          rootPath: tab.projectRootPath,
          expandedPaths: tab.projectExpandedPaths,
          selectedPath: tab.projectSelectedPath,
          scrollOffset: tab.projectScrollOffset,
          skipAutoSelect: true,
        }).then(() => {
          currentDirectoryPanel?.setSuppressInitialSelect(false);
          setTimeout(() => void synchronizeProjectTreeWatcher(), 0);
        });
      } else {
        setTimeout(() => void synchronizeProjectTreeWatcher(), 0);
      }
      focusPanelNow(actualPanel);
      const focusDone = performance.now();
      console.log(`[tab-perf] focusPanel=${(focusDone-tickDone).toFixed(1)}ms`);
      if (import.meta.env.DEV) tracePerformance('tab-focus-restore', 'tab-focus-restore-start');
      setTimeout(() => {
        const syncStart = performance.now();
        if (import.meta.env.DEV) performance.mark('directory-sync-schedule-start');
        void refreshCoordinator?.synchronizeActivePanels().finally(() => {
          console.log(`[tab-perf] syncPanels=${(performance.now()-syncStart).toFixed(1)}ms`);
          if (import.meta.env.DEV) tracePerformance('directory-sync-schedule', 'directory-sync-schedule-start');
        });
      }, 0);
    });
  }

  function restoreTabAndFocus() {
    restoreTabContent(getActiveTab());
  }

  function getActiveTab() {
    const state = getTabsState();
    return state.tabs.find((t: any) => t.id === state.activeTabId);
  }

  function getTabsState() {
    let result: any;
    const unsub = tabs.subscribe(v => result = v);
    unsub();
    return result;
  }

  function handleSelect(filePath: string) {
    layout.setSelectedFile(filePath);
    selectedFile = filePath;
  }

  function handleActivate(filePath: string) {
    // Check for unsaved changes before switching files
    if (previewEditor?.getIsModified()) {
      pendingActionPath = filePath;
      pendingAction = 'activate';
      showUnsavedConfirm = true;
      return;
    }
    layout.setSelectedFile(filePath);
    selectedFile = filePath;
    if (!$layout.previewExpanded) {
      layout.expandPreview();
    }
    focusPanel('preview');
  }

  function promptConflict(fileName: string): Promise<'overwrite' | 'skip' | 'abort'> {
    return new Promise(resolve => {
      confirmFileName = fileName;
      showConfirmModal = true;
      pasteResolve = resolve;
    });
  }

  function promptConflictStream(fileName: string): Promise<'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort'> {
    return new Promise(resolve => {
      streamConflictName = fileName;
      streamConflictVisible = true;
      streamConflictResolve = resolve;
    });
  }

  function resolveStreamConflict(choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') {
    streamConflictVisible = false;
    streamConflictResolve?.(choice);
    streamConflictResolve = null;
  }

  async function scanConflicts(startScan: () => Promise<unknown>): Promise<{ dirSkipMap: Map<string, string[]>; fileSkipSet: Set<string> } | null> {
    const dirSkipMap = new Map<string, string[]>();
    const fileSkipSet = new Set<string>();
    let applyToAll: 'overwrite' | 'skip' | null = null;
    let queue: Array<{ kind: string; dir_source: string; rel_path: string }> = [];
    let processing = false;
    let finished = false;
    let aborted = false;

    const addSkip = (kind: string, dir_source: string, rel: string) => {
      if (kind === 'dir') {
        if (!dirSkipMap.has(dir_source)) dirSkipMap.set(dir_source, []);
        dirSkipMap.get(dir_source)!.push(rel);
      } else {
        fileSkipSet.add(rel);
      }
    };

    return new Promise((resolve) => {
      let unlistenFound: (() => void) | null = null;
      let unlistenDone: (() => void) | null = null;

      const maybeFinish = () => {
        if (finished && !processing && queue.length === 0) {
          unlistenFound?.();
          unlistenDone?.();
          resolve(aborted ? null : { dirSkipMap, fileSkipSet });
        }
      };

      const processQueue = async () => {
        if (processing) return;
        processing = true;
        while (queue.length > 0 && !aborted) {
          const c = queue.shift()!;
          if (applyToAll === 'skip') {
            addSkip(c.kind, c.dir_source, c.rel_path);
          } else if (applyToAll === 'overwrite') {
            // 覆盖所有：不 skip
          } else {
            const choice = await promptConflictStream(c.rel_path);
            if (choice === 'skip') addSkip(c.kind, c.dir_source, c.rel_path);
            else if (choice === 'overwrite-all') applyToAll = 'overwrite';
            else if (choice === 'skip-all') { applyToAll = 'skip'; addSkip(c.kind, c.dir_source, c.rel_path); }
            else if (choice === 'abort') aborted = true;
          }
        }
        queue.length = 0;
        processing = false;
        maybeFinish();
      };

      Promise.all([
        listen<Record<string, unknown>>('transfer-conflict-found', (event) => {
          if (aborted) return;
          queue.push(event.payload as any);
          processQueue();
        }),
        listen('transfer-conflict-scan-done', () => {
          finished = true;
          maybeFinish();
        }),
      ]).then(([found, done]) => {
        unlistenFound = found;
        unlistenDone = done;
        startScan().catch(() => {
          finished = true;
          maybeFinish();
        });
      });
    });
  }

  function handleConfirmOverwrite() {
    showConfirmModal = false;
    pasteResolve?.('overwrite');
    pasteResolve = null;
  }

  function handleConfirmSkip() {
    showConfirmModal = false;
    pasteResolve?.('skip');
    pasteResolve = null;
  }

  function handleConfirmAbort() {
    showConfirmModal = false;
    pasteResolve?.('abort');
    pasteResolve = null;
  }

  async function handleUnsavedSave() {
    showUnsavedConfirm = false;
    const targetPath = pendingActionPath;
    const action = pendingAction;
    pendingActionPath = '';
    const filePath = selectedFile;
    const content = previewEditor?.getContent();
    if (filePath && content !== undefined) {
      try {
        await invoke('write_file', { path: filePath, content });
        previewEditor?.setContent(content);
      } catch (e) {
        showToast(`Failed to save: ${e}`);
        return;
      }
    }
    dispatchPendingAction(targetPath, action);
  }

  function handleUnsavedDiscard() {
    showUnsavedConfirm = false;
    const targetPath = pendingActionPath;
    const action = pendingAction;
    pendingActionPath = '';
    dispatchPendingAction(targetPath, action);
  }

  function handleUnsavedCancel() {
    showUnsavedConfirm = false;
    pendingActionPath = '';
    focusPanel('preview');
  }

  function dispatchPendingAction(path: string, action: 'navigate' | 'activate') {
    if (action === 'navigate') {
      layout.setCurrentPath(path);
      currentPath = path;
    } else {
      layout.setSelectedFile(path);
      selectedFile = path;
      if (!$layout.previewExpanded) {
        layout.expandPreview();
      }
      focusPanel('preview');
    }
  }

  function isFtpPath(p: string): boolean { return p.startsWith('ftp://'); }
  function getFtpConnName(p: string): string {
    const rest = p.slice(6);
    const slash = rest.indexOf('/');
    return slash >= 0 ? rest.substring(0, slash) : rest;
  }
  function getFtpRemotePath(p: string): string {
    const rest = p.slice(6);
    const slash = rest.indexOf('/');
    return slash >= 0 ? rest.substring(slash) : '/';
  }
  function getFtpDestPath(dirPath: string, name: string): string {
    const base = dirPath.replace(/\/$/, '');
    return base + '/' + name;
  }

  async function handlePaste(force: boolean = false) {
    let state: any;
    const unsub = clipboard.subscribe(v => state = v)();
    if (!state.entries || state.entries.length === 0) {
      showToast('Clipboard empty');
      return;
    }

    const entries = state.entries;
    const operation = state.operation;
    const destinationPath = currentDirectoryPanel?.getOperationDirectory() ?? currentPath;
    const destIsFtp = isFtpPath(destinationPath);
    const srcIsFtp = entries.some((e: any) => isFtpPath(e.path));
    const isCrossBackend = destIsFtp || srcIsFtp;

    // Cross-backend paste: skip local conflict detection
    if (isCrossBackend) {
      if (srcIsFtp && !destIsFtp) {
        // FTP → Local: download via TransferManager
        const destDir = destinationPath.replace(/[\\\/]+$/, '');

        const dirs = entries.filter((e: any) => e.is_dir);
        const files = entries.filter((e: any) => !e.is_dir);

        // Stream conflict detection on local targets
        const skipFiles = new Set<string>();
        const skipDirMap = new Map<string, string[]>();   // dir.path (ftp://) -> rel 列表
        if (!force) {
          scanningConflicts = true;
          const result = await scanConflicts(() => invoke('scan_ftp_download_conflicts', {
            connName: getFtpConnName(entries[0].path),
            files: files.map((e: any) => e.path),
            dirs: dirs.map((e: any) => e.path),
            localDir: destDir,
          }));
          scanningConflicts = false;
          if (result === null) {
            showToast('Download aborted');
            currentDirectoryPanel?.refresh();
            return;
          }
          for (const [dirSource, rels] of result.dirSkipMap) {
            skipDirMap.set(dirSource, rels);
          }
          for (const f of result.fileSkipSet) {
            skipFiles.add(f);
          }
        }

        let totalQueued = 0;

        // Download folders as batch downloads (skip internal conflicting files)
        for (const dir of dirs) {
          const skipRel = skipDirMap.get(dir.path) || [];
          await invoke('ftp_download_folder', {
            connName: getFtpConnName(dir.path),
            remotePath: getFtpRemotePath(dir.path),
            localPath: destDir + '\\' + dir.name,
            moveMode: operation === 'cut',
            skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
          });
          totalQueued++;
        }

        // Download files as individual transfers (skip conflicting files)
        const downloadFiles = files.filter((f: any) => !skipFiles.has(f.name));
        if (downloadFiles.length > 0) {
          const tasks = downloadFiles.map((entry: any) => ({
            op_type: 'ftp-download' as const,
            source: entry.path,
            destination: destDir + '\\' + entry.name,
            total_bytes: entry.size || 0,
            conn_name: getFtpConnName(entry.path),
          }));
          const ids = await transfer.enqueueTransfers(tasks);
          totalQueued += ids.length;
        }

        if (totalQueued > 0) {
          showToast(`Queued ${totalQueued} download(s)`);
          showTransfer = true;
        }

        if (operation === 'cut') {
          // Delete remote sources, but preserve skipped files; skip whole dir if any file skipped
          for (const entry of files) {
            if (skipFiles.has(entry.name)) continue;
            await invoke('ftp_delete', { path: entry.path, permanent: true }).catch(() => {});
          }
          for (const dir of dirs) {
            const hasSkipped = (skipDirMap.get(dir.path) || []).length > 0 || (dir.skip_rel_paths || []).length > 0;
            if (hasSkipped) continue;
            await invoke('ftp_delete', { path: dir.path, permanent: true }).catch(() => {});
          }
        }
      } else if (!srcIsFtp && destIsFtp) {
        // Local → FTP: upload via TransferManager
        const destConn = getFtpConnName(destinationPath);
        const destBase = getFtpRemotePath(destinationPath).replace(/\/+$/, '');

        const dirs = entries.filter((e: any) => e.is_dir);
        const files = entries.filter((e: any) => !e.is_dir);

        // Stream conflict detection on remote targets
        const skipFiles = new Set<string>();                // 顶层文件冲突（basename）
        const skipDirMap = new Map<string, string[]>();     // 目录名 -> 内部 skip 相对路径
        if (!force) {
          scanningConflicts = true;
          const result = await scanConflicts(() => invoke('scan_ftp_upload_conflicts', {
            connName: destConn,
            sources: entries.map((e: any) => e.path),
            remoteDir: destBase,
          }));
          scanningConflicts = false;
          if (result === null) {
            showToast('Upload aborted');
            currentDirectoryPanel?.refresh();
            return;
          }
          for (const [dirSource, rels] of result.dirSkipMap) {
            const name = dirSource.split(/[/\\]/).pop() || dirSource;
            skipDirMap.set(name, rels);
          }
          for (const f of result.fileSkipSet) {
            skipFiles.add(f);
          }
        }

        let totalQueued = 0;

        // Upload folders as batch uploads (skip internal conflicting files)
        for (const dir of dirs) {
          const skipRel = skipDirMap.get(dir.name) || [];
          await invoke('ftp_upload_folder', {
            connName: destConn,
            localPath: dir.path,
            remotePath: `${destBase}/${dir.name}`,
            moveMode: operation === 'cut',
            skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
          });
          totalQueued++;
        }

        // Upload files as individual transfers (skip conflicting files)
        const uploadFiles = files.filter((f: any) => !skipFiles.has(f.name));
        if (uploadFiles.length > 0) {
          const tasks = uploadFiles.map((entry: any) => ({
            op_type: 'ftp-upload' as const,
            source: entry.path,
            destination: `ftp://${destConn}${destBase}/${entry.name}`,
            total_bytes: 0,
            conn_name: destConn,
          }));
          const ids = await transfer.enqueueTransfers(tasks);
          totalQueued += ids.length;
        }

        if (totalQueued > 0) {
          showToast(`Queued ${totalQueued} upload(s)`);
          showTransfer = true;
        }

        if (operation === 'cut') {
          const delEntries = entries.filter((e: any) => !e.is_dir && !skipFiles.has(e.name));
          if (delEntries.length > 0) {
            const delTasks = delEntries.map((entry: any) => ({
              op_type: 'delete' as const,
              source: entry.path,
              destination: '',
              total_bytes: entry.size || 0,
            }));
            await transfer.enqueueTransfers(delTasks);
          }
          // Directories in cut mode: delete locally after enqueuing upload batch.
          // Skip directories that had internal conflicting files skipped, to avoid
          // deleting files that were not uploaded.
          for (const dir of dirs) {
            const hasSkipped = (skipDirMap.get(dir.name) || []).length > 0 || (dir.skip_rel_paths || []).length > 0;
            if (hasSkipped) continue;
            await invoke('delete_file', { path: dir.path }).catch(() => {});
          }
        }
      } else if (srcIsFtp && destIsFtp) {
        // FTP → FTP
        const srcConn = getFtpConnName(entries[0].path);
        const destConn = getFtpConnName(destinationPath);
        if (srcConn === destConn) {
          const destBase = getFtpDestPath(destinationPath, '').replace(/\/+$/, '');
          if (operation === 'cut') {
            // Move: server-side rename (instant, regardless of file size)
            for (const entry of entries) {
              const newFullPath = destBase + '/' + entry.name;
              try {
                await invoke('ftp_rename', { oldPath: entry.path, newPath: newFullPath });
              } catch (e: any) {
                showToast(`Move failed: ${e}`);
              }
            }
            showToast(`Moved ${entries.length} item(s) on ${srcConn}`);
          } else {
            // Copy: need download + re-upload (no server-side copy in FTP)
            let done = 0;
            for (const entry of entries) {
              showToast(`Copying ${entry.name} (${done + 1}/${entries.length})...`);
              const newRemote = destBase + '/' + entry.name;
              // Download to temp, re-upload (streaming)
              await invoke('ftp_copy', {
                connName: srcConn,
                srcPath: getFtpRemotePath(entry.path),
                dstPath: newRemote,
              }).catch((e: any) => { showToast(`Copy failed: ${e}`); });
              done++;
            }
            showToast(`Copied ${entries.length} item(s) on ${srcConn}`);
          }
        } else {
          // Different servers: download + upload relay
          showToast('Cross-server transfer not yet supported');
        }
      }

      if (operation === 'cut') {
        clipboard.clear();
      }
      // Refresh panels after cross-backend transfer
      currentDirectoryPanel?.refresh();
      return;
    }

    // Local-to-local paste (existing behavior)
    const destDir = destinationPath.replace(/[\\\/]+$/, '');
    let processed = 0;
    let firstPastedPath: string | null = null;
    const resolvedSources: string[] = [];
    const skipMap = new Map<string, string[]>();

    const dirEntries = entries.filter((e: any) => e.is_dir);
    const fileEntries = entries.filter((e: any) => !e.is_dir);

    // Phase 1a: directories — stream conflict detection
    if (dirEntries.length > 0) {
      if (force) {
        for (const dir of dirEntries) {
          resolvedSources.push(dir.path);
          if (!firstPastedPath) firstPastedPath = destDir + '\\' + dir.name;
        }
      } else {
        const dirTasks = dirEntries.map((entry: any) => ({
          op_type: operation === 'copy' ? 'copy' : 'move',
          source: entry.path,
          destination: destDir + '\\' + entry.name,
          total_bytes: 0,
          skip_rel_paths: entry.skip_rel_paths || [],
        }));
        scanningConflicts = true;
        const result = await scanConflicts(() => invoke('scan_transfer_conflicts', { tasks: dirTasks }));
        scanningConflicts = false;
        if (result === null) {
          showToast('Paste aborted');
          currentDirectoryPanel?.refresh();
          return;
        }
        for (const dir of dirEntries) {
          resolvedSources.push(dir.path);
          if (!firstPastedPath) firstPastedPath = destDir + '\\' + dir.name;
          const skip = result.dirSkipMap.get(dir.path);
          if (skip) skipMap.set(dir.path, skip);
        }
      }
    }

    // Phase 1b: files — existing sync conflict check
    for (const entry of fileEntries) {
      const destPath = destDir + '\\' + entry.name;

      let exists = false;
      try {
        exists = await invoke<boolean>('file_exists', { path: destPath });
      } catch {
        exists = false;
      }

      if (exists) {
        if (!force) {
          const choice = await promptConflict(entry.name);
          if (choice === 'abort') {
            showToast(`Paste aborted (${processed}/${entries.length} done)`);
            currentDirectoryPanel?.refresh();
            return;
          }
          if (choice === 'skip') {
            continue;
          }
        }
        // Overwrite: delete existing first
        try {
          await invoke('delete_file', { path: destPath });
        } catch (e) {
          showToast(`Failed to overwrite ${entry.name}: ${e}`);
          continue;
        }
      }

      resolvedSources.push(entry.path);
      if (!firstPastedPath) firstPastedPath = destPath;
    }

    if (resolvedSources.length === 0) return;

    // Phase 2: execute via TransferManager
    const tasks = resolvedSources.map((src: string) => {
      // Find the original entry to get its size
      const origEntry = entries.find((e: any) => e.path === src);
      return {
        op_type: operation === 'copy' ? 'copy' : 'move',
        source: src,
        destination: destDir + '\\' + (src.split(/[/\\]/).pop() || src),
        total_bytes: origEntry?.size || 0,
        skip_rel_paths: [...(origEntry?.skip_rel_paths || []), ...(skipMap.get(src) || [])],
      };
    });
    const ids = await transfer.enqueueTransfers(tasks);
    showToast(`Queued ${ids.length} transfer(s)`);
    if (ids.length > 0) showTransfer = true;

    // Cut: clear clipboard since source files no longer exist
    if (operation === 'cut') {
      clipboard.clear();
    }

    // Auto-refresh handled by persistent listeners in onMount
  }

  function togglePreviewLayout() {
    if ($layout.previewExpanded) {
      layout.collapsePreview();
    } else {
      layout.expandPreview();
    }
  }

  function handleSwitchPanel(direction: 'left' | 'right') {
    const current = $layout.activeColumn;
    if (direction === 'left') {
      if (current === 'preview') {
        focusPanel('current');
      } else if (current === 'current') {
        focusPanel('parent');
      }
    } else if (direction === 'right') {
      if (current === 'parent') {
        focusPanel('current');
      } else if (current === 'current') {
        focusPanel('preview');
      } else if (current === 'preview') {
        // Ctrl+W l from preview content: switch to TOC if visible and TOC not already focused
        if ($layout.previewExpanded && previewEditor?.isTocVisible() && !previewEditor?.isTocFocused()) {
          previewEditor.focusToc();
        }
      }
    }
  }

  function handleFullscreenEditor() {
    if (selectedFile && isImageFile(selectedFile)) {
      const imageFiles = currentDirectoryPanel?.getImageFiles() || [];
      const idx = imageFiles.findIndex(f => f.path === selectedFile);
      fullscreenImageList = imageFiles;
      fullscreenImageIndex = idx >= 0 ? idx : 0;
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenImageViewer();
    } else if (selectedFile && isPdfFile(selectedFile)) {
      const pdfInfo = previewEditor?.getPdfInfo();
      fullscreenPdfPath = selectedFile;
      fullscreenPdfPage = pdfInfo?.currentPage ?? 0;
      fullscreenPdfPageCount = pdfInfo?.pageCount ?? 0;
      fullscreenPdfFileSize = 0;
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenPdfViewer();
    } else if (selectedFile && isVideoFile(selectedFile)) {
      fullscreenVideoPlayerPath = selectedFile;
      fullscreenVideoPlayerFileSize = currentDirectoryPanel?.getSelectedFileSize() ?? 0;
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenVideoPlayer();
    } else {
      preFullscreenColumn = $layout.activeColumn;
      editorInitialLine = previewEditor?.getVisibleLine() ?? 0;
      layout.openFullscreenEditor();
    }
  }

  function handleCloseFullscreen() {
    layout.closeFullscreenEditor();
    // Restore focus to the column that was active before fullscreen
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleSaveFullscreen(content: string) {
    // Update preview editor content if needed
    if (previewEditor) {
      previewEditor.setContent(content);
    }
  }

  async function handleBatchRenameStart(files: { path: string; name: string }[]) {
    try {
      const tempPath = await invoke<string>('create_batch_rename_temp_file', {
        files: files.map(f => f.name),
      });
      batchRenameTempPath = tempPath;
      batchRenameFileEntries = files;
      // Show temp file in preview editor
      selectedFile = tempPath;
      layout.setSelectedFile(tempPath);
      layout.setActiveColumn('preview');
      // Enter vim normal mode after file loads
      setTimeout(() => previewEditor?.enterEditorMode(), 200);
    } catch (e) {
      showToast(`Failed to create temp file: ${e}`);
    }
  }

  let batchRenameFileEntries: { path: string; name: string }[] = [];

  async function handleBatchRenameSave(content: string) {
    if (!batchRenameTempPath) return;
    const lines = content.split('\n').map(l => l.trim()).filter(l => l.length > 0);
    if (lines.length !== batchRenameFileEntries.length) {
      showToast(`Expected ${batchRenameFileEntries.length} filenames, got ${lines.length}`);
      return;
    }

    const renames: { old_path: string; new_name: string }[] = [];
    for (let i = 0; i < batchRenameFileEntries.length; i++) {
      if (lines[i] !== batchRenameFileEntries[i].name) {
        renames.push({ old_path: batchRenameFileEntries[i].path, new_name: lines[i] });
      }
    }

    if (renames.length === 0) {
      handleBatchRenameCancel();
      return;
    }

    try {
      const result = await invoke<string[]>('batch_rename', { entries: renames });
      showToast(`Renamed ${result.length} file(s)`);
    } catch (e) {
      showToast(`Batch rename error: ${e}`);
    }

    // Cleanup
    await invoke('delete_temp_file', { path: batchRenameTempPath });
    batchRenameTempPath = null;
    batchRenameFileEntries = [];
    selectedFile = null;
    currentDirectoryPanel?.focus();
    // Refresh directory
    currentDirectoryPanel?.refresh();
  }

  async function handleBatchRenameCancel() {
    if (batchRenameTempPath) {
      await invoke('delete_temp_file', { path: batchRenameTempPath });
    }
    batchRenameTempPath = null;
    batchRenameFileEntries = [];
    selectedFile = null;
    currentDirectoryPanel?.focus();
    showToast('Batch rename cancelled');
  }

  function handleCloseTerminal() {
    layout.hideTerminal();
    focusPanel($layout.activeColumn);
  }

  function handleCloseImageViewer() {
    layout.closeFullscreenImageViewer();
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleClosePdfViewer() {
    layout.closeFullscreenPdfViewer();
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleCloseVideoPlayer() {
    layout.closeFullscreenVideoPlayer();
    fullscreenVideoPlayerPath = '';
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleImageViewerNavigate(index: number) {
    fullscreenImageIndex = index;
  }

  async function handleGlobalKeydown(event: KeyboardEvent) {
    // Skip when typing in an input field (InputDialog, SearchModal search box, etc.)
    if ((event.code === 'AltLeft' || event.code === 'AltRight') && !event.repeat) {
      altHeld = true;
    }

    const target = event.target as HTMLElement | null;
    const isTerminalAltTabChord = event.altKey && !event.ctrlKey && !event.metaKey
      && !event.shiftKey && isSupportedAltTabCode(event.code) && isTerminalInputTarget(event.target);
    const isPreviewCodeMirrorTab = event.key === 'Tab'
      && $layout.activeColumn === 'preview'
      && !$layout.fullscreenEditorOpen
      && target instanceof Element
      && target.closest('.preview-panel .cm-editor') !== null;
    if ((target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable)
      && !isTerminalAltTabChord
      && !isPreviewCodeMirrorTab) {
      // Only allow Escape and Ctrl shortcuts through
      if (event.key !== 'Escape' && !event.ctrlKey) return;
    }

    // Tab / Shift+Tab: prevent native focus switching
    // Skip when command palette is open (Tab = path completion)
    if (event.key === 'Tab' && !showCommandPalette) {
      event.preventDefault();
      const isPreviewEditorTab = $layout.activeColumn === 'preview'
        && !$layout.fullscreenEditorOpen
        && target instanceof Element
        && target.closest('.preview-panel .cm-editor') !== null;
      if (isPreviewEditorTab) {
        if (event.shiftKey) {
          previewEditor?.pressShiftTab();
        } else {
          previewEditor?.pressTab();
        }
        return;
      }
      if ($layout.fullscreenEditorOpen) {
        if (event.shiftKey) {
          fullscreenEditor?.pressShiftTab();
        } else {
          fullscreenEditor?.pressTab();
        }
        return;
      }
    }


    // Ctrl+= / Ctrl+- / Ctrl+0 for zoom
    if (event.ctrlKey && (event.key === '=' || event.key === '+')) {
      event.preventDefault();
      applyZoom(zoomLevel + 0.1);
      return;
    }
    if (event.ctrlKey && event.key === '-') {
      event.preventDefault();
      applyZoom(zoomLevel - 0.1);
      return;
    }
    if (event.ctrlKey && event.key === '0') {
      event.preventDefault();
      applyZoom(1);
      return;
    }

    // F1 to show help overlay
    if (event.key === 'F1') {
      event.preventDefault();
      showHelp = true;
      return;
    }

    // Ctrl+T to toggle Transfer Manager
    if (event.ctrlKey && !event.altKey && event.key === 't' && !waitingForWindowKey) {
      const canToggle = !showCommandPalette && !showFileSearch
        && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
        && !$layout.fullscreenTerminalOpen;
      if (canToggle) {
        event.preventDefault();
        showTransfer = !showTransfer;
        return;
      }
    }

    // Ctrl+L to manually restore focus (skip if Ctrl+W prefix is active)
    if (event.ctrlKey && event.shiftKey && event.code === 'KeyE'
      && $layout.activeColumn === 'current' && !showCommandPalette && !showFileSearch) {
      event.preventDefault();
      event.stopPropagation();
      const state = currentDirectoryPanel?.getProjectTreeState();
      await currentDirectoryPanel?.setProjectMode(!state?.enabled);
      const tree = currentDirectoryPanel?.getProjectTreeState();
      if (tree?.enabled && tree.rootPath) tabs.renameTab(getTabsState().activeTabId, getDirName(tree.rootPath));
      await synchronizeProjectTreeWatcher();
      return;
    }

    if (event.ctrlKey && event.key === 'l' && !waitingForWindowKey) {
      const canRestore = !showCommandPalette && !showFileSearch
        && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
        && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen;
      if (canRestore) {
        event.preventDefault();
        focusPanel($layout.activeColumn);
        showToast(`Focus: ${$layout.activeColumn.toUpperCase()}`);
      }
      return;
    }

    // Ctrl+Shift+` to toggle fullscreen terminal
    // When Shift is pressed, backtick becomes tilde
    if (event.ctrlKey && event.shiftKey && (event.key === '`' || event.key === '~')) {
      event.preventDefault();
      if ($layout.fullscreenTerminalOpen) {
        // Exit fullscreen but keep terminal visible
        layout.closeFullscreenTerminal();
      } else if ($layout.terminalVisible) {
        // Terminal visible, enter fullscreen
        layout.openFullscreenTerminal();
      } else {
        // Terminal hidden, show it and enter fullscreen
        layout.showTerminal();
        layout.openFullscreenTerminal();
        focusPanel('terminal');
      }
      return;
    }

    // Ctrl+` to toggle terminal
    if (event.ctrlKey && event.key === '`' && !event.shiftKey) {
      event.preventDefault();
      if ($layout.fullscreenTerminalOpen) {
        // In fullscreen mode, Ctrl+` closes terminal completely
        layout.closeFullscreenTerminal();
        layout.hideTerminal();
        focusPanel($layout.activeColumn);
      } else if ($layout.terminalVisible) {
        layout.hideTerminal();
        focusPanel($layout.activeColumn);
      } else {
        layout.showTerminal();
        focusPanel('terminal');
      }
      return;
    }

    // Ctrl+W prefix for vim-style window navigation
    // Skip when fullscreen terminal is open (no panel switching in fullscreen)
    if (event.ctrlKey && event.key === 'w' && !$layout.fullscreenTerminalOpen) {
      event.preventDefault();
      if ($layout.activeColumn === 'terminal') {
        event.stopPropagation(); // prevent Ctrl+W from reaching xterm.js
      }
      waitingForWindowKey = true;
      layout.setKeyPrefix('^W');
      if (windowKeyTimeout) clearTimeout(windowKeyTimeout);
      windowKeyTimeout = setTimeout(() => { waitingForWindowKey = false; layout.clearKeyPrefix(); }, 1000);
      return;
    }

    // Handle direction keys after Ctrl+W
    if (waitingForWindowKey) {
      waitingForWindowKey = false;
      layout.clearKeyPrefix();
      if (windowKeyTimeout) { clearTimeout(windowKeyTimeout); windowKeyTimeout = null; }

      const code = event.code;
      // TOC focused: Ctrl+W h → focus preview content
      if (previewEditor?.isTocFocused?.() && code === 'KeyH') {
        event.preventDefault();
        event.stopPropagation();
        previewEditor.focusContent();
        return;
      }
      if (code === 'KeyH') {
        event.preventDefault();
        event.stopPropagation();
        handleSwitchPanel('left');
        return;
      } else if (code === 'KeyL') {
        event.preventDefault();
        event.stopPropagation();
        handleSwitchPanel('right');
        return;
      } else if (code === 'KeyJ') {
        event.preventDefault();
        event.stopPropagation();
        if ($layout.terminalVisible && $layout.activeColumn !== 'terminal') {
          layout.setPreTerminalColumn($layout.activeColumn as 'parent' | 'current' | 'preview');
          focusPanel('terminal');
        }
        return;
      } else if (code === 'KeyK') {
        event.preventDefault();
        event.stopPropagation();
        if ($layout.activeColumn === 'terminal') {
          focusPanel($layout.preTerminalColumn === 'terminal' ? 'current' : $layout.preTerminalColumn);
        }
        return;
      } else if (code === 'KeyM') {
        // Ctrl+W m: toggle expanded/collapsed preview layout
        event.preventDefault();
        event.stopPropagation();
        togglePreviewLayout();
        return;
      }
    }

    const previewMode = previewEditor?.getMode?.() || 'global-normal';
    const canOpenCommandPalette = !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen && !$layout.fullscreenTerminalOpen && ($layout.activeColumn !== 'preview' || previewMode === 'global-normal');

    const canUseTabShortcuts = !showCommandPalette && !showFileSearch
      && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
      && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen
      && !($layout.activeColumn === 'preview' && previewMode === 'editor-insert');
    const canUseGlobalFileOperations = !showCommandPalette && !showFileSearch
      && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
      && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen
      && !($layout.activeColumn === 'preview' && previewMode !== 'global-normal');

    if (switcherActive && altHeld) {
      event.preventDefault();
      event.stopPropagation();
      if (event.code === 'KeyM') {
        moveSwitcherIn(switcherMruIds, 1);
      } else if (event.code === 'KeyL') {
        moveSwitcherIn(switcherPhysicalIds, 1);
      } else if (event.code === 'KeyH') {
        moveSwitcherIn(switcherPhysicalIds, -1);
      }
      return;
    }

    const isAltTabShortcut = event.altKey && !event.ctrlKey && !event.metaKey
      && !event.shiftKey && canUseTabShortcuts && isSupportedAltTabCode(event.code);
    if (isAltTabShortcut) {
      if (event.repeat) {
        event.preventDefault();
        event.stopPropagation();
        return;
      }

      let handled = true;
      switch (event.code) {
        case 'KeyN':
          handleTabNew();
          break;
        case 'KeyU':
          handleTabClose();
          break;
        case 'KeyR':
          showToast('Double-click tab name to rename');
          break;
        case 'KeyM':
          startSwitcher('mru');
          break;
        case 'KeyL':
          startSwitcher('physical');
          break;
        case 'KeyH':
          startSwitcher('physical', -1);
          break;
        case 'Comma':
          tabs.swapTab(-1);
          break;
        case 'Period':
          tabs.swapTab(1);
          break;
        case 'KeyD':
          layout.toggleDetach();
          showToast($layout.leftMode === 'manual' ? 'Panel detached' : 'Panel attached');
          break;
        default:
          if (/^Digit[1-9]$/.test(event.code)) {
            handleTabSwitchByIndex(parseInt(event.code.substring(5)) - 1);
          } else {
            handled = false;
          }
      }

      if (!handled) return;
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    // P key for force paste (skip conflict confirmation)
    if (event.key === 'P' && event.shiftKey && !event.ctrlKey && !event.altKey && canUseGlobalFileOperations) {
      event.preventDefault();
      handlePaste(true);
      return;
    }

    // p key for paste (works in directory panels, not in terminal insert or editor)
    if (event.code === 'KeyP' && !event.ctrlKey && !event.altKey && !event.shiftKey && canUseGlobalFileOperations) {
      event.preventDefault();
      handlePaste();
      return;
    }

    if (event.key === ':' && !showCommandPalette && !showFileSearch && canOpenCommandPalette) {
      event.preventDefault();
      showCommandPalette = true;
      commandQuery = '';
      setTimeout(() => commandInput?.focus(), 0);
    }
    if (event.ctrlKey && event.key === 'p') {
      event.preventDefault();
      const homeDir = await invoke<string>('get_home_dir');
      fileSearchHomeDir = homeDir;
      showFileSearch = true;
    }
  }

  function handleGlobalKeyup(event: KeyboardEvent) {
    if (event.code === 'AltLeft' || event.code === 'AltRight') {
      altHeld = event.getModifierState('Alt');
      if (!altHeld && switcherActive) commitSwitcher();
    }
  }

  function handleGlobalWheel(event: WheelEvent) {
    if (!event.ctrlKey) return;
    event.preventDefault();
    applyZoom(zoomLevel + (event.deltaY < 0 ? 0.1 : -0.1));
  }

  function executeCommand(cmd: typeof commands[0]) {
    cmd.action();
    showCommandPalette = false;
    focusPanel($layout.activeColumn);
  }

  function handleCommandKeydown(event: KeyboardEvent) {
    // Tab completion
    if (event.key === 'Tab') {
      event.preventDefault();
      triggerCompletion();
      return;
    }

    // Reset completion state on any non-Tab key
    if (event.key !== 'Tab') {
      resetCompletion();
    }

    if (event.key === 'Escape') {
      showCommandPalette = false;
      focusPanel($layout.activeColumn);
      return;
    }
    if (event.key === 'Enter') {
      const q = commandQuery.trim();

      // detach / attach commands
      if (q === 'detach') {
        layout.detach();
        showToast('Panel detached');
        showCommandPalette = false;
        focusPanel('current');
        return;
      }
      if (q === 'attach') {
        layout.attach();
        showToast('Panel attached');
        showCommandPalette = false;
        focusPanel('current');
        return;
      }
      if (q === 'td') {
        layout.toggleDetach();
        showToast($layout.leftMode === 'manual' ? 'Panel detached' : 'Panel attached');
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      // cd command
      if (q === 'cd' || q.startsWith('cd ')) {
        const arg = q.substring(2).trim();
        const isLeftManual = $layout.activeColumn === 'parent' && $layout.leftMode === 'manual';
        const keepProjectTree = !isLeftManual && currentDirectoryPanel?.getProjectTreeState().enabled;

        if (!arg) {
          invoke<string>('get_home_dir').then(homeDir => {
            if (isLeftManual) {
              handleLeftNavigate(homeDir);
            } else {
              handleNavigate(homeDir);
              if (keepProjectTree) void setProjectTreeRoot(homeDir);
            }
          });
        } else {
          // Handle ftp:// paths directly (no resolvePath needed)
          if (arg.startsWith('ftp://')) {
            const normalized = arg.replace(/\\/g, '/');
            invoke('read_directory', { path: normalized }).then(() => {
              if (isLeftManual) {
                handleLeftNavigate(normalized);
              } else {
                handleNavigate(normalized);
              }
            }).catch((e: any) => {
              showToast(`Failed to open FTP: ${e}`);
            });
          } else {
            const resolved = resolvePath(arg);
            invoke('read_directory', { path: resolved }).then(() => {
              if (isLeftManual) {
                handleLeftNavigate(resolved);
              } else {
                handleNavigate(resolved);
                if (keepProjectTree) void setProjectTreeRoot(resolved);
              }
            }).catch(() => {
              showToast(`E344: Can't find directory: ${arg}`);
            });
          }
        }
        showCommandPalette = false;
        if (isLeftManual) {
          focusPanel('parent');
        } else {
          focusPanel('current');
        }
        return;
      }

      // ftp commands
      if (q === 'ftp list' || q === 'ftp connections') {
        invoke<{name: string; host: string; port: number; user: string}[]>('list_ftp_connections').then(configs => {
          if (configs.length === 0) {
            showToast('No FTP connections');
          } else {
            const lines = configs.map(c => `  ${c.name}: ${c.user}@${c.host}:${c.port}`).join('\n');
            showToast(`FTP connections:\n${lines}`);
          }
        });
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      if (q === 'ftp disconnect') {
        showToast('Usage: :ftp disconnect <name>');
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      if (q.startsWith('ftp disconnect ')) {
        const name = q.substring(15).trim();
        invoke('ftp_disconnect', { name }).then((msg: any) => {
          showToast(msg);
        }).catch((e: any) => showToast(`Error: ${e}`));
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      if (q === 'ftp connect') {
        showToast('Usage: :ftp connect <name> [<host[:port]>] [--port N] [--user X] [--pass X]');
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      if (q.startsWith('ftp connect ')) {
        const rest = q.substring(12).trim();

        // Shorthand: :ftp connect <name>  (reconnect via stored config)
        const simpleRe = /^(\S+)$/;
        const simpleMatch = rest.match(simpleRe);
        if (simpleMatch) {
          const name = simpleMatch[1];
          // Try to ensure connection + navigate
          invoke('check_ftp_connection', { name }).then(() => {
            showToast(`Reconnected to ${name}`);
            const ftpPath = `ftp://${name}/`;
            handleNavigate(ftpPath);
          }).catch(() => {
            showToast(`No stored connection '${name}'. Use: :ftp connect ${name} <host> [--port N]`);
          });
          showCommandPalette = false;
          focusPanel('current');
          return;
        }

        // Full form: <name> <host[:port]> [--port X] [--user X] [--pass X]
        const argRe = /^(\S+)\s+(\S+?)(?::(\d+))?(?:\s+--port[=:\s]+(\d+))?(?:\s+--user[=:\s]+(\S+))?(?:\s+--pass[=:\s]+(\S+))?$/;
        const match = rest.match(argRe);
        if (!match) {
          showToast('Usage: :ftp connect <name> [<host[:port]>] [--port N] [--user X] [--pass X]');
        } else {
          const [, name, host, colonPort, optPort, user, pass] = match;
          const port = optPort || colonPort || null;
          invoke('ftp_connect', {
            name,
            host,
            port: port ? parseInt(port) : null,
            user: user || null,
            password: pass || null,
          }).then((msg: any) => {
            showToast(msg);
            const ftpPath = `ftp://${name}/`;
            invoke('read_directory', { path: ftpPath }).then(() => {
              handleNavigate(ftpPath);
            }).catch((e: any) => {
              showToast(`FTP connected but failed to list: ${e}`);
            });
          }).catch((e: any) => showToast(`Error: ${e}`));
        }
        showCommandPalette = false;
        focusPanel('current');
        return;
      }

      // e command
      if (q === 'e' || q.startsWith('e ') && !q.startsWith('e!')) {
        const arg = q.substring(1).trim();
        if (!arg) {
          currentDirectoryPanel?.refresh();
          showCommandPalette = false;
          focusPanel('current');
        } else if (arg.startsWith('ftp://')) {
          // FTP path: no resolvePath, keep forward slashes
          const normalized = arg.replace(/\\/g, '/');
          invoke('read_directory', { path: normalized }).then(() => {
            handleNavigate(normalized);
            showCommandPalette = false;
            focusPanel('current');
          }).catch((e: any) => {
            showToast(`Failed to open FTP: ${e}`);
            showCommandPalette = false;
            focusPanel('current');
          });
        } else {
          const resolved = resolvePath(arg);
          invoke('read_directory', { path: resolved }).then(() => {
            if (resolved.replace(/\//g, '\\') === currentPath) {
              currentDirectoryPanel?.refresh();
            } else {
              handleNavigate(resolved);
            }
            showCommandPalette = false;
            focusPanel('current');
          }).catch(() => {
            invoke('file_exists', { path: resolved }).then((exists: any) => {
              if (exists) {
                handleSelect(resolved);
                showCommandPalette = false;
                focusPanel('preview');
              } else {
                showToast(`E344: Can't find: ${arg}`);
                showCommandPalette = false;
                focusPanel('current');
              }
            }).catch(() => {
              showToast(`E344: Can't find: ${arg}`);
              showCommandPalette = false;
              focusPanel('current');
            });
          });
          return;
        }
        return;
      }

      // Tab commands
      if (q === 'tab new' || q === 'tabn') {
        handleTabNew();
        showCommandPalette = false;
        return;
      } else if (q === 'tab close' || q === 'tabc') {
        handleTabClose();
        showCommandPalette = false;
        return;
      } else if (q.startsWith('tab rename ') || q.startsWith('tabr ')) {
        const name = q.startsWith('tab rename ') ? q.substring(11).trim() : q.substring(5).trim();
        if (name) {
          tabs.renameTab(getTabsState().activeTabId, name);
        }
        showCommandPalette = false;
        return;
      } else if (q === 'tab swap' || q === 'tabs') {
        tabs.swapTab(1);
        showCommandPalette = false;
        return;
      }
      // Help command
      if (commandQuery === 'help') {
        showCommandPalette = false;
        showHelp = true;
        return;
      }
      // Ratio command
      if (q === 'ratio') {
        layout.setRatios([1, 1, 3]);
        showCommandPalette = false;
        return;
      }
      if (q === 'ratio dual') {
        layout.setRatios([1, 1, 1]);
        showCommandPalette = false;
        return;
      }
      if (commandQuery.startsWith('ratio ')) {
        const ratioStr = commandQuery.substring(6).trim();
        const parts = ratioStr.split(':').map(Number);
        if (parts.length === 3 && parts.every(p => !isNaN(p) && p > 0)) {
          layout.setRatios([parts[0], parts[1], parts[2]]);
          showCommandPalette = false;
          return;
        }
      }
      // transfer slots command
      if (q === 'transfer slots') {
        invoke<[number, number]>('transfer_get_slots').then(([ftp, local]) => {
          showToast(`Transfer slots: FTP=${ftp}, Local=${local}`);
        });
        showCommandPalette = false;
        return;
      }
      if (q.startsWith('transfer slots ftp ')) {
        const n = parseInt(q.substring(19).trim());
        if (n >= 1 && n <= 8) {
          invoke('transfer_set_ftp_slots', { n });
          showToast(`FTP slots set to ${n}`);
        } else {
          showToast('FTP slots: 1–8');
        }
        showCommandPalette = false;
        return;
      }
      if (q.startsWith('transfer slots local ')) {
        const n = parseInt(q.substring(21).trim());
        if (n >= 1 && n <= 8) {
          invoke('transfer_set_local_slots', { n });
          showToast(`Local slots set to ${n}`);
        } else {
          showToast('Local slots: 1–8');
        }
        showCommandPalette = false;
        return;
      }

      // Clip command - show clipboard contents
      if (q === 'clip') {
        let clipState: any;
        const unsub = clipboard.subscribe(v => clipState = v)();
        if (clipState.entries.length === 0) {
          showToast('Clipboard empty');
        } else {
          const op = clipState.operation === 'copy' ? 'yanked' : 'cut';
          const lines = clipState.entries.map((e: any) => `  ${e.name}`).join('\n');
          showToast(`${clipState.entries.length} files ${op}:\n${lines}`);
        }
        showCommandPalette = false;
        return;
      }
      // Clear command - clear clipboard
      if (q === 'clear') {
        clipboard.clear();
        showToast('Clipboard cleared');
        showCommandPalette = false;
        return;
      }
      // Otherwise execute filtered command
      if (filteredCommands.length > 0) {
        executeCommand(filteredCommands[0]);
      }
    }
  }

  function handleFileSearchSelect(path: string, isDir: boolean) {
    if (isDir) {
      handleNavigate(path);
    } else {
      handleSelect(path);
    }
    showFileSearch = false;
  }

  function focusPanel(panel: 'parent' | 'current' | 'preview' | 'terminal') {
    layout.setActiveColumn(panel);
    focusPanelNow(panel);
    void tick().then(() => focusPanelNow(panel));
  }

  function focusPanelNow(panel: 'parent' | 'current' | 'preview' | 'terminal') {
    if (panel === 'terminal' && floatingTerminal) {
      const _ft0 = performance.now();
      floatingTerminal.focus();
      const _ft1 = performance.now();
      if (_ft1 - _ft0 > 1) {
        console.log(`[tab-perf] focusPanelNow terminal.focus=${(_ft1-_ft0).toFixed(1)}ms`);
      }
    } else if (panel === 'parent' && parentDirectoryPanel) {
      parentDirectoryPanel.focus();
    } else if (panel === 'current' && currentDirectoryPanel) {
      currentDirectoryPanel.focus();
    } else if (panel === 'preview' && previewPanel) {
      if (previewEditor?.isTocFocused() && previewEditor?.isTocVisible()) {
        previewEditor.focusToc();
      } else {
        const element = previewPanel.querySelector('.preview-editor') as HTMLElement;
        if (element) element.focus({ preventScroll: true });
        previewEditor?.focusActiveInput();
      }
    }
  }

  function isPanelFocused(): boolean {
    const active = document.activeElement;
    if (!active || active === document.body) return false;
    const panelLayout = document.querySelector('.panel-layout');
    if (!panelLayout || !panelLayout.contains(active)) return false;
    const element = active as HTMLElement;
    const style = window.getComputedStyle(element);
    return style.display !== 'none' && style.visibility !== 'hidden' && element.getClientRects().length > 0;
  }

  function handleAppFocusIn(event: FocusEvent) {
    const target = event.target as HTMLElement;
    if (!target) return;
    const column = target.closest('.parent-panel, .current-panel, .preview-panel');
    if (column) {
      let panel: 'parent' | 'current' | 'preview';
      if (column.classList.contains('parent-panel')) panel = 'parent';
      else if (column.classList.contains('current-panel')) panel = 'current';
      else if (column.classList.contains('preview-panel')) panel = 'preview';
      else return;
      if ($layout.activeColumn !== panel) {
        layout.setActiveColumn(panel);
      }
      return;
    }
    if (target.closest('.terminal-containers')) {
      if ($layout.activeColumn !== 'terminal') {
        layout.setActiveColumn('terminal');
      }
    }
  }

  function handleWindowFocusChanged(focused: boolean) {
    if (!focused) {
      // Window lost focus mid-switcher — commit to avoid a stuck preview state
      if (switcherActive) commitSwitcher();
      altHeld = false;
      return;
    }
    if (!windowReady) return;
    const hadPanelFocus = isPanelFocused();
    focusPanelNow($layout.activeColumn);
    if (!hadPanelFocus) {
      showToast(`Focus: ${$layout.activeColumn.toUpperCase()}`);
    }
  }

  // Column resize drag handlers
  function startResize(event: MouseEvent, handle: 'first' | 'second') {
    event.preventDefault();
    isDragging = handle;
    dragStartX = event.clientX;
    dragStartRatios = [...$layout.columnRatios] as [number, number, number];
    window.addEventListener('mousemove', handleResize);
    window.addEventListener('mouseup', stopResize);
  }

  function handleResize(event: MouseEvent) {
    if (!isDragging) return;

    const containerWidth = window.innerWidth;
    const deltaX = event.clientX - dragStartX;
    const deltaRatio = (deltaX / containerWidth) * 5; // Scale factor

    if (isDragging === 'first') {
      // Resize between parent and current columns
      const newRatio1 = Math.max(0.5, Math.min(3, dragStartRatios[0] + deltaRatio));
      const newRatio2 = Math.max(0.5, Math.min(3, dragStartRatios[1] - deltaRatio));
      layout.setRatios([newRatio1, newRatio2, dragStartRatios[2]]);
    } else if (isDragging === 'second') {
      // Resize between current and preview columns
      const newRatio2 = Math.max(0.5, Math.min(3, dragStartRatios[1] + deltaRatio));
      const newRatio3 = Math.max(0.5, Math.min(3, dragStartRatios[2] - deltaRatio));
      layout.setRatios([dragStartRatios[0], newRatio2, newRatio3]);
    }
  }

  function stopResize() {
    isDragging = null;
    window.removeEventListener('mousemove', handleResize);
    window.removeEventListener('mouseup', stopResize);
  }
</script>

<!-- svelte-ignore a11y_no_nonactive_element_interactions -->
<div class="app-layout" role="application" aria-label="Wind Panel Layout" onfocusin={handleAppFocusIn}>

  <WindowTitlebar />
  <TabBar onSwitchTab={handleTabSwitch} switcherActive={switcherActive} switcherSelectionId={switcherSelectionId} />

  <div class="content-area">
    <div class="panel-layout" style="
      grid-template-columns: {$columnWidths.parent}fr 4px {$columnWidths.current}fr 4px {$columnWidths.preview}fr;
    ">
    <!-- Parent Directory Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel parent-panel"
      class:active={$layout.activeColumn === 'parent'}
      onclick={() => focusPanel('parent')}
      onkeydown={() => {}}
      role="region"
      aria-label="Parent Directory"
      tabindex="-1"
    >
      <DirectoryPanel
        bind:this={parentDirectoryPanel}
        type="parent"
        path={leftPanelPath}
        selectedPath={$layout.currentPath}
        detached={$layout.leftMode === 'manual'}
        onNavigate={(p) => $layout.leftMode === 'manual' ? handleLeftNavigate(p) : handleNavigate(p)}
        onNavigateUp={() => {
          if ($layout.leftMode === 'manual') {
            const parent = getParentPathForNavigate(leftPanelPath);
            handleLeftNavigate(parent);
          } else {
            handleNavigate($layout.parentPath);
          }
        }}
        onSelect={() => {}}  // Parent column doesn't need to select files
        onSwitchPanel={handleSwitchPanel}
        onToast={showToast}
        getDirectoryVersion={getDirectoryVersion}
        onDirectorySynchronized={(directory, version) => handleDirectorySynchronized('left', directory, version)}
      />
    </div>

    <!-- First Resize Handle -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="resize-handle"
      class:active={isDragging === 'first'}
      onmousedown={(e) => startResize(e, 'first')}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Current Directory Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel current-panel"
      class:active={$layout.activeColumn === 'current'}
      onclick={() => focusPanel('current')}
      onkeydown={() => {}}
      role="region"
      aria-label="Current Directory"
      tabindex="-1"
    >
      <DirectoryPanel
        bind:this={currentDirectoryPanel}
        type="current"
        path={$layout.currentPath}
        selectedPath={selectedFile}
        onNavigate={handleNavigate}
        onSelect={handleSelect}
        onActivate={handleActivate}
        onSwitchPanel={handleSwitchPanel}
        onFullscreen={handleFullscreenEditor}
        onNavigateUp={() => handleNavigate($layout.parentPath)}
        onToast={showToast}
        onBatchRenameStart={handleBatchRenameStart}
        getDirectoryVersion={getDirectoryVersion}
        onDirectorySynchronized={(directory, version) => handleDirectorySynchronized('current', directory, version)}
      />
    </div>

    <!-- Second Resize Handle -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="resize-handle"
      class:active={isDragging === 'second'}
      onmousedown={(e) => startResize(e, 'second')}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Preview/Editor Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel preview-panel"
      bind:this={previewPanel}
      class:active={$layout.activeColumn === 'preview'}
      onclick={() => focusPanel('preview')}
      onkeydown={() => {}}
      role="region"
      aria-label="Preview/Editor"
      tabindex="-1"
    >
      <PreviewEditor
        bind:this={previewEditor}
        filePath={selectedFile}
        currentTabId={$activeTab.id}
        previewTabId={switcherActive ? switcherSelectionId : $activeTab.id}
        activeColumn={$layout.activeColumn}
        onFullscreen={handleFullscreenEditor}
        onSwitchPanel={handleSwitchPanel}
        onToast={showToast}
        onToggleLayout={togglePreviewLayout}
        batchRenameTempPath={batchRenameTempPath}
        onBatchRenameSave={handleBatchRenameSave}
        onBatchRenameCancel={handleBatchRenameCancel}
      />
    </div>
    </div>

    <!-- Fullscreen Editor Overlay -->
    {#if $layout.fullscreenEditorOpen && selectedFile}
      <FullscreenEditor
        bind:this={fullscreenEditor}
        filePath={selectedFile}
        content={previewEditor?.getContent() || ''}
        initialLine={editorInitialLine}
        onClose={handleCloseFullscreen}
        onSave={handleSaveFullscreen}
      />
    {/if}

    <!-- Fullscreen Image Viewer Overlay -->
    {#if $layout.fullscreenImageViewerOpen && fullscreenImageList.length > 0}
      <FullscreenImageViewer
        imageList={fullscreenImageList}
        currentIndex={fullscreenImageIndex}
        onClose={handleCloseImageViewer}
        onNavigate={handleImageViewerNavigate}
      />
    {/if}

    <!-- Fullscreen PDF Viewer Overlay -->
    {#if $layout.fullscreenPdfViewerOpen && fullscreenPdfPath}
      <FullscreenPdfViewer
        pdfPath={fullscreenPdfPath}
        initialPage={fullscreenPdfPage}
        pageCount={fullscreenPdfPageCount}
        fileSize={fullscreenPdfFileSize}
        onClose={handleClosePdfViewer}
      />
    {/if}

    <!-- Fullscreen Video Player Overlay -->
    {#if $layout.fullscreenVideoPlayerOpen && fullscreenVideoPlayerPath}
      <FullscreenVideoPlayer
        filePath={fullscreenVideoPlayerPath}
        fileSize={fullscreenVideoPlayerFileSize}
        onClose={handleCloseVideoPlayer}
      />
    {/if}

    <!-- Floating Terminal -->
    <FloatingTerminal
      bind:this={floatingTerminal}
      visible={$layout.terminalVisible}
      fullscreen={$layout.fullscreenTerminalOpen}
      currentPath={currentPath}
      initialCwd={$activeTab.terminalInitialCwd}
      shellType={$activeTab.shellType}
      currentTabId={$activeTab.id}
      zoomLevel={zoomLevel}
      onClose={handleCloseTerminal}
      onInitialCwd={(cwd) => tabs.setActiveTerminalInitialCwd(cwd)}
    />

    <!-- Transfer Manager -->
    <TransferManager
      visible={showTransfer}
      onClose={() => showTransfer = false}
    />
  </div>

  <!-- Status Bar -->
  <div class="status-bar">
    <span class="status-mode">{$layout.activeColumn.toUpperCase()}</span>
    <span class="status-path">{currentPath || 'No path'}</span>
    <span class="status-prefix">{$layout.keyPrefix || ''}</span>
    {#if $clipboardSummary}
      <span class="status-clipboard">{$clipboardSummary}</span>
    {/if}
    {#if $activeTransferCount > 0}
      <span class="status-transfer" onclick={() => showTransfer = true} title="Click to open Transfer Manager">↑↓ {$activeTransferCount}</span>
    {/if}
    <button class="theme-toggle" class:light={$theme === 'dark'} class:dark={$theme === 'light'} onclick={() => theme.toggle()}>
      {$theme === 'dark' ? 'LGT' : 'DRK'}
    </button>
  </div>

  <!-- Command Palette -->
  {#if showCommandPalette}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="command-palette-overlay" onclick={() => { showCommandPalette = false; resetCompletion(); focusPanel($layout.activeColumn); }} onkeydown={(e) => { if (e.key === 'Escape') { showCommandPalette = false; resetCompletion(); focusPanel($layout.activeColumn); } }}>
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="command-palette" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
        <input
          type="text"
          class="command-input"
          placeholder="Type a command..."
          bind:value={commandQuery}
          bind:this={commandInput}
          onkeydown={handleCommandKeydown}
        />
        <div class="command-list">
          {#each filteredCommands as cmd}
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <div class="command-item" onclick={() => executeCommand(cmd)} onkeydown={() => {}}>
              {cmd.name}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <!-- File Search -->
  <SearchModal
    visible={showFileSearch}
    rootPath={fileSearchHomeDir}
    mode="recursive"
    onClose={() => showFileSearch = false}
    onSelect={handleFileSearchSelect}
  />

  <!-- Help Overlay -->
  <HelpOverlay bind:visible={showHelp} />

  <!-- Unsaved Changes Confirm Modal -->
  <ConfirmModal
    visible={showUnsavedConfirm}
    title="Unsaved changes"
    fileName={selectedFile?.split(/[/\\]/).pop() || ''}
    buttons={[
      { key: 'w', label: 'rite', action: handleUnsavedSave, style: 'primary' },
      { key: 'q!', label: 'uit', action: handleUnsavedDiscard, style: 'danger' },
      { key: 'C', label: 'ancel', action: handleUnsavedCancel },
    ]}
  />

  <!-- Paste Conflict Confirm Modal -->
  <ConfirmModal
    visible={showConfirmModal}
    fileName={confirmFileName}
    onOverwrite={handleConfirmOverwrite}
    onSkip={handleConfirmSkip}
    onAbort={handleConfirmAbort}
  />

  <!-- Streaming Conflict Confirm Modal (per-file + apply-to-all) -->
  <ConfirmModal
    visible={streamConflictVisible}
    title="Conflict"
    fileName={streamConflictName}
    buttons={[
      { key: 'o', label: 'verwrite', action: () => resolveStreamConflict('overwrite'), style: 'danger' },
      { key: 's', label: 'kip', action: () => resolveStreamConflict('skip') },
      { key: 'a', label: 'll overwrite', action: () => resolveStreamConflict('overwrite-all'), style: 'danger' },
      { key: 'i', label: 'gnore all', action: () => resolveStreamConflict('skip-all') },
      { key: 'c', label: 'ancel', action: () => resolveStreamConflict('abort') },
    ]}
  />

  <!-- Conflict scanning loading indicator -->
  {#if scanningConflicts}
    <div class="scanning-toast" role="status">
      正在检查冲突…
    </div>
  {/if}

  <!-- Toast Notification -->
  {#if toastMessage}
    <div class="toast" role="alert">
      {toastMessage}
    </div>
  {/if}
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    overflow: hidden;
  }

  .content-area {
    display: flex;
    position: relative;
    flex: 1;
    min-height: 0;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-layout {
    display: grid;
    flex: 1;
    overflow: hidden;
    gap: 1px;
    background-color: var(--border);
  }

  .panel {
    background-color: var(--bg-primary);
    overflow: hidden;
    zoom: var(--zoom-level);
  }

  .panel.active {
    border: 1px solid var(--border-focus);
  }

  .resize-handle {
    background-color: var(--border);
    cursor: col-resize;
    transition: background-color 0.2s ease;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background-color: var(--accent);
  }

  .app-layout.dragging {
    cursor: col-resize;
    user-select: none;
  }

  .status-bar {
    display: flex;
    align-items: center;
    padding: 2px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-secondary);
    gap: 0;
    flex-shrink: 0;
    font-family: var(--font-mono);
  }

  .status-mode {
    font-weight: bold;
    padding: 1px 10px;
    background-color: var(--accent);
    color: var(--bg-primary);
    margin-right: 12px;
  }

  .status-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .status-prefix {
    color: var(--accent);
    font-weight: bold;
    margin-left: 12px;
    min-width: 24px;
    text-align: right;
  }

  .status-clipboard {
    color: var(--text-muted);
    margin-left: 12px;
    font-size: 11px;
  }

  .theme-toggle {
    background: none;
    border: 1px solid var(--border);
    cursor: pointer;
    font-size: 12px;
    padding: 1px 8px;
    line-height: 1;
    margin-left: 8px;
  }

  .theme-toggle.light {
    color: var(--bg-primary);
    background-color: var(--accent);
    border-color: var(--accent);
  }

  .theme-toggle.dark {
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
    border-color: var(--text-muted);
  }

  .command-palette-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 20%;
    z-index: 1000;
  }

  .command-palette {
    width: 400px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    overflow: hidden;
    font-family: var(--font-mono);
  }

  .command-input {
    width: 100%;
    padding: 10px 16px;
    background-color: var(--bg-primary);
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-mono);
    outline: none;
    box-sizing: border-box;
  }

  .command-list {
    max-height: 300px;
    overflow-y: auto;
  }

  .command-item {
    padding: 6px 16px;
    cursor: pointer;
    color: var(--text-primary);
    font-size: 13px;
    transition: background-color 0.1s ease;
  }

  .command-item:hover {
    background-color: var(--bg-hover);
  }

  .toast {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 6px 16px;
    font-size: 12px;
    font-family: var(--font-mono);
    z-index: 2000;
    animation: toast-fade 3s ease-in-out;
    border: 1px solid var(--border);
  }

  .scanning-toast {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--bg-tertiary);
    color: var(--accent);
    padding: 6px 16px;
    font-size: 12px;
    font-family: var(--font-mono);
    z-index: 2000;
    border: 1px solid var(--accent);
  }

  @keyframes toast-fade {
    0% { opacity: 0; }
    10% { opacity: 1; }
    80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
