<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy, tick, untrack } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { layout, type ArchiveFormat } from '$lib/stores/layout';
  import { clipboard, type ClipboardEntry } from '$lib/stores/clipboard';
  import { transfer } from '$lib/stores/transfer';
  import FileListPanel from './FileListPanel.svelte';
  import SearchModal from './SearchModal.svelte';
  import InputDialog from './InputDialog.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import FileInfoPanel from './FileInfoPanel.svelte';
  import { directoryCache } from '$lib/utils/directory-cache';
  import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';
  import { directoryKeyId, normalizeDirectoryKey, type DirectoryKey } from '$lib/utils/directory-refresh';
  import {
    getCollapseSelectionTarget,
    isProjectTreePathWithin as isTreePathWithin,
    projectTreePathKey as treePathKey,
  } from '$lib/utils/project-tree-focus';
  import { isArchiveFile } from '$lib/utils/file-types';
  import {
    type SelectionState,
    createSelectionState,
    hasSelection,
    clearSelection as clearSelectionState,
    togglePathSelection,
    selectTreeSubtree,
    deselectTreeSubtree,
    deselectTreeNode,
    selectTreeNode,
    toggleTreeSelection,
    getSelectedTreeRoot,
    isTreePathExcluded,
    isTreePathPartiallyDeselected,
    isTreeNodeSelected,
    getTreeEntry,
    getTreeSkipPaths,
    getSelectedProjectEntries,
    getEntriesToOperate,
  } from '$lib/utils/selection-manager';
  import {
    type ArchiveState,
    getArchiveFormat,
    createArchiveState,
    getArchiveParentPath,
    readArchiveDirectory,
    readArchiveFile,
    extractArchiveFiles,
    extractArchive,
    deleteArchiveEntries,
    markArchiveForExtraction,
    enterArchive,
  } from '$lib/utils/archive-browser';
  import ProjectTreePanel from './ProjectTreePanel.svelte';
  import { createDirectorySortFilter } from '$lib/composables/directory-sort-filter.svelte';
  import { createProjectTree } from '$lib/composables/project-tree.svelte';
  import { createDirectoryDialogs } from '$lib/composables/directory-dialogs.svelte';
  import type { FileEntry, TreeNode, SelectOptions, ProjectTreeState } from '$lib/types/file-explorer';

  let {
    type = 'current',
    path = '',
    selectedPath = null,
    detached = false,
    onNavigate = (path: string) => {},
    onSelect = (path: string, _isDir?: boolean) => {},
    onActivate = (path: string) => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onFullscreen = () => {},
    onNavigateUp = () => {},
    onToast = (message: string) => {},
    onBatchRenameStart = (_files: { path: string; name: string }[]) => {},
    getDirectoryVersion = (_directory: DirectoryKey) => 0,
    onDirectorySynchronized = (_directory: DirectoryKey, _version: number) => {},
  }: {
    type: 'parent' | 'current';
    path: string;
    selectedPath: string | null;
    detached?: boolean;
    onNavigate?: (path: string) => void;
    onSelect?: (path: string, isDir?: boolean) => void;
    onActivate?: (path: string) => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onFullscreen?: () => void;
    onNavigateUp?: () => void;
    onToast?: (message: string) => void;
    onBatchRenameStart?: (files: { path: string; name: string }[]) => void;
    getDirectoryVersion?: (directory: DirectoryKey) => number;
    onDirectorySynchronized?: (directory: DirectoryKey, version: number) => void;
  } = $props();

  // Archive mode: read from layout store
  let archiveState: { archivePath: string; internalPath: string; format: ArchiveFormat } | null = $derived($layout.archiveState);
  let isArchiveMode: boolean = $derived(archiveState !== null);

  let files: FileEntry[] = $state([]);
  let isLoading: boolean = $state(false);
  let errorMessage: string = $state('');
  let selectedIndex: number = $state(-1);
  let selectedPathInternal: string | null = $state(null);
  let lastKeyTime: number = 0;
  let lastKey: string = '';
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  let isFocused: boolean = $state(false);
  let selectTimeout: ReturnType<typeof setTimeout> | null = null;
  let scrollRafId: number = 0;
  let pendingSelectName: string | null = null;
  let isSearchModalOpen: boolean = $state(false);
  let searchMode: 'current' | 'recursive' = $state('current');

  // Multi-select state
  let selectionState: SelectionState = $state(createSelectionState());

  // Hidden files toggle
  let showHidden: boolean = $state(false);

  const sortFilter = createDirectorySortFilter({
    getFiles: () => files,
    getShowHidden: () => showHidden,
    getIsArchiveMode: () => isArchiveMode,
    getArchiveState: () => archiveState,
    getPath: () => path,
    isVirtualRoot,
    getParentPath,
    onToast,
  });

  const projectTree = createProjectTree({
    getPath: () => path,
    getSelectedFile: () => selectedFile,
    getSelectedIndex: () => selectedIndex,
    getShowHidden: () => showHidden,
    selectByIndex,
    clearSelection: () => { selectionState = clearSelectionState(); },
    onToast,
    onDirectorySynchronized: (dir, ver) => onDirectorySynchronized(dir, ver),
    getDirectoryVersion: (dir) => getDirectoryVersion(dir),
    getScrollOffset,
    setScrollOffset,
    getParentPath,
  });

  const dialogs = createDirectoryDialogs({
    getSelectedIndex: () => selectedIndex,
    getDisplayFiles: () => displayFiles,
    isArchiveMode: () => isArchiveMode,
    getArchiveState: () => archiveState,
    getPath: () => path,
    getOperationDirectory: () => getOperationDirectory(),
    refresh: () => { refresh(); },
    loadDirectory: async (p, f) => { await loadDirectory(p, f); },
    onSelect: (p, d) => onSelect(p, d),
    onToast,
    onBatchRenameStart,
    refocusPanel,
    collectEntriesToOperate,
    setFilterPattern: sortFilter.setFilterPattern,
    setFilterMode: sortFilter.setFilterMode,
  });

  let normalDisplayFiles: FileEntry[] = $derived.by(() => sortFilter.getNormalDisplayFiles());
  let displayFiles: FileEntry[] = $derived(projectTree.getProjectMode() ? projectTree.getTreeVisibleNodes().map(node => node.entry) : normalDisplayFiles);
  let selectedFile: FileEntry | null = $derived(
    selectedIndex >= 0 && selectedIndex < displayFiles.length ? displayFiles[selectedIndex] : null
  );

  // Clamp selectedIndex when displayFiles shrinks (e.g. hidden files toggled off)
  $effect(() => {
    const len = displayFiles.length;
    if (selectedIndex >= len && len > 0) {
      selectedIndex = len - 1;
      selectedPathInternal = displayFiles[selectedIndex].path;
    }
  });

  // Cut file paths from clipboard (for visual indicator)
  let cutPaths: Set<string> = $state(new Set());
  let clipboardUnsub: (() => void) | null = null;

  const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico']);

  export function getImageFiles(): FileEntry[] {
    return files.filter(f => !f.is_dir && IMAGE_EXTENSIONS.has(
      f.name.split('.').pop()?.toLowerCase() || ''
    ));
  }

  export function getSelectedFileSize(): number {
    if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
      return displayFiles[selectedIndex].size ?? 0;
    }
    return 0;
  }

  export function getSelectedIndex(): number {
    return selectedIndex;
  }

  export function getSelectedEntry(): FileEntry | null {
    return selectedIndex >= 0 && selectedIndex < displayFiles.length
      ? displayFiles[selectedIndex]
      : null;
  }

  export function getScrollOffset(): number {
    if (!panelElement) return 0;
    const container = panelElement.querySelector('.panel-content');
    return container?.scrollTop ?? 0;
  }

  export function setSelectedIndex(index: number) {
    if (index >= 0 && index < displayFiles.length) {
      selectedIndex = index;
      selectedPathInternal = displayFiles[index].path;
    }
  }

  export function setScrollOffset(offset: number) {
    if (!panelElement) return;
    const container = panelElement.querySelector('.panel-content');
    if (container) container.scrollTop = offset;
  }

  // Pending cursor/scroll restoration, applied after next directory load
  let pendingCursorIndex: number = -1;
  let pendingScrollOffset: number = -1;
  let _suppressInitialSelect = false;

  export function setPendingRestore(cursorIndex: number, scrollOffset: number) {
    pendingCursorIndex = cursorIndex;
    pendingScrollOffset = scrollOffset;
  }

  export function setSuppressInitialSelect(value: boolean) {
    _suppressInitialSelect = value;
  }

  function applyPendingRestore() {
    if (pendingCursorIndex >= 0) {
      setSelectedIndex(pendingCursorIndex);
    }
    if (pendingScrollOffset >= 0) {
      setScrollOffset(pendingScrollOffset);
    }
    pendingCursorIndex = -1;
    pendingScrollOffset = -1;
  }


  function isVirtualRoot(dirPath: string): boolean {
    return dirPath === '/' || dirPath === '\\';
  }

  function isDriveRoot(dirPath: string): boolean {
    const normalized = dirPath.replace(/\//g, '\\');
    return /^[A-Za-z]:\\$/.test(normalized);
  }

  function getParentPath(dirPath: string): string {
    if (isVirtualRoot(dirPath)) return '/';
    if (isDriveRoot(dirPath)) return '/';
    // FTP paths: use forward-slash logic
    if (dirPath.startsWith('ftp://')) {
      const stripped = dirPath.replace(/\/$/, ''); // strip trailing slash
      const lastSlash = stripped.lastIndexOf('/');
      if (lastSlash <= 6) return '\\'; // ftp:// is 6 chars → root of connection → virtual root
      return stripped.substring(0, lastSlash);
    }
    const normalized = dirPath.replace(/\//g, '\\');
    const lastSlash = normalized.lastIndexOf('\\');
    if (lastSlash <= 0) return '/';
    return normalized.substring(0, lastSlash);
  }

  export function focus() {
    if (panelElement) {
      panelElement.focus({ preventScroll: true });
      isFocused = true;
    }
  }

  export function getDirectoryKey(): DirectoryKey {
    return normalizeDirectoryKey(path);
  }

  export function refresh() {
    if (projectTree.getProjectMode()) return projectTree.refreshProjectTree();
    if (isArchiveMode && archiveState) {
      const key = `${archiveState.archivePath}::${archiveState.internalPath}`;
      return loadDirectory(key, true);
    }
    return loadDirectory(path, true);
  }

  export function synchronize(version: number) {
    if (projectTree.getProjectMode()) return projectTree.refreshProjectTree(version);
    return loadDirectory(path, false, version);
  }

  export function getProjectTreeState(): ProjectTreeState {
    return projectTree.getProjectTreeState();
  }

  export function getOperationDirectory(): string {
    return projectTree.getOperationDirectory();
  }

  export async function setProjectMode(enabled: boolean, state?: ProjectTreeState): Promise<boolean> {
    return projectTree.setProjectMode(enabled, state);
  }

  function handleFocus() {
    isFocused = true;
  }

  function handleBlur() {
    isFocused = false;
  }

  // Load directory content when path changes
  let prevPath: string = '';
  $effect.pre(() => {
    if (path && path !== prevPath) {
      prevPath = path;
      selectedIndex = -1;
      selectedPathInternal = null;
      clearSelection();
      untrack(() => loadDirectory(path, false));
    }
  });

  // Load archive directory when archiveState changes
  let prevArchiveKey: string = '';
  $effect.pre(() => {
    if (archiveState) {
      const key = `${archiveState.archivePath}::${archiveState.internalPath}`;
      if (key !== prevArchiveKey) {
        prevArchiveKey = key;
        selectedIndex = -1;
        selectedPathInternal = null;
        clearSelection();
        untrack(() => loadDirectory(key, false));
      }
    } else if (prevArchiveKey !== '') {
      // Exited archive mode: reload the original directory
      prevArchiveKey = '';
      selectedIndex = -1;
      selectedPathInternal = null;
      clearSelection();
      untrack(() => loadDirectory(path, true));
    }
  });

  // Subscribe to clipboard for cut file indicators
  let transferCompleteUnlisten: (() => void) | null = null;

  onMount(() => {
    clipboardUnsub = clipboard.subscribe(state => {
      if (state.operation === 'cut') {
        cutPaths = new Set(state.entries.map(e => e.path));
      } else {
        cutPaths = new Set();
      }
    });

    // Auto-refresh when a transfer completes in the current directory
    listen<{ source: string; destination: string }>('transfer-complete', (event) => {
      const src = event.payload.source;
      const srcDir = src.replace(/[\\/][^\\/]+$/, '');
      const dstDir = event.payload.destination ? event.payload.destination.replace(/[\\/][^\\/]+$/, '') : '';
      if (srcDir === path || dstDir === path) {
        loadDirectory(path, true);
      }
    }).then(unlisten => { transferCompleteUnlisten = unlisten; });
  });

  onDestroy(() => {
    if (clipboardUnsub) { clipboardUnsub(); clipboardUnsub = null; }
    if (transferCompleteUnlisten) { transferCompleteUnlisten(); transferCompleteUnlisten = null; }
  });

  // Sync from selectedPath prop only when it actually changes
  let prevPropPath: string | null = null;
  $effect(() => {
    const sp = selectedPath;
    const df = displayFiles;
    if (sp !== prevPropPath && sp && df.length > 0) {
      prevPropPath = sp;
      const idx = df.findIndex(entry => entry.path === sp);
      if (idx >= 0) {
        selectedIndex = idx;
        selectedPathInternal = sp;
      }
    }
  });

  // Scroll selected item into view when selectedIndex changes
  $effect(() => {
    const idx = selectedIndex;
    if (idx >= 0 && panelElement) {
      cancelAnimationFrame(scrollRafId);
      scrollRafId = requestAnimationFrame(() => {
        const el = panelElement?.querySelector(`[data-index="${idx}"]`);
        const container = panelElement?.querySelector('.panel-content');
        if (!el || !container) return;

        const cr = container.getBoundingClientRect();
        const er = el.getBoundingClientRect();

        if (er.top < cr.top) {
          container.scrollTop -= cr.top - er.top;
        } else if (er.bottom > cr.bottom) {
          container.scrollTop += er.bottom - cr.bottom;
        }
      });
    }
  });

  function selectInitialEntry() {
    if (_suppressInitialSelect || selectedIndex >= 0 || displayFiles.length === 0) return;
    // If navigating back, try to highlight the directory we came from
    if (pendingSelectName) {
      const target = pendingSelectName;
      pendingSelectName = null;
      const idx = displayFiles.findIndex(f => f.name === target);
      if (idx >= 0) {
        selectedIndex = idx;
        selectedPathInternal = displayFiles[idx].path;
        console.log(`[tab-perf] selectInitialEntry pendingSelectName=${target} file=${displayFiles[idx].name}`);
        onSelect(displayFiles[idx].path, displayFiles[idx].is_dir);
        return;
      }
    }
    // If selectedPath prop is set (e.g. from tab restore), try to use it
    if (selectedPath) {
      const idx = displayFiles.findIndex(f => f.path === selectedPath);
      if (idx >= 0) {
        selectedIndex = idx;
        selectedPathInternal = displayFiles[idx].path;
        console.log(`[tab-perf] selectInitialEntry selectedPath file=${displayFiles[idx].name}`);
        onSelect(displayFiles[idx].path, displayFiles[idx].is_dir);
        return;
      }
    }
    const firstReal = displayFiles.findIndex(f => f.name !== '..');
    selectedIndex = firstReal >= 0 ? firstReal : 0;
    selectedPathInternal = displayFiles[selectedIndex].path;
    console.log(`[tab-perf] selectInitialEntry FALLBACK firstReal=${displayFiles[selectedIndex]?.name}`);
    onSelect(displayFiles[selectedIndex].path, displayFiles[selectedIndex].is_dir);
  }

  let loadingGen = 0;

  function markDirectorySynchronized(dirPath: string, version?: number) {
    const directory = normalizeDirectoryKey(dirPath);
    onDirectorySynchronized(directory, version ?? getDirectoryVersion(directory));
  }

  async function loadDirectory(dirPath: string, forceRefresh: boolean = false, version?: number): Promise<boolean> {
    const t0 = performance.now();
    const gen = ++loadingGen;
    isLoading = true;
    errorMessage = '';

    const isVirtual = isVirtualRoot(dirPath);

    // Check cache first
    if (!forceRefresh && directoryCache.has(dirPath)) {
      files = directoryCache.get(dirPath)!;
      // Safety dedup
      const seen = new Set<string>();
      files = files.filter(f => { if (seen.has(f.path)) return false; seen.add(f.path); return true; });
      if (gen !== loadingGen) return false;
      selectInitialEntry();
      applyPendingRestore();
      markDirectorySynchronized(dirPath, version);
      isLoading = false;
      console.log(`[tab-perf] loadDirectory CACHE_HIT dir=${dirPath.split(/[/\\]/).pop()} time=${(performance.now()-t0).toFixed(1)}ms`);
      return true;
    }

    try {
      if (isVirtual) {
        // Virtual root: list drives
        files = await invoke<FileEntry[]>('list_drives');
      } else if (archiveState) {
        // Archive mode: list entries within the archive
        const archiveEntries = await invokeArchiveWithOptionalPassword<FileEntry[]>(
          'read_archive_directory',
          {
            archivePath: archiveState.archivePath,
            internalPath: archiveState.internalPath,
          },
          'password'
        );
        if (archiveEntries === null) {
          if (archiveState.internalPath === '') {
            layout.clearArchiveState();
          }
          return false;
        }
        files = archiveEntries;
        files.sort((a, b) => {
          if (a.is_dir && !b.is_dir) return -1;
          if (!a.is_dir && b.is_dir) return 1;
          return a.name.localeCompare(b.name);
        });
      } else {
        files = await invoke<FileEntry[]>('read_directory', { path: dirPath });
        files.sort((a, b) => {
          if (a.is_dir && !b.is_dir) return -1;
          if (!a.is_dir && b.is_dir) return 1;
          return a.name.localeCompare(b.name);
        });
      }
      // Client-side dedup by path (safety net, backend should already handle this)
      const seen = new Set<string>();
      files = files.filter(f => {
        if (seen.has(f.path)) return false;
        seen.add(f.path);
        return true;
      });

      // Discard stale results from superseded concurrent calls
      if (gen !== loadingGen) return false;

      // Update cache (store without .., inject on read)
      directoryCache.set(dirPath, files.filter(f => f.name !== '..'));
      selectInitialEntry();
      applyPendingRestore();
      markDirectorySynchronized(dirPath, version);
      return true;
    } catch (error) {
      if (gen !== loadingGen) return false;
      console.error('Failed to load directory:', error);
      errorMessage = `Failed to load: ${error}`;
      return false;
    } finally {
      if (gen === loadingGen) {
        isLoading = false;
      }
    }
  }


  export async function refreshProjectDirectories(paths: string[]): Promise<void> {
    return projectTree.refreshProjectDirectories(paths);
  }

  function selectByIndex(index: number, options: SelectOptions = {}) {
    if (index >= 0 && index < displayFiles.length) {
      selectedIndex = index;
      selectedPathInternal = displayFiles[index].path;
      const selectedPath = displayFiles[index].path;
      const notify = options.notify ?? 'debounced';

      // Debounce onSelect to avoid rapid file loading
      if (selectTimeout) {
        clearTimeout(selectTimeout);
        selectTimeout = null;
      }
      if (notify === 'silent') {
        return;
      }
      if (notify === 'immediate') {
        onSelect(selectedPath, displayFiles[index].is_dir);
        return;
      }
      const capturedIndex = index;
      const capturedPath = selectedPath;
      selectTimeout = setTimeout(() => {
        // Only fire if user is still on the same item
        if (selectedIndex === capturedIndex && displayFiles[capturedIndex]?.path === capturedPath) {
          onSelect(capturedPath, displayFiles[capturedIndex]?.is_dir);
        }
      }, 200);
    }
  }

  function openSearchModal() {
    isSearchModalOpen = true;
  }

  function closeSearchModal() {
    isSearchModalOpen = false;
    // Refocus the panel
    if (panelElement) {
      panelElement.focus();
    }
  }

  function handleSearchSelect(filePath: string, isDir: boolean, isHidden: boolean) {
    if (projectTree.getProjectMode()) {
      void projectTree.revealTreePath(filePath, isHidden).then(() => {
        if (!isDir) onSelect(filePath, false);
      });
      return;
    }
    if (isDir) {
      onNavigate(filePath);
    } else {
      onSelect(filePath, false);
    }
  }


  function getSelectedProjectNodes(): TreeNode[] {
    const root = projectTree.getProjectRoot();
    if (!root) return [];
    const nodes: TreeNode[] = [];
    const visit = (node: TreeNode) => {
      if (selectionState.selectedPaths.has(node.entry.path)) nodes.push(node);
      node.children.forEach(visit);
    };
    visit(root);
    return nodes;
  }

  function checkHasSelection(): boolean {
    return hasSelection(selectionState);
  }

  function clearSelection(): void {
    selectionState = clearSelectionState();
  }

  function collectEntriesToOperate(): ClipboardEntry[] {
    return getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, projectTree.getProjectMode(), projectTree.findTreeNode);
  }

  function startBatchRename() {
    const entries = projectTree.getProjectMode()
      ? getSelectedProjectNodes().filter(node => !node.entry.is_dir && isTreeNodeSelected(selectionState, node.entry.path)).map(node => node.entry)
      : files.filter(entry => selectionState.selectedPaths.has(entry.path));
    if (entries.length < 2) return;
    const renameFiles = entries.map(f => ({ path: f.path, name: f.name }));
    onBatchRenameStart(renameFiles);
  }


  async function openSelectedFile() {
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    try {
      await invoke('open_file', { path: entry.path });
    } catch (e) {
      onToast(`Failed to open: ${e}`);
    }
  }

  async function openSelectedFileWith() {
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    try {
      await invoke('open_with_dialog', { path: entry.path });
    } catch (e) {
      onToast(`Failed to open: ${e}`);
    }
  }

  // Refresh helper that returns a promise
  let refreshResolve: (() => void) | null = null;
  function currentDirectoryPanel_refresh(): Promise<void> {
    return new Promise(resolve => {
      refreshResolve = resolve;
      loadDirectory(path, true).then(() => {
        resolve();
        refreshResolve = null;
      });
    });
  }


  function refocusPanel() {
    setTimeout(() => {
      if (panelElement) panelElement.focus();
    }, 0);
  }

  async function handleDelete(permanent: boolean) {
    const entries = getEntriesToOperate(
      selectionState,
      files,
      displayFiles,
      selectedIndex,
      projectTree.getProjectMode(),
      projectTree.findTreeNode
    );
    if (entries.length === 0) return;

    const confirmed = await dialogs.promptDelete(permanent);
    if (!confirmed) return;

    if (permanent) {
      // Permanent delete: route through TransferManager for progress display
      const tasks = entries.map((entry: ClipboardEntry) => {
        const isFtp = entry.path.startsWith('ftp://');
        return {
          op_type: 'delete' as const,
          source: entry.path,
          destination: '',
          total_bytes: entry.size || 0,
          conn_name: isFtp ? entry.path.slice(6).split('/')[0] : undefined,
          skip_rel_paths: entry.skip_rel_paths || [],
          permanent: true,
        };
      });
      const ids = await transfer.enqueueTransfers(tasks);
      window.dispatchEvent(new CustomEvent('transfer:open'));
      onToast(`Deleting ${ids.length} ${ids.length === 1 ? 'file' : 'files'}...`);
    } else {
      // Move to trash: invoke directly, add synthetic transfer entries
      for (const entry of entries) {
        try {
          await invoke('delete_file', { path: entry.path });
          transfer.addSyntheticEntry({
            opType: 'delete',
            source: entry.path,
            destination: 'Recycle Bin',
            totalBytes: entry.size || 0,
            status: 'done',
          });
        } catch (e) {
          transfer.addSyntheticEntry({
            opType: 'delete',
            source: entry.path,
            destination: 'Recycle Bin',
            totalBytes: entry.size || 0,
            status: 'failed',
            error: String(e),
          });
        }
      }
      window.dispatchEvent(new CustomEvent('transfer:open'));
      loadDirectory(path, true);
    }

    clearSelection();
    refocusPanel();
  }

  function handleArchiveUp() {
    if (!archiveState) return;
    if (archiveState.internalPath === '') {
      // Exiting archive mode
      layout.clearArchiveState();
    } else {
      // Go up one level within archive
      const parts = archiveState.internalPath.split('/');
      parts.pop();
      layout.setArchiveInternalPath(parts.join('/'));
    }
  }

  async function handleArchiveExtract() {
    if (!archiveState) return;
    const entries = getEntriesToOperate(
      selectionState,
      files,
      displayFiles,
      selectedIndex,
      projectTree.getProjectMode(),
      projectTree.findTreeNode
    );
    if (entries.length === 0) return;
    const internalPaths = entries.map(e => e.path);
    const destDir = archiveState.archivePath.replace(/[\\/][^\\/]*$/, "");
    try {
      const extracted = await extractArchiveFiles(
        archiveState.archivePath,
        internalPaths,
        destDir
      );
      if (extracted === null) return;
      onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} extracted`);
    } catch (e) {
      onToast(`Extract failed: ${e}`);
    }
  }

  async function handleArchiveDelete() {
    if (!archiveState) return;
    if (archiveState.format !== 'zip') {
      onToast('Delete is only supported for ZIP archives');
      return;
    }
    const entries = getEntriesToOperate(
      selectionState,
      files,
      displayFiles,
      selectedIndex,
      projectTree.getProjectMode(),
      projectTree.findTreeNode
    );
    if (entries.length === 0) return;
    const internalPaths = entries.map(e => e.path);
    try {
      await deleteArchiveEntries(
        archiveState.archivePath,
        internalPaths
      );
      onToast(`${entries.length} ${entries.length === 1 ? 'entry' : 'entries'} deleted`);
      refresh();
    } catch (e) {
      onToast(`Delete failed: ${e}`);
    }
  }

  async function handleArchiveRename() {
    if (!archiveState) return;
    if (archiveState.format !== 'zip') {
      onToast('Rename is only supported for ZIP archives');
      return;
    }
    dialogs.startRename();
  }

  async function handleExtractHere(archivePath: string) {
    // Extract to the directory containing the archive file
    const destDir = archivePath.replace(/[\\/][^\\/]*$/, '') || path;
    try {
      const extracted = await extractArchive(
        archivePath,
        destDir
      );
      if (extracted === null) return;
      onToast('Archive extracted');
      refresh();
    } catch (e) {
      onToast(`Extract failed: ${e}`);
    }
  }

  async function handleCompress() {
    const entries = getEntriesToOperate(
      selectionState,
      files,
      displayFiles,
      selectedIndex,
      projectTree.getProjectMode(),
      projectTree.findTreeNode
    );
    if (entries.length === 0) return;
    const defaultName = entries.length === 1
      ? entries[0].name.replace(/\.\w+$/, '') + '.zip'
      : 'archive.zip';
    dialogs.startCompress(defaultName);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isFocused) return;
    if (dialogs.getShowDeleteConfirm() || dialogs.getInputVisible() || isSearchModalOpen || dialogs.getShowFileInfo()) {
      event.stopPropagation();
      return;
    }
    event.stopPropagation();
    if (['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(event.code)) return;

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;
    const isGSlash = lastKey === 'KeyG' && event.code === 'Slash' && now - lastKeyTime < 500;

    if (sortFilter.handleSortPrefix(event)) return;
    if (isArchiveMode) { handleArchiveKey(event, now); updateLastKey(event, now); return; }
    if (projectTree.getProjectMode() && handleTreeKey(event, isDoubleG, now)) { updateLastKey(event, now); return; }
    if (handleOperationKey(event, now, isGSlash)) { updateLastKey(event, now); return; }
    handleNavigationKey(event, isDoubleG);
    updateLastKey(event, now);
  }

  function updateLastKey(event: KeyboardEvent, now: number) {
    if (!['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(event.code)) {
      lastKey = event.code;
      lastKeyTime = now;
    }
  }


  function handleArchiveKey(event: KeyboardEvent, _now: number) {
    switch (event.key) {
      case 'Escape':
        if (sortFilter.getFilterPattern()) { event.preventDefault(); sortFilter.clearFilter(); }
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
          const entry = displayFiles[selectedIndex];
          if (entry.is_dir) {
            if (entry.name === '..') handleArchiveUp();
            else layout.setArchiveInternalPath(entry.path);
          } else onSelect(entry.path, entry.is_dir);
        }
        break;
      case 'R': event.preventDefault(); refresh(); break;
      case 'r': event.preventDefault(); handleArchiveRename(); break;
      case 'D': event.preventDefault(); handleDelete(true); break;
      case 'd': event.preventDefault(); handleArchiveDelete(); break;
      case 'x': event.preventDefault(); handleArchiveExtract(); break;
      case 'i': event.preventDefault(); dialogs.toggleFileInfo(); break;
      case 'f': event.preventDefault(); dialogs.startFilter('wildcard'); break;
      case 'o': event.preventDefault(); openSelectedFile(); break;
      case 'O': event.preventDefault(); openSelectedFileWith(); break;
      case 'a':
        event.preventDefault();
        setTimeout(() => { if (lastKey === 'KeyA') dialogs.startCreateFile(); }, 300);
        break;
      case '/':
        if (lastKey === 'KeyA' && Date.now() - lastKeyTime < 500) { event.preventDefault(); lastKey = ''; dialogs.startCreateDir(); }
        break;
      case 'A': event.preventDefault(); dialogs.startCreateDir(); break;
      default:
        switch (event.code) {
          case 'KeyJ': event.preventDefault(); selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1)); break;
          case 'KeyK': event.preventDefault(); selectByIndex(Math.max(selectedIndex - 1, 0)); break;
          case 'KeyG':
            if (lastKey === 'KeyG' && Date.now() - lastKeyTime < 500) { event.preventDefault(); selectByIndex(0); lastKey = ''; }
            else if (event.shiftKey) { event.preventDefault(); selectByIndex(displayFiles.length - 1); }
            break;
          case 'KeyH':
            event.preventDefault();
            if (archiveState && archiveState.internalPath) {
              const parts = archiveState.internalPath.split('/');
              pendingSelectName = parts[parts.length - 1] || null;
            } else if (archiveState) {
              pendingSelectName = archiveState.archivePath.split(/[/\\]/).pop() || null;
            }
            handleArchiveUp();
            break;
          case 'KeyL':
            event.preventDefault();
            if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
              const entry = displayFiles[selectedIndex];
              if (entry.is_dir) {
                if (entry.name === '..') handleArchiveUp();
                else layout.setArchiveInternalPath(entry.path);
              } else {
                if (isArchiveFile(entry.name)) onToast('Nested archives not supported in this phase');
                else onSelect(entry.path, entry.is_dir);
              }
            }
            break;
          case 'Space':
            event.preventDefault();
            if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
              const entry = displayFiles[selectedIndex];
              if (entry.name !== '..') selectionState = togglePathSelection(selectionState, entry.path);
              selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
            }
            break;
          case 'KeyY':
            event.preventDefault();
            {
              const entries = getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, projectTree.getProjectMode(), projectTree.findTreeNode);
              if (entries.length > 0 && archiveState) {
                clipboard.yankFromArchive(archiveState.archivePath, entries);
                layout.setMark('copy', entries.map(e => e.path));
                clearSelection();
                onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} yanked from archive`);
              }
            }
            break;
          case 'Period':
            event.preventDefault();
            showHidden = !showHidden;
            onToast(showHidden ? 'Showing hidden files' : 'Hiding hidden files');
            break;
        }
    }
  }

  function handleTreeKey(event: KeyboardEvent, isDoubleG: boolean, _now: number): boolean {
    const tvn = projectTree.getTreeVisibleNodes();
    switch (event.code) {
      case 'KeyJ':
        event.preventDefault();
        selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
        return true;
      case 'KeyK':
        event.preventDefault();
        if (event.shiftKey) {
          const parentPath = tvn[selectedIndex]?.parentPath;
          if (parentPath) projectTree.selectTreePathOrAncestor(parentPath);
        } else selectByIndex(Math.max(selectedIndex - 1, 0));
        return true;
      case 'KeyG':
        if (isDoubleG) { event.preventDefault(); selectByIndex(0); lastKey = ''; return true; }
        if (event.shiftKey) { event.preventDefault(); selectByIndex(displayFiles.length - 1); return true; }
        return false;
      case 'KeyH':
        event.preventDefault();
        if (event.shiftKey) { projectTree.collapseDeepestTreeLevel(); }
        else {
          const node = tvn[selectedIndex];
          if (node?.expanded) projectTree.collapseTreeNode(node);
          else if (node?.parentPath) { const parent = projectTree.findTreeNode(node.parentPath); if (parent?.expanded) projectTree.collapseTreeNode(parent); }
        }
        return true;
      case 'KeyL':
        event.preventDefault();
        {
          const node = tvn[selectedIndex];
          if (node?.entry.is_dir) {
            if (event.shiftKey) void projectTree.expandRecursively(node);
            else if (node.expanded && node.children.length > 0) selectByIndex(selectedIndex + 1);
            else void projectTree.expandTreeNode(node);
          } else if (node) onActivate(node.entry.path);
        }
        return true;
      case 'KeyV':
        event.preventDefault();
        if (checkHasSelection()) clearSelection();
        else {
          const root = projectTree.getProjectRoot();
          if (root) selectionState = selectTreeSubtree(selectionState, root.entry.path);
          else selectionState = { ...selectionState, selectedPaths: new Set(displayFiles.filter(f => f.name !== '..').map(f => f.path)) };
        }
        return true;
    }
    return false;
  }

  function handleOperationKey(event: KeyboardEvent, now: number, isGSlash: boolean): boolean {
    const isProjectMode = projectTree.getProjectMode();
    const tvn = projectTree.getTreeVisibleNodes();
    switch (event.key) {
      case 'Escape':
        if (sortFilter.getFilterPattern()) { event.preventDefault(); sortFilter.clearFilter(); return true; }
        return false;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
          const entry = displayFiles[selectedIndex];
          if (entry.is_dir) {
            if (isProjectMode) projectTree.toggleTreeNode(tvn[selectedIndex]);
            else onNavigate(entry.path);
          } else onActivate(entry.path);
        }
        return true;
      case 'R': event.preventDefault(); refresh(); return true;
      case 'r':
        event.preventDefault();
        {
          const selectedNonDirCount = isProjectMode
            ? getSelectedProjectNodes().filter(node => !node.entry.is_dir).length
            : files.filter(entry => !entry.is_dir && selectionState.selectedPaths.has(entry.path)).length;
          if (selectedNonDirCount > 1) startBatchRename();
          else dialogs.startRename();
        }
        return true;
      case 'D': event.preventDefault(); handleDelete(true); return true;
      case 'E':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
          const entry = displayFiles[selectedIndex];
          if (!entry.is_dir && isArchiveFile(entry.name)) {
            void markArchiveForExtraction(entry.path, (paths) => { layout.setMark('extract', paths); onToast(`Archive marked: ${paths[0]}`); }, (msg) => onToast(msg));
          } else onToast('E key only works on archive files');
        }
        return true;
      case 'e':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
          const entry = displayFiles[selectedIndex];
          if (!entry.is_dir && isArchiveFile(entry.name)) handleExtractHere(entry.path);
        }
        return true;
      case 'c': event.preventDefault(); handleCompress(); return true;
      case 'C':
        event.preventDefault();
        {
          const entries = getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, isProjectMode, projectTree.findTreeNode);
          if (entries.length > 0) {
            layout.setMark('compress', entries.map(e => e.path));
            clearSelection();
            onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} marked for compression. Press p to compress.`);
          }
        }
        return true;
      case 'i': event.preventDefault(); dialogs.toggleFileInfo(); return true;
      case 'f': event.preventDefault(); dialogs.startFilter('wildcard'); return true;
      case 'o': event.preventDefault(); openSelectedFile(); return true;
      case 'O': event.preventDefault(); openSelectedFileWith(); return true;
      default:
        switch (event.code) {
          case 'Slash':
            event.preventDefault();
            if (lastKey === 'KeyA' && now - lastKeyTime < 500) { lastKey = ''; dialogs.startCreateDir(); }
            else if (isGSlash) { searchMode = 'recursive'; lastKey = ''; openSearchModal(); }
            else { searchMode = 'current'; openSearchModal(); }
            return true;
          case 'Space':
            event.preventDefault();
            if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
              const entry = displayFiles[selectedIndex];
              if (entry.name !== '..') {
                const node = isProjectMode ? tvn[selectedIndex] : null;
                if (node) selectionState = toggleTreeSelection(selectionState, node.entry.path, node.entry.is_dir);
                else selectionState = togglePathSelection(selectionState, entry.path);
              }
              selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
            }
            return true;
          case 'KeyV':
            event.preventDefault();
            if (checkHasSelection()) clearSelection();
            else {
              const root = projectTree.getProjectRoot();
              if (isProjectMode && root) selectionState = selectTreeSubtree(selectionState, root.entry.path);
              else selectionState = { ...selectionState, selectedPaths: new Set(displayFiles.filter(f => f.name !== '..').map(f => f.path)) };
            }
            return true;
          case 'KeyY':
            event.preventDefault();
            {
              const entries = getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, isProjectMode, projectTree.findTreeNode);
              if (entries.length > 0) {
                clipboard.yank(entries);
                layout.setMark('copy', entries.map(e => e.path));
                clearSelection();
                onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} yanked`);
              }
            }
            return true;
          case 'KeyX':
            event.preventDefault();
            {
              const entries = getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, isProjectMode, projectTree.findTreeNode);
              if (entries.length > 0) {
                clipboard.cut(entries);
                layout.setMark('cut', entries.map(e => e.path));
                clearSelection();
                onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} cut`);
              }
            }
            return true;
          case 'Period':
            event.preventDefault();
            showHidden = !showHidden;
            if (isProjectMode) void projectTree.refreshProjectTree();
            onToast(showHidden ? 'Showing hidden files' : 'Hiding hidden files');
            return true;
          case 'KeyA':
            event.preventDefault();
            setTimeout(() => { if (lastKey === 'KeyA') dialogs.startCreateFile(); }, 300);
            return true;
          case 'KeyD':
            event.preventDefault();
            handleDelete(false);
            return true;
        }
    }
    return false;
  }

  function handleNavigationKey(event: KeyboardEvent, isDoubleG: boolean) {
    switch (event.code) {
      case 'KeyJ':
        event.preventDefault();
        selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
        break;
      case 'KeyK':
        event.preventDefault();
        selectByIndex(Math.max(selectedIndex - 1, 0));
        break;
      case 'KeyG':
        if (isDoubleG) { event.preventDefault(); selectByIndex(0); lastKey = ''; }
        else if (event.shiftKey) { event.preventDefault(); selectByIndex(displayFiles.length - 1); }
        break;
      case 'KeyH':
        if (type === 'current' || type === 'parent') {
          event.preventDefault();
          const shouldRestore = isFocused;
          const dirName = path.split(/[/\\]/).filter(Boolean).pop();
          if (dirName && type === 'current') pendingSelectName = dirName;
          onNavigateUp();
          if (shouldRestore) setTimeout(() => { panelElement?.focus(); isFocused = true; }, 100);
        }
        break;
      case 'KeyL':
        if (type === 'current' || detached || type === 'parent') {
          event.preventDefault();
          if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
            const entry = displayFiles[selectedIndex];
            if (entry.is_dir) onNavigate(entry.path);
            else {
              if (isArchiveFile(entry.name)) enterArchive(entry.path, layout);
              else onActivate(entry.path);
            }
          }
        }
        break;
    }
  }

  function isTreeToggleHit(event: MouseEvent, index: number): boolean {
    const node = projectTree.getTreeVisibleNodes()[index];
    if (!projectTree.getProjectMode() || !node?.entry.is_dir) return false;
    const row = event.currentTarget as HTMLElement;
    const toggle = row.querySelector<HTMLButtonElement>('.tree-toggle');
    if (!toggle) return false;
    const bounds = toggle.getBoundingClientRect();
    const hitPadding = 8;
    return event.clientX >= bounds.left - hitPadding
      && event.clientX <= bounds.right + hitPadding
      && event.clientY >= bounds.top - hitPadding
      && event.clientY <= bounds.bottom + hitPadding;
  }

  function handleItemClick(index: number, event: MouseEvent) {
    const node = projectTree.getTreeVisibleNodes()[index];
    if (node && isTreeToggleHit(event, index)) {
      panelElement?.focus();
      projectTree.toggleTreeNode(node);
      return;
    }
    selectByIndex(index);
  }

  function handleItemDblClick(entry: FileEntry, event: MouseEvent, index: number) {
    if (isTreeToggleHit(event, index)) return;
    if (entry.is_dir) {
      if (projectTree.getProjectMode()) {
        const node = projectTree.findTreeNode(entry.path);
        if (node) projectTree.toggleTreeNode(node);
      } else onNavigate(entry.path);
    } else {
      onActivate(entry.path);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="directory-panel"
  bind:this={panelElement}
  onkeydown={handleKeydown}
  onfocus={handleFocus}
  onblur={handleBlur}
  onclick={(e) => (e.currentTarget as HTMLDivElement).focus()}
  role="tree"
  aria-label="{type === 'parent' ? 'Parent Directory' : 'Current Directory'}"
  aria-activedescendant={projectTree.getProjectMode() && selectedIndex >= 0 ? `project-tree-${type}-${selectedIndex}` : undefined}
  tabindex="0"
>
  <div class="panel-header">
    {#if isArchiveMode && archiveState}
      {#if detached}
        <span class="detach-marker" title="Manual mode (detached)">&#9679;</span>
      {/if}
      <span class="panel-path archive-path" title={archiveState.archivePath}>
        {archiveState.archivePath.split('\\').pop() || archiveState.archivePath}
        {#if archiveState.internalPath}
          <span class="archive-internal"> / {archiveState.internalPath.replace(/\//g, ' / ')}</span>
        {/if}
      </span>
    {:else if path}
      {#if detached}
        <span class="detach-marker" title="Manual mode (detached)">&#9679;</span>
      {/if}
      <span class="panel-path" title={path}>{path === '/' ? '/' : path.split('\\').pop() || path.split('/').pop() || path}</span>
    {/if}
    {#if sortFilter.getFilterPattern()}
      <span class="filter-badge" title={sortFilter.getFilterPattern()}>{sortFilter.getFilterPattern()}</span>
    {/if}
  </div>

  <div class="panel-content">
    <InputDialog
      visible={dialogs.getInputVisible()}
      value={dialogs.getInputValue()}
      placeholder={dialogs.getInputPlaceholder()}
      prompt={dialogs.getInputPrompt()}
      onConfirm={dialogs.handleInputConfirm}
      onCancel={dialogs.handleInputCancel}
    />
    {#if isLoading}
      <p class="placeholder">Loading...</p>
    {:else if errorMessage}
      <p class="error">{errorMessage}</p>
    {:else if displayFiles.length === 0}
      <p class="placeholder">Empty directory</p>
    {:else}
      {#if projectTree.getProjectMode()}
        <ProjectTreePanel
          visibleNodes={projectTree.getTreeVisibleNodes()}
          selectedIndex={selectedIndex}
          showHidden={showHidden}
          onSelect={selectByIndex}
          onToggle={projectTree.toggleTreeNode}
          onDblClick={(node, event) => handleItemDblClick(node.entry, event, projectTree.getTreeVisibleNodes().indexOf(node))}
          cutPaths={cutPaths}
          isTreeNodeSelected={(node) => isTreeNodeSelected(selectionState, node.entry.path)}
          type={type}
        />
      {:else}
        <FileListPanel
          files={displayFiles}
          selectedIndex={selectedIndex}
          cutPaths={cutPaths}
          selectionState={selectionState}
          onSelect={selectByIndex}
          onDblClick={(entry, _index, _event) => {
            if (entry.is_dir) onNavigate(entry.path);
            else onActivate(entry.path);
          }}
        />
      {/if}
    {/if}
  </div>

  <SearchModal
    visible={isSearchModalOpen}
    rootPath={projectTree.getProjectMode() && projectTree.getProjectRoot() ? projectTree.getProjectRoot()!.entry.path : path}
    mode={searchMode}
    allowRecursive={!path.startsWith('ftp://')}
    entries={path.startsWith('ftp://') ? files : null}
    onClose={closeSearchModal}
    onSelect={handleSearchSelect}
  />

  <ConfirmModal
    visible={dialogs.getShowDeleteConfirm()}
    title={dialogs.getDeleteIsPermanent() ? 'Permanent Delete' : 'Move to Trash'}
    fileName={dialogs.getDeleteIsPermanent()
      ? `Permanently delete ${getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, projectTree.getProjectMode(), projectTree.findTreeNode).length} item(s)?`
      : `Move ${getEntriesToOperate(selectionState, files, displayFiles, selectedIndex, projectTree.getProjectMode(), projectTree.findTreeNode).length} item(s) to trash?`}
    buttons={[
      { key: 'D', label: 'elete', action: dialogs.handleDeleteConfirm, style: 'danger' },
      { key: 'C', label: 'ancel', action: dialogs.handleDeleteCancel },
    ]}
  />

  <FileInfoPanel
    visible={dialogs.getShowFileInfo()}
    info={dialogs.getFileInfo()}
    onClose={dialogs.closeFileInfo}
  />

</div>

<style>
  .directory-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-primary);
    outline: none;
    font-family: var(--font-mono);
  }

  .panel-header {
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .detach-marker {
    font-size: 8px;
    color: var(--accent);
    margin-right: 4px;
    flex-shrink: 0;
  }

  .panel-path {
    font-size: 11px;
    color: var(--accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .filter-badge {
    font-size: 10px;
    color: var(--bg-primary);
    background-color: var(--accent);
    padding: 0 6px;
    margin-left: auto;
    flex-shrink: 0;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 2px 0;
    overflow-anchor: none;
  }

  .file-list {
    font-size: 13px;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 2px 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
    user-select: none;
  }

  .file-item:hover {
    background-color: var(--bg-hover);
  }

  .file-item.selected {
    background-color: var(--bg-active);
  }

  .file-item.multi-selected {
    background-color: color-mix(in srgb, var(--accent) 18%, var(--bg-primary));
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .file-item.selected.multi-selected {
    background-color: var(--bg-active);
    box-shadow: inset 3px 0 0 var(--accent), inset 0 0 0 1px var(--accent);
  }

  .file-item.cut-marked {
    opacity: 0.5;
  }

  .file-item.hidden-file {
    opacity: 0.6;
  }

  .cut-marker {
    display: inline-block;
    width: 12px;
    font-size: 11px;
    color: var(--error);
    flex-shrink: 0;
    text-align: center;
    font-weight: bold;
  }

  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--file-color);
  }

  .file-name.is-dir {
    color: var(--dir-color);
    font-weight: 500;
  }

  .tree-indent,
  .tree-toggle {
    flex-shrink: 0;
  }

  .tree-indent {
    align-self: stretch;
    background-image: repeating-linear-gradient(
      to right,
      transparent 0,
      transparent 7px,
      var(--border) 7px,
      var(--border) 8px,
      transparent 8px,
      transparent 16px
    );
    opacity: 0.65;
  }

  .tree-toggle {
    width: 28px;
    margin-left: -8px;
    margin-right: 0;
    position: relative;
    border: 0;
    padding: 0;
    background: transparent;
  }

  .tree-toggle::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 0;
    width: 8px;
    border-top: 1px solid var(--border);
    opacity: 0.65;
  }

  .tree-toggle.directory-toggle::after {
    content: '';
    position: absolute;
    top: calc(50% - 3px);
    left: 10px;
    width: 7px;
    height: 7px;
    border-right: 1px solid var(--text-muted);
    border-bottom: 1px solid var(--text-muted);
    transform: rotate(-45deg);
    transition: transform 0.1s ease;
  }

  .tree-toggle.directory-toggle.expanded::after {
    transform: rotate(45deg);
  }

  .tree-toggle.directory-toggle {
    cursor: pointer;
  }

  .placeholder {
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
    margin-top: 40px;
  }

  .error {
    color: var(--error);
    font-size: 13px;
    text-align: center;
    margin-top: 40px;
  }
</style>
