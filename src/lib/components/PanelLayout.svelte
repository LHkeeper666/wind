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
  import RecycleBinPanel from './RecycleBinPanel.svelte';
  import PreviewEditor from './PreviewEditor.svelte';
  import FullscreenEditor from './FullscreenEditor.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import FullscreenImageViewer from './FullscreenImageViewer.svelte';
  import FullscreenVideoPlayer from './FullscreenVideoPlayer.svelte';
  import FullscreenPdfViewer from './FullscreenPdfViewer.svelte';
  import {
    getCompressDialogVisible, getCompressDialogValue, getCompressMarkPaths,
    getShowConfirmModal, getConfirmFileName,
    getStreamConflictVisible, getStreamConflictName, getScanningConflicts,
    getShowUnsavedConfirm, getPendingActionPath, getPendingAction,
    setCompressDialogVisible, setCompressDialogValue, setCompressMarkPaths,
    setShowConfirmModal, setConfirmFileName,
    setStreamConflictVisible, setStreamConflictName, setScanningConflicts,
    setShowUnsavedConfirm, setPendingActionPath, setPendingAction,
    resolveStreamConflict,
    handleConfirmOverwrite, handleConfirmSkip, handleConfirmAbort,
    handleUnsavedSave, handleUnsavedDiscard, handleUnsavedCancel,
    handleCompressDialogConfirm as doCompressConfirm, handleCompressDialogCancel,
    initDialogDeps,
  } from '$lib/composables/dialog-state.svelte';
  import {
    getFullscreenImageList, getFullscreenImageIndex,
    getFullscreenPdfPath, getFullscreenPdfPage, getFullscreenPdfPageCount, getFullscreenPdfFileSize,
    getFullscreenVideoPlayerPath, getFullscreenVideoPlayerFileSize,
    getEditorInitialLine, setEditorInitialLine,
    handleFullscreenEditor as doFullscreenEditor,
    handleCloseFullscreen as doCloseFullscreen,
    handleSaveFullscreen as doSaveFullscreen,
    handleCloseImageViewer as doCloseImageViewer,
    handleClosePdfViewer as doClosePdfViewer,
    handleCloseVideoPlayer as doCloseVideoPlayer,
    handleImageViewerNavigate as doImageViewerNavigate,
  } from '$lib/composables/fullscreen-manager.svelte';
  import FloatingTerminal from './FloatingTerminal.svelte';
  import { terminalManager } from '$lib/terminal/terminal-manager';
  import SearchModal from './SearchModal.svelte';
  import HelpOverlay from './HelpOverlay.svelte';
  import TabBar from './TabBar.svelte';
  import WindowTitlebar from './WindowTitlebar.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import InputDialog from './InputDialog.svelte';
  import TransferManager from './TransferManager.svelte';
  import ArchivePasswordDialog from './ArchivePasswordDialog.svelte';
  import { transfer, activeTransferCount } from '$lib/stores/transfer';
  import { clipboardSummary, markSummary } from '$lib/stores/clipboard';
  import { directoryCache } from '$lib/utils/directory-cache';
  import { handlePaste as doPaste, initClipboardDeps } from '$lib/utils/clipboard-operations';
  import {
    getSwitcherActive, getSwitcherSelectionId,
    initKeyboardDeps, setup as setupKeyboard, teardown as teardownKeyboard,
    notifyWindowFocusLost,
  } from '$lib/composables/keyboard-shortcuts.svelte';
  import {
    adaptLegacyTransferEvent,
    DirectoryRefreshCoordinator,
    type DirectoryKey,
    directoryKeyId,
  } from '$lib/utils/directory-refresh';

  let currentPath: string = $state('');
  let leftPanelPath: string = $derived($layout.leftMode === 'manual' ? $layout.leftPath : $layout.parentPath);
  let selectedFile: string | null = $state(null);
  let selectedFileIsDir: boolean | null = $state(null);
  let commandPaletteVisible: boolean = $state(false);
  let showFileSearch: boolean = $state(false);
  let showHelp: boolean = $state(false);
  let showTransfer: boolean = $state(false);
  let fileSearchHomeDir: string = $state('');
  let zoomLevel: number = $state(1);
  let previewEditor: PreviewEditor | undefined = $state(undefined);
  let fullscreenEditor: FullscreenEditor | undefined = $state(undefined);
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

  // Recycle bin state
  let recycleBinPanel: RecycleBinPanel | undefined = $state(undefined);
  let recycleBinPreviewItem: { id: string; name: string; original_path: string; date_deleted: number; size: number | null } | null = $state(null);

  // Focus restore state
  let windowReady: boolean = $state(false);
  let focusUnlisten: (() => void) | null = null;

  function tracePerformance(name: string, start: string): void {
    if (!import.meta.env.DEV || typeof performance === 'undefined') return;
    performance.mark(name);
    const measure = performance.measure(name, start, name);
    if (measure.duration > 16) console.debug(`[PanelLayout] ${name}: ${measure.duration.toFixed(1)}ms`);
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

  // Drag state for column resizing
  let isDragging: 'first' | 'second' | null = $state(null);
  let dragStartX: number = 0;
  let dragStartRatios: [number, number, number] = [1, 1, 3];

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

  function panelMatchesChangedPath(panel: DirectoryPanel | undefined, paths: string[]): boolean {
    if (!panel) return false;
    const panelId = directoryKeyId(panel.getDirectoryKey());
    return paths.some(path => directoryKeyId(path) === panelId);
  }

  async function refreshPanelsForDirectoryChanges(paths: string[]): Promise<void> {
    if (paths.length === 0) return;
    if (currentDirectoryPanel?.getProjectTreeState().enabled) {
      await currentDirectoryPanel.refreshProjectDirectories(paths);
      return;
    }

    const refreshTargets = [currentDirectoryPanel, parentDirectoryPanel].filter(
      (panel): panel is DirectoryPanel => !!panel && panelMatchesChangedPath(panel, paths)
    );

    if (refreshTargets.length > 0) {
      await Promise.all(refreshTargets.map(panel => panel.refresh()));
      return;
    }

    await currentDirectoryPanel?.refresh();
  }

  onMount(async () => {
    initDialogDeps({
      getSelectedFile: () => selectedFile,
      getPreviewEditor: () => previewEditor,
      showToast,
      focusPanel,
      getCurrentPath: () => currentPath,
    });
    initClipboardDeps({
      showToast,
      refreshPanels: refreshPanelsForDirectoryChanges,
      onOpenTransfer: () => { showTransfer = true; },
    });
    initKeyboardDeps({
      getPreviewEditor: () => previewEditor,
      getFullscreenEditor: () => fullscreenEditor,
      getRecycleBinPanel: () => recycleBinPanel,
      getCurrentDirectoryPanel: () => currentDirectoryPanel,
      getCommandPaletteVisible: () => commandPaletteVisible,
      setCommandPaletteVisible: (v) => { commandPaletteVisible = v; },
      getShowFileSearch: () => showFileSearch,
      setShowHelp: (v) => { showHelp = v; },
      toggleShowTransfer: () => { showTransfer = !showTransfer; },
      getZoomLevel: () => zoomLevel,
      applyZoom,
      focusPanel,
      handleTabNew,
      handleTabClose,
      handleTabSwitchByIndex,
      handlePaste,
      showToast,
      saveCurrentTabState,
      getTabActiveId: () => getTabsState().activeTabId,
      getTabPhysicalIds: () => getTabsState().tabs.map((t: any) => t.id),
      getTabsMruOrder: () => tabs.getMruOrder().map((t: any) => t.id),
      restoreTabContent,
      getTabById: (id) => getTabsState().tabs.find((t: any) => t.id === id),
      commitTabSwitch: (tabId) => { tabs.switchTab(tabId); },
      swapTab: (delta) => { tabs.swapTab(delta); },
      renameActiveTab: (path) => { tabs.renameTab(getTabsState().activeTabId, getDirName(path)); },
      openFileSearch: async () => {
        const homeDir = await invoke<string>('get_home_dir');
        fileSearchHomeDir = homeDir;
        showFileSearch = true;
      },
      synchronizeProjectTreeWatcher,
    });

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
        void refreshPanelsForDirectoryChanges(paths);
      }, 250);
    });

    // Pre-load vim config so options are ready before first editor init
    vimOptions.preload();

    setupKeyboard();

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
    teardownKeyboard();
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
      if (getSwitcherActive()) return;
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
      setPendingActionPath(path);
      setPendingAction('navigate');
      setShowUnsavedConfirm(true);
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

  function handleSelect(filePath: string, isDir: boolean | null = null) {
    layout.setSelectedFile(filePath);
    selectedFile = filePath;
    selectedFileIsDir = isDir;
  }

  function handleActivate(filePath: string) {
    // Check for unsaved changes before switching files
    if (previewEditor?.getIsModified()) {
      setPendingActionPath(filePath);
      setPendingAction('activate');
      setShowUnsavedConfirm(true);
      return;
    }
    layout.setSelectedFile(filePath);
    selectedFile = filePath;
    selectedFileIsDir = false;
    if (!$layout.previewExpanded) {
      layout.expandPreview();
    }
    focusPanel('preview');
  }

  function handlePaste(force: boolean = false) {
    return doPaste(
      currentPath,
      () => currentDirectoryPanel?.getOperationDirectory() ?? currentPath,
      force,
    );
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
        if ($layout.previewExpanded && previewEditor?.isTocVisible() && !previewEditor?.isTocFocused()) {
          previewEditor.focusToc();
        }
      }
    }
  }

  function handleFullscreenEditor() {
    doFullscreenEditor(selectedFile, currentDirectoryPanel, previewEditor);
  }

  function handleCloseFullscreen() {
    doCloseFullscreen(focusPanel);
  }

  function handleSaveFullscreen(content: string) {
    doSaveFullscreen(content, previewEditor);
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

    // Cleanup — always clear state even if temp file deletion fails
    const tempPath = batchRenameTempPath;
    batchRenameTempPath = null;
    batchRenameFileEntries = [];
    selectedFile = null;
    try { await invoke('delete_temp_file', { path: tempPath }); } catch {}
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

  async function handleCompressDialogConfirm(value: string) {
    await doCompressConfirm(value, currentDirectoryPanel);
  }

  function handleCloseTerminal() {
    layout.hideTerminal();
    focusPanel($layout.activeColumn);
  }

  function handleCloseImageViewer() {
    doCloseImageViewer(focusPanel);
  }

  function handleClosePdfViewer() {
    doClosePdfViewer(focusPanel);
  }

  function handleCloseVideoPlayer() {
    doCloseVideoPlayer(focusPanel);
  }

  function handleImageViewerNavigate(index: number) {
    doImageViewerNavigate(index);
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
    } else if (panel === 'current' && $layout.recycleBinMode && recycleBinPanel) {
      recycleBinPanel.focus();
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
      notifyWindowFocusLost();
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
  <TabBar onSwitchTab={handleTabSwitch} switcherActive={getSwitcherActive()} switcherSelectionId={getSwitcherSelectionId()} />

  <div class="content-area">
    <div class="panel-layout" style="
      grid-template-columns: {$columnWidths.parent}fr 4px {$columnWidths.current}fr 4px {$columnWidths.preview}fr;
    ">
    <!-- Parent Directory Column (or Recycle Bin Overview) -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel parent-panel"
      class:active={$layout.activeColumn === 'parent'}
      onclick={() => focusPanel('parent')}
      onkeydown={() => {}}
      role="region"
      aria-label={$layout.recycleBinMode ? 'Recycle Bin Overview' : 'Parent Directory'}
      tabindex="-1"
    >
      <div style="display: {$layout.recycleBinMode ? 'contents' : 'none'}">
        <div class="recycle-overview">
          <div class="recycle-overview-header">&#x1F5D1; Recycle Bin</div>
          <div class="recycle-overview-content">
            <div class="recycle-stat">
              <span class="recycle-stat-label">Shortcuts</span>
              <div class="recycle-shortcut-list">
                <div class="recycle-shortcut"><kbd>r</kbd> Restore</div>
                <div class="recycle-shortcut"><kbd>d</kbd> Permanent delete</div>
                <div class="recycle-shortcut"><kbd>g d</kbd> Empty recycle bin</div>
                <div class="recycle-shortcut"><kbd>h</kbd> Exit recycle bin</div>
                <div class="recycle-shortcut"><kbd>g r</kbd> Toggle recycle bin</div>
                <div class="recycle-shortcut"><kbd>Space</kbd> Multi-select</div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div style="display: {!$layout.recycleBinMode ? 'contents' : 'none'}">
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

    <!-- Current Directory Column (or Recycle Bin) -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel current-panel"
      class:active={$layout.activeColumn === 'current'}
      onclick={() => focusPanel('current')}
      onkeydown={() => {}}
      role="region"
      aria-label={$layout.recycleBinMode ? 'Recycle Bin' : 'Current Directory'}
      tabindex="-1"
    >
      <div style="display: {$layout.recycleBinMode ? 'contents' : 'none'}">
        <RecycleBinPanel
          bind:this={recycleBinPanel}
          onPreview={(id, item) => {
            recycleBinPreviewItem = item;
            selectedFile = id;
            layout.setSelectedFile(id);
          }}
          onExit={() => {
            layout.recycleBinExit();
            focusPanel('current');
            showToast('Exited Recycle Bin');
          }}
          onToast={showToast}
        />
      </div>
      <div style="display: {!$layout.recycleBinMode ? 'contents' : 'none'}">
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
      {#if $layout.recycleBinMode && recycleBinPreviewItem}
        <div class="recycle-preview-info">
          <div class="recycle-preview-row">
            <span class="recycle-preview-label">Original path:</span>
            <span class="recycle-preview-value" title={recycleBinPreviewItem.original_path}>{recycleBinPreviewItem.original_path}</span>
          </div>
          <div class="recycle-preview-row">
            <span class="recycle-preview-label">Deleted:</span>
            <span class="recycle-preview-value">{new Date(recycleBinPreviewItem.date_deleted * 1000).toLocaleString('zh-CN')}</span>
          </div>
          {#if recycleBinPreviewItem.size !== null}
            <div class="recycle-preview-row">
              <span class="recycle-preview-label">Size:</span>
              <span class="recycle-preview-value">{recycleBinPreviewItem.size < 1024 ? `${recycleBinPreviewItem.size} B` : recycleBinPreviewItem.size < 1048576 ? `${(recycleBinPreviewItem.size / 1024).toFixed(1)} KB` : `${(recycleBinPreviewItem.size / 1048576).toFixed(1)} MB`}</span>
            </div>
          {/if}
        </div>
      {/if}
      <PreviewEditor
        bind:this={previewEditor}
        filePath={selectedFile}
        selectedEntryIsDir={selectedFileIsDir}
        currentTabId={$activeTab.id}
        previewTabId={getSwitcherActive() ? getSwitcherSelectionId() : $activeTab.id}
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
    {#if $layout.fullscreenViewer === 'editor' && selectedFile}
      <FullscreenEditor
        bind:this={fullscreenEditor}
        filePath={selectedFile}
        content={previewEditor?.getContent() || ''}
        initialLine={getEditorInitialLine()}
        onClose={handleCloseFullscreen}
        onSave={handleSaveFullscreen}
      />
    {/if}

    <!-- Fullscreen Image Viewer Overlay -->
    {#if $layout.fullscreenViewer === 'image' && getFullscreenImageList().length > 0}
      <FullscreenImageViewer
        imageList={getFullscreenImageList()}
        currentIndex={getFullscreenImageIndex()}
        onClose={handleCloseImageViewer}
        onNavigate={handleImageViewerNavigate}
      />
    {/if}

    <!-- Fullscreen PDF Viewer Overlay -->
    {#if $layout.fullscreenViewer === 'pdf' && getFullscreenPdfPath()}
      <FullscreenPdfViewer
        pdfPath={getFullscreenPdfPath()}
        initialPage={getFullscreenPdfPage()}
        pageCount={getFullscreenPdfPageCount()}
        fileSize={getFullscreenPdfFileSize()}
        onClose={handleClosePdfViewer}
      />
    {/if}

    <!-- Fullscreen Video Player Overlay -->
    {#if $layout.fullscreenViewer === 'video' && getFullscreenVideoPlayerPath()}
      <FullscreenVideoPlayer
        filePath={getFullscreenVideoPlayerPath()}
        fileSize={getFullscreenVideoPlayerFileSize()}
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

  <ArchivePasswordDialog />
</div>

  <!-- Status Bar -->
  <div class="status-bar">
    <span class="status-mode">{$layout.activeColumn.toUpperCase()}</span>
    <span class="status-path">{currentPath || 'No path'}</span>
    <span class="status-prefix">{$layout.keyPrefix || ''}</span>
    {#if $markSummary}
      <span class="status-clipboard">{$markSummary}</span>
    {:else if $clipboardSummary}
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
  <CommandPalette
    bind:visible={commandPaletteVisible}
    currentPath={currentPath}
    activeColumn={$layout.activeColumn}
    previewMode={previewEditor?.getMode?.() || 'global-normal'}
    leftMode={$layout.leftMode}
    actions={{
      onNavigate: handleNavigate,
      onLeftNavigate: handleLeftNavigate,
      onSelect: handleSelect,
      onShowToast: showToast,
      onToggleTerminal: () => layout.toggleTerminal(),
      onToggleHelp: () => { showHelp = true; },
      onToggleDetach: () => layout.toggleDetach(),
      onToggleProjectTree: async () => { await currentDirectoryPanel?.setProjectMode(!currentDirectoryPanel?.getProjectTreeState().enabled); },
      onRefreshDirectory: () => currentDirectoryPanel?.refresh(),
      onTogglePdfToc: () => previewEditor?.togglePdfToc(),
      onJumpToPdfPage: (p) => previewEditor?.jumpToPdfPage(p),
      onTabNew: handleTabNew,
      onTabClose: handleTabClose,
      onSetProjectTreeRoot: setProjectTreeRoot,
      getPreviewEditor: () => previewEditor,
      getCurrentDirectoryPanel: () => currentDirectoryPanel,
      getTabsState,
      focusPanel,
    }}
  />

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
    visible={getShowUnsavedConfirm()}
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
    visible={getShowConfirmModal()}
    fileName={getConfirmFileName()}
    onOverwrite={handleConfirmOverwrite}
    onSkip={handleConfirmSkip}
    onAbort={handleConfirmAbort}
  />

  <!-- Streaming Conflict Confirm Modal (per-file + apply-to-all) -->
  <ConfirmModal
    visible={getStreamConflictVisible()}
    title="Conflict"
    fileName={getStreamConflictName()}
    buttons={[
      { key: 'o', label: 'verwrite', action: () => resolveStreamConflict('overwrite'), style: 'danger' },
      { key: 's', label: 'kip', action: () => resolveStreamConflict('skip') },
      { key: 'a', label: 'll overwrite', action: () => resolveStreamConflict('overwrite-all'), style: 'danger' },
      { key: 'i', label: 'gnore all', action: () => resolveStreamConflict('skip-all') },
      { key: 'c', label: 'ancel', action: () => resolveStreamConflict('abort') },
    ]}
  />

  <!-- Compress Mark Dialog -->
  <InputDialog
    visible={getCompressDialogVisible()}
    value={getCompressDialogValue()}
    placeholder="Archive name"
    prompt="Archive name:"
    onConfirm={handleCompressDialogConfirm}
    onCancel={handleCompressDialogCancel}
  />

  <!-- Conflict scanning loading indicator -->
  {#if getScanningConflicts()}
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

  /* Recycle bin overview sidebar */
  .recycle-overview {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 12px;
    overflow-y: auto;
  }

  .recycle-overview-header {
    font-size: 14px;
    font-weight: 600;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
    color: var(--warning);
  }

  .recycle-overview-content {
    padding-top: 12px;
  }

  .recycle-stat-label {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 8px;
    display: block;
  }

  .recycle-shortcut-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .recycle-shortcut {
    font-size: 12px;
    color: var(--text-color);
  }

  .recycle-shortcut kbd {
    display: inline-block;
    padding: 1px 5px;
    font-size: 11px;
    font-family: var(--font-mono);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 3px;
    margin-right: 4px;
    min-width: 20px;
    text-align: center;
  }

  /* Recycle bin preview info bar */
  .recycle-preview-info {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .recycle-preview-row {
    display: flex;
    gap: 8px;
    margin-bottom: 2px;
  }

  .recycle-preview-label {
    color: var(--text-muted);
    white-space: nowrap;
  }

  .recycle-preview-value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
