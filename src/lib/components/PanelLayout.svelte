<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, onDestroy } from 'svelte';
  import { layout, columnWidths } from '$lib/stores/layout';
  import { theme } from '$lib/stores/theme';
  import { tabs, activeTab } from '$lib/stores/tabs';
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
  import ConfirmModal from './ConfirmModal.svelte';
  import FileOpProgress from './FileOpProgress.svelte';
  import { clipboard, clipboardSummary } from '$lib/stores/clipboard';

  let currentPath: string = $state('');
  let selectedFile: string | null = $state(null);
  let showCommandPalette: boolean = $state(false);
  let commandQuery: string = $state('');
  let commandInput: HTMLInputElement | undefined = $state(undefined);
  let showFileSearch: boolean = $state(false);
  let showHelp: boolean = $state(false);
  let fileSearchHomeDir: string = $state('');
  let zoomLevel: number = $state(1);
  let previewEditor: PreviewEditor | undefined = $state(undefined);
  let editorInitialLine: number = $state(0);
  let floatingTerminal: FloatingTerminal | undefined = $state(undefined);
  let parentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let currentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let previewPanel: HTMLDivElement | undefined = $state(undefined);
  let tabBar: TabBar | undefined = $state(undefined);

  // Batch rename state
  let batchRenameTempPath: string | null = $state(null);

  // Paste conflict state
  let showConfirmModal: boolean = $state(false);
  let confirmFileName: string = $state('');
  let pasteResolve: ((choice: 'overwrite' | 'skip' | 'abort') => void) | null = null;

  // Unsaved changes confirm state
  let showUnsavedConfirm: boolean = $state(false);
  let pendingActionPath: string = $state('');
  let pendingAction: 'navigate' | 'activate' = $state('activate');

  // Ctrl+W prefix state for vim-style window navigation
  let waitingForWindowKey: boolean = $state(false);
  let windowKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  // t prefix state for tab operations (global, not per-panel)
  let waitingForTabKey: boolean = $state(false);
  let tabKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  // Focus restore state
  let windowReady: boolean = $state(false);
  let focusUnlisten: (() => void) | null = null;

  // Tab completion state for command palette
  let completions: { name: string; is_dir: boolean }[] = [];
  let completionIndex: number = -1;
  let completionPrefix: string = '';
  let completionDir: string = '';

  // Tab command handler (called from DirectoryPanel's t prefix)
  function handleTabCommand(cmd: string) {
    if (cmd === 'new') {
      handleTabNew();
    } else if (cmd === 'close') {
      handleTabClose();
    } else if (cmd === 'rename-hint') {
      showToast('Double-click tab name to rename');
    } else if (cmd === 'next') {
      handleTabSwitchRelative(1);
    } else if (cmd === 'prev') {
      handleTabSwitchRelative(-1);
    } else if (cmd === 'swap-prev') {
      tabs.swapTab(-1);
    } else if (cmd === 'swap-next') {
      tabs.swapTab(1);
    } else if (cmd.startsWith('switch-')) {
      const idx = parseInt(cmd.substring(7)) - 1;
      handleTabSwitchByIndex(idx);
    }
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
    let trimmed = input.trim().replace(/\//g, '\\');
    if (!trimmed) return '';
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

  onMount(async () => {
    // Register global listeners in capturing phase
    window.addEventListener('keydown', handleGlobalKeydown, true);
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

    // Listen to file operation events to auto-refresh current directory
    const refreshCurrentDir = () => {
      currentDirectoryPanel?.refresh();
    };
    fileOpUnlistens.push(
      await listen('op-complete', refreshCurrentDir),
      await listen('op-cancelled', refreshCurrentDir),
      await listen('op-failed', refreshCurrentDir),
    );
  });

  let fileOpUnlistens: (() => void)[] = [];

  onDestroy(() => {
    fileOpUnlistens.forEach(fn => fn());
    window.removeEventListener('keydown', handleGlobalKeydown, true);
    window.removeEventListener('wheel', handleGlobalWheel, { capture: true } as any);
    if (focusUnlisten) { focusUnlisten(); focusUnlisten = null; }
  });

  // Subscribe to layout changes
  $effect(() => {
    const unsubscribe = layout.subscribe(state => {
      currentPath = state.currentPath;
      selectedFile = state.selectedFile;
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
    layout.setCurrentPath(path);
    currentPath = path;
  }

  // Tab operations
  function saveCurrentTabState() {
    // Cache full editor state for tab restore
    const state = getTabsState();
    previewEditor?.cacheTabState(state.activeTabId);
    const snapshot = previewEditor?.getEditorStateSnapshot();

    tabs.saveActiveTabState({
      cursorIndex: currentDirectoryPanel?.getSelectedIndex() ?? 0,
      scrollOffset: currentDirectoryPanel?.getScrollOffset() ?? 0,
      editorMode: snapshot?.mode ?? 'global-normal',
      previewScrollTop: snapshot?.previewScrollTop ?? 0,
      isModified: snapshot?.isModified ?? false,
      pdfCurrentPage: snapshot?.pdfCurrentPage ?? 0,
      tocOpen: snapshot?.tocOpen ?? true,
    });
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
    saveCurrentTabState();
    tabs.switchTab(tabId);
    restoreTabAndFocus();
  }

  function handleTabSwitchRelative(delta: number) {
    const tStart = performance.now();
    saveCurrentTabState();
    const tSaved = performance.now();
    tabs.switchTabRelative(delta);
    const tSwitched = performance.now();
    restoreTabAndFocus();
    const tRestored = performance.now();
    console.log(`[perf] tab-switch save:${(tSaved-tStart).toFixed(0)}ms switch:${(tSwitched-tSaved).toFixed(0)}ms restore:${(tRestored-tSwitched).toFixed(0)}ms total:${(tRestored-tStart).toFixed(0)}ms`);
  }

  function handleTabSwitchByIndex(index: number) {
    saveCurrentTabState();
    tabs.switchTabByIndex(index);
    restoreTabAndFocus();
  }

  function restoreTabAndFocus() {
    const active = getActiveTab();
    if (!active) return;
    // Set pending cursor/scroll BEFORE path change — for cached dirs, loadDirectory
    // completes synchronously, so pending must be set first
    if (active.cursorIndex > 0 || active.scrollOffset > 0) {
      currentDirectoryPanel?.setPendingRestore(active.cursorIndex, active.scrollOffset);
    }
    // Update layout store (preserve selectedFile during tab switch)
    if (active.currentPath) {
      layout.setCurrentPath(active.currentPath, false);
    }
    // Sync PanelLayout local state
    currentPath = active.currentPath;
    selectedFile = active.selectedFile;
    // Also update layout store's selectedFile
    layout.setSelectedFile(active.selectedFile);
    // Restore terminal state
    layout.setTerminalHeight(active.terminalHeight);
    if (active.terminalVisible) {
      layout.showTerminal();
      if (active.fullscreenTerminalOpen) {
        layout.openFullscreenTerminal();
      } else {
        layout.closeFullscreenTerminal();
      }
      layout.setTerminalMode(active.terminalMode || 'insert');
    } else {
      layout.closeFullscreenTerminal();
      layout.hideTerminal();
    }
    // Restore focus to saved activeColumn
    const targetPanel = (active.activeColumn === 'terminal' && active.terminalVisible)
      ? 'terminal'
      : (active.activeColumn !== 'terminal' ? active.activeColumn : 'current');
    focusPanel(targetPanel);
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

  async function handlePaste(force: boolean = false) {
    let state: any;
    const unsub = clipboard.subscribe(v => state = v)();
    if (!state.entries || state.entries.length === 0) {
      showToast('Clipboard empty');
      return;
    }

    const entries = state.entries;
    const operation = state.operation;
    const destDir = currentPath.replace(/[\\\/]+$/, '');
    let processed = 0;
    let firstPastedPath: string | null = null;
    const resolvedSources: string[] = [];

    // Phase 1: resolve conflicts
    for (const entry of entries) {
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

    // Phase 2: execute async
    if (operation === 'copy') {
      // Batch copy for multiple files
      if (resolvedSources.length > 1) {
        await invoke('copy_file_async', { sources: resolvedSources, destDir });
        showToast(`Copying ${resolvedSources.length} items...`);
      } else {
        await invoke('copy_file_async', { sources: resolvedSources, destDir });
        showToast('Copying...');
      }
    } else {
      // Move: handle one by one through async
      for (const src of resolvedSources) {
        const name = src.split(/[/\\]/).pop() || src;
        const destPath = destDir + '\\' + name;
        await invoke('move_file_async', { source: src, destination: destPath });
      }
      showToast(`Moving ${resolvedSources.length} items...`);
    }

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
    focusPanel('current');
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
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
      // Only allow Escape and Ctrl shortcuts through
      if (event.key !== 'Escape' && !event.ctrlKey) return;
    }

    // Tab / Shift+Tab: prevent native focus switching
    // Skip when command palette is open (Tab = path completion)
    if (event.key === 'Tab' && !showCommandPalette) {
      event.preventDefault();
      if ($layout.activeColumn === 'preview' && !$layout.fullscreenEditorOpen) {
        if (event.shiftKey) {
          previewEditor?.pressShiftTab();
        } else {
          previewEditor?.pressTab();
        }
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

    // Ctrl+L to manually restore focus (skip if Ctrl+W prefix is active)
    if (event.ctrlKey && event.key === 'l' && !waitingForWindowKey) {
      const canRestore = !showCommandPalette && !showFileSearch
        && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
        && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen
        && !($layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert');
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
        focusPanel('current');
      } else if ($layout.terminalVisible) {
        layout.hideTerminal();
        focusPanel('current');
      } else {
        layout.showTerminal();
        focusPanel('terminal');
      }
      return;
    }

    // Ctrl+W prefix for vim-style window navigation
    // Skip when terminal is in insert mode (Ctrl+W should go to shell)
    // Skip when fullscreen terminal is open (no panel switching in fullscreen)
    // Skip when TOC is focused (let PreviewEditor handle Ctrl+W h)
    if (event.ctrlKey && event.key === 'w' && !$layout.fullscreenTerminalOpen && !($layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert')) {
      event.preventDefault();
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
          focusPanel('terminal');
        }
        return;
      } else if (code === 'KeyK') {
        event.preventDefault();
        event.stopPropagation();
        if ($layout.activeColumn === 'terminal') {
          focusPanel('current');
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
    const canOpenCommandPalette = !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen && !$layout.fullscreenTerminalOpen && ($layout.activeColumn !== 'preview' || previewMode === 'global-normal') && !($layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert');

    // t prefix for tab operations (global)
    // Works when: no modal open, not in terminal insert, not in editor insert
    const canUseTabPrefix = !showCommandPalette && !showFileSearch
      && !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen
      && !$layout.fullscreenPdfViewerOpen && !$layout.fullscreenVideoPlayerOpen
      && !($layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert')
      && !($layout.activeColumn === 'preview' && previewMode !== 'global-normal');

    // Handle second key when waiting for t prefix
    if (waitingForTabKey) {
      waitingForTabKey = false;
      layout.clearKeyPrefix();
      if (tabKeyTimeout) { clearTimeout(tabKeyTimeout); tabKeyTimeout = null; }

      const code = event.code;
      const key = event.key;
      event.preventDefault();
      event.stopPropagation();

      if (code === 'KeyT') {
        handleTabNew();
      } else if (code === 'KeyC') {
        handleTabClose();
      } else if (code === 'KeyR') {
        const tabsState = getTabsState();
        tabBar?.triggerRename(tabsState.activeTabId);
      } else if (code === 'KeyN' || code === 'BracketRight') {
        handleTabSwitchRelative(1);
      } else if (code === 'KeyP' || code === 'BracketLeft') {
        handleTabSwitchRelative(-1);
      } else if (code === 'Comma') {
        tabs.swapTab(-1);
      } else if (code === 'Period') {
        tabs.swapTab(1);
      } else if (key >= '1' && key <= '9') {
        handleTabSwitchByIndex(parseInt(key) - 1);
      }
      return;
    }

    // Start t prefix
    if (event.code === 'KeyT' && !event.ctrlKey && !event.altKey && canUseTabPrefix) {
      event.preventDefault();
      event.stopPropagation();
      waitingForTabKey = true;
      layout.setKeyPrefix('t');
      if (tabKeyTimeout) clearTimeout(tabKeyTimeout);
      tabKeyTimeout = setTimeout(() => { waitingForTabKey = false; layout.clearKeyPrefix(); }, 1000);
      return;
    }

    // P key for force paste (skip conflict confirmation)
    if (event.key === 'P' && event.shiftKey && !event.ctrlKey && !event.altKey && canUseTabPrefix) {
      event.preventDefault();
      handlePaste(true);
      return;
    }

    // p key for paste (works in directory panels, not in terminal insert or editor)
    if (event.code === 'KeyP' && !event.ctrlKey && !event.altKey && !event.shiftKey && canUseTabPrefix) {
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

      // cd command
      if (q === 'cd' || q.startsWith('cd ')) {
        const arg = q.substring(2).trim();
        if (!arg) {
          invoke<string>('get_home_dir').then(homeDir => {
            handleNavigate(homeDir);
          });
        } else {
          const resolved = resolvePath(arg);
          invoke('read_directory', { path: resolved }).then(() => {
            handleNavigate(resolved);
          }).catch(() => {
            showToast(`E344: Can't find directory: ${arg}`);
          });
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
      if (commandQuery.startsWith('ratio ')) {
        const ratioStr = commandQuery.substring(6).trim();
        const parts = ratioStr.split(':').map(Number);
        if (parts.length === 3 && parts.every(p => !isNaN(p) && p > 0)) {
          layout.setRatios([parts[0], parts[1], parts[2]]);
          showCommandPalette = false;
          return;
        }
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
    // Use rAF instead of setTimeout so the browser finishes async layout
    // before we call focus(). This avoids a synchronous forced layout on
    // massive DOM (e.g. markdown with thousands of KaTeX formulas).
    requestAnimationFrame(() => {
      if (panel === 'terminal' && floatingTerminal) {
        floatingTerminal.focus();
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
        }
      }
    });
  }

  function isPanelFocused(): boolean {
    const active = document.activeElement;
    if (!active || active === document.body) return false;
    const panelLayout = document.querySelector('.panel-layout');
    return panelLayout ? panelLayout.contains(active) : false;
  }

  function handleWindowFocusChanged(focused: boolean) {
    if (!focused || !windowReady) return;
    if (isPanelFocused()) return;
    focusPanel($layout.activeColumn);
    showToast(`Focus: ${$layout.activeColumn.toUpperCase()}`);
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
<div class="app-layout" role="application" aria-label="Wind Panel Layout">

  <TabBar bind:this={tabBar} onSwitchTab={handleTabSwitch} />

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
        path={$layout.parentPath}
        selectedPath={$layout.currentPath}
        onNavigate={handleNavigate}
        onSelect={() => {}}  // Parent column doesn't need to select files
        onSwitchPanel={handleSwitchPanel}
        onTabCommand={handleTabCommand}
        onToast={showToast}
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
        onTabCommand={handleTabCommand}
        onToast={showToast}
        onBatchRenameStart={handleBatchRenameStart}
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
        onFullscreen={handleFullscreenEditor}
        onSwitchPanel={handleSwitchPanel}
        onToast={showToast}
        onTabCommand={handleTabCommand}
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
    shellType={$activeTab.shellType}
    currentTabId={$activeTab.id}
    zoomLevel={zoomLevel}
    onClose={handleCloseTerminal}
  />

  <!-- File Operation Progress -->
  <FileOpProgress />

  <!-- Status Bar -->
  <div class="status-bar">
    <span class="status-mode">{$layout.activeColumn === 'terminal' ? `TERMINAL-${($layout.terminalMode || 'insert').toUpperCase()}` : $layout.activeColumn.toUpperCase()}</span>
    <span class="status-path">{currentPath || 'No path'}</span>
    <span class="status-prefix">{$layout.keyPrefix || ''}</span>
    {#if $clipboardSummary}
      <span class="status-clipboard">{$clipboardSummary}</span>
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

  .panel-layout {
    display: grid;
    flex: 1;
    overflow: hidden;
    gap: 1px;
    background-color: var(--border);
    transition: grid-template-columns 0.2s ease;
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

  @keyframes toast-fade {
    0% { opacity: 0; }
    10% { opacity: 1; }
    80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
