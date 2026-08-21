<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy, untrack } from 'svelte';
  import { layout } from '$lib/stores/layout';
  import { clipboard, type ClipboardEntry } from '$lib/stores/clipboard';
  import { transfer } from '$lib/stores/transfer';
  import SearchModal from './SearchModal.svelte';
  import InputDialog from './InputDialog.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import FileInfoPanel from './FileInfoPanel.svelte';
  import { directoryCache } from '$lib/utils/directory-cache';
  import { normalizeDirectoryKey, type DirectoryKey } from '$lib/utils/directory-refresh';

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size: number | null;
    is_hidden?: boolean;
    modified?: number | null;
    created?: number | null;
  }

  interface TreeNode {
    entry: FileEntry;
    depth: number;
    parentPath: string | null;
    expanded: boolean;
    loaded: boolean;
    loading: boolean;
    error: string;
    children: TreeNode[];
  }

  export interface ProjectTreeState {
    enabled: boolean;
    rootPath: string | null;
    expandedPaths: string[];
    selectedPath: string | null;
    scrollOffset: number;
  }

  let {
    type = 'current',
    path = '',
    selectedPath = null,
    detached = false,
    onNavigate = (path: string) => {},
    onSelect = (path: string) => {},
    onActivate = (path: string) => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onFullscreen = () => {},
    onNavigateUp = () => {},
    onTabCommand = (cmd: string) => {},
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
    onSelect?: (path: string) => void;
    onActivate?: (path: string) => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onFullscreen?: () => void;
    onNavigateUp?: () => void;
    onTabCommand?: (cmd: string) => void;
    onToast?: (message: string) => void;
    onBatchRenameStart?: (files: { path: string; name: string }[]) => void;
    getDirectoryVersion?: (directory: DirectoryKey) => number;
    onDirectorySynchronized?: (directory: DirectoryKey, version: number) => void;
  } = $props();

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
  let selectedPaths: Set<string> = $state(new Set());

  // Hidden files toggle
  let showHidden: boolean = $state(false);

  // Sort state
  let sortBy: 'name' | 'size' | 'ext' | 'modified' | 'created' = $state('name');
  let sortReverse: boolean = $state(false);
  let dirFirst: boolean = $state(true);
  let sortPrefixPending: boolean = $state(false);
  let sortPrefixTimeout: ReturnType<typeof setTimeout> | null = null;

  // Filter state
  let filterPattern: string = $state('');
  let filterMode: 'prefix' | 'wildcard' = $state('prefix');
  let projectMode: boolean = $state(false);
  let projectRoot: TreeNode | null = $state(null);
  let projectRestoreGeneration = 0;

  // Derived values that depend on state declared above
  let normalDisplayFiles: FileEntry[] = $derived.by(() => {
    let result = showHidden ? files : files.filter(f => f.name === '..' || !f.is_hidden);

    // Apply filter
    if (filterPattern) {
      if (filterMode === 'prefix') {
        const lower = filterPattern.toLowerCase();
        result = result.filter(f => f.name === '..' || f.name.toLowerCase().startsWith(lower));
      } else {
        const pattern = filterPattern.replace(/\*/g, '.*').replace(/\?/g, '.');
        const regex = new RegExp(`^${pattern}$`, 'i');
        result = result.filter(f => f.name === '..' || regex.test(f.name));
      }
    }

    // Apply sort (.. always stays first)
    const dotdot = result.filter(f => f.name === '..');
    const rest = result.filter(f => f.name !== '..');

    rest.sort((a, b) => {
      // dirFirst: directories before files
      if (dirFirst && a.is_dir !== b.is_dir) {
        return a.is_dir ? -1 : 1;
      }

      let cmp = 0;
      if (sortBy === 'name') {
        cmp = a.name.localeCompare(b.name, undefined, { numeric: true });
      } else if (sortBy === 'size') {
        cmp = (a.size ?? 0) - (b.size ?? 0);
      } else if (sortBy === 'ext') {
        const extA = a.name.split('.').pop()?.toLowerCase() || '';
        const extB = b.name.split('.').pop()?.toLowerCase() || '';
        cmp = extA.localeCompare(extB);
      } else if (sortBy === 'modified') {
        cmp = (a.modified ?? 0) - (b.modified ?? 0);
      } else if (sortBy === 'created') {
        cmp = (a.created ?? 0) - (b.created ?? 0);
      }

      return sortReverse ? -cmp : cmp;
    });

    return [...dotdot, ...rest];
  });
  let treeVisibleNodes: TreeNode[] = $derived.by(() => {
    if (!projectMode || !projectRoot) return [];
    const result: TreeNode[] = [];
    const visit = (node: TreeNode) => {
      result.push(node);
      if (node.expanded) node.children.forEach(visit);
    };
    visit(projectRoot);
    return result;
  });
  let displayFiles: FileEntry[] = $derived(projectMode ? treeVisibleNodes.map(node => node.entry) : normalDisplayFiles);
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

  // InputDialog state
  let inputVisible: boolean = $state(false);
  let inputValue: string = $state('');
  let inputPlaceholder: string = $state('');
  let inputPrompt: string = $state('');
  let inputMode: 'rename' | 'create-file' | 'create-dir' | 'filter' = $state('rename');

  // Delete confirmation state
  let showDeleteConfirm: boolean = $state(false);
  let deleteIsPermanent: boolean = $state(false);
  let deleteResolve: ((confirm: boolean) => void) | null = null;

  // File info state
  let showFileInfo: boolean = $state(false);
  let fileInfo: any = $state(null);

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

  export function setPendingRestore(cursorIndex: number, scrollOffset: number) {
    pendingCursorIndex = cursorIndex;
    pendingScrollOffset = scrollOffset;
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
      panelElement.focus();
      isFocused = true;
    }
  }

  export function getDirectoryKey(): DirectoryKey {
    return normalizeDirectoryKey(path);
  }

  export function refresh() {
    if (projectMode) return refreshProjectTree();
    return loadDirectory(path, true);
  }

  export function synchronize(version: number) {
    if (projectMode) return refreshProjectTree(version);
    return loadDirectory(path, false, version);
  }

  export function getProjectTreeState(): ProjectTreeState {
    return {
      enabled: projectMode,
      rootPath: projectRoot?.entry.path ?? null,
      expandedPaths: treeVisibleNodes.filter(node => node.expanded && node !== projectRoot).map(node => node.entry.path),
      selectedPath: selectedFile?.path ?? null,
      scrollOffset: getScrollOffset(),
    };
  }

  export async function setProjectMode(enabled: boolean, state?: ProjectTreeState): Promise<boolean> {
    if (!enabled) {
      projectMode = false;
      projectRoot = null;
      return true;
    }
    const rootPath = state?.rootPath ?? path;
    if (!rootPath || rootPath === '/' || rootPath === '\\' || rootPath.startsWith('ftp://')) {
      onToast('Project mode requires a local directory');
      return false;
    }
    const generation = ++projectRestoreGeneration;
    projectMode = true;
    projectRoot = { entry: { name: rootPath.split(/[/\\]/).filter(Boolean).pop() || rootPath, path: rootPath, is_dir: true, size: null }, depth: 0, parentPath: null, expanded: true, loaded: false, loading: false, error: '', children: [] };
    await loadTreeChildren(projectRoot, generation);
    if (generation !== projectRestoreGeneration) return false;
    for (const expandedPath of state?.expandedPaths ?? []) {
      const node = findTreeNode(expandedPath);
      if (node?.entry.is_dir) await expandTreeNode(node, generation);
    }
    if (state?.selectedPath) selectTreePathOrAncestor(state.selectedPath);
    else selectByIndex(0);
    if (state?.scrollOffset) setScrollOffset(state.scrollOffset);
    return true;
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
      selectedPaths = new Set();
      untrack(() => loadDirectory(path, false));
    }
  });

  // Subscribe to clipboard for cut file indicators
  onMount(() => {
    clipboardUnsub = clipboard.subscribe(state => {
      if (state.operation === 'cut') {
        cutPaths = new Set(state.entries.map(e => e.path));
      } else {
        cutPaths = new Set();
      }
    });
  });

  onDestroy(() => {
    if (clipboardUnsub) { clipboardUnsub(); clipboardUnsub = null; }
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
    if (selectedIndex >= 0 || displayFiles.length === 0) return;
    // If navigating back, try to highlight the directory we came from
    if (pendingSelectName) {
      const target = pendingSelectName;
      pendingSelectName = null;
      const idx = displayFiles.findIndex(f => f.name === target);
      if (idx >= 0) {
        selectedIndex = idx;
        selectedPathInternal = displayFiles[idx].path;
        onSelect(displayFiles[idx].path);
        return;
      }
    }
    // If selectedPath prop is set (e.g. from tab restore), try to use it
    if (selectedPath) {
      const idx = displayFiles.findIndex(f => f.path === selectedPath);
      if (idx >= 0) {
        selectedIndex = idx;
        selectedPathInternal = displayFiles[idx].path;
        onSelect(displayFiles[idx].path);
        return;
      }
    }
    const firstReal = displayFiles.findIndex(f => f.name !== '..');
    selectedIndex = firstReal >= 0 ? firstReal : 0;
    selectedPathInternal = displayFiles[selectedIndex].path;
    onSelect(displayFiles[selectedIndex].path);
  }

  let loadingGen = 0;

  function markDirectorySynchronized(dirPath: string, version?: number) {
    const directory = normalizeDirectoryKey(dirPath);
    onDirectorySynchronized(directory, version ?? getDirectoryVersion(directory));
  }

  async function loadDirectory(dirPath: string, forceRefresh: boolean = false, version?: number): Promise<boolean> {
    const gen = ++loadingGen;
    isLoading = true;
    errorMessage = '';

    const isVirtual = isVirtualRoot(dirPath);

    // Check cache first
    if (!forceRefresh && directoryCache.has(dirPath)) {
      files = directoryCache.get(dirPath)!;
      // Inject .. for non-root directories (virtual root has no ..)
      if (!isVirtual) {
        const parentPath = getParentPath(dirPath);
        files = [{ name: '..', path: parentPath, is_dir: true, size: null }, ...files];
      }
      // Safety dedup
      const seen = new Set<string>();
      files = files.filter(f => { if (seen.has(f.path)) return false; seen.add(f.path); return true; });
      if (gen !== loadingGen) return false;
      selectInitialEntry();
      applyPendingRestore();
      markDirectorySynchronized(dirPath, version);
      isLoading = false;
      return true;
    }

    try {
      if (isVirtual) {
        // Virtual root: list drives
        files = await invoke<FileEntry[]>('list_drives');
      } else {
        files = await invoke<FileEntry[]>('read_directory', { path: dirPath });
        files.sort((a, b) => {
          if (a.is_dir && !b.is_dir) return -1;
          if (!a.is_dir && b.is_dir) return 1;
          return a.name.localeCompare(b.name);
        });
        // Inject .. for parent directory navigation
        const parentPath = getParentPath(dirPath);
        files = [{ name: '..', path: parentPath, is_dir: true, size: null }, ...files];
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

  function findTreeNode(nodePath: string, node: TreeNode | null = projectRoot): TreeNode | null {
    if (!node) return null;
    if (node.entry.path === nodePath) return node;
    for (const child of node.children) {
      const found = findTreeNode(nodePath, child);
      if (found) return found;
    }
    return null;
  }

  async function loadTreeChildren(node: TreeNode, generation: number, forceRefresh: boolean = false): Promise<void> {
    if (!node.entry.is_dir || node.loading || (node.loaded && !forceRefresh)) return;
    node.loading = true;
    node.error = '';
    try {
      const entries = !forceRefresh && directoryCache.has(node.entry.path)
        ? directoryCache.get(node.entry.path)!
        : await invoke<FileEntry[]>('read_directory', { path: node.entry.path });
      if (generation !== projectRestoreGeneration) return;
      directoryCache.set(node.entry.path, entries);
      node.children = entries
        .filter(entry => showHidden || !entry.is_hidden)
        .sort((a, b) => a.is_dir === b.is_dir ? a.name.localeCompare(b.name, undefined, { numeric: true }) : a.is_dir ? -1 : 1)
        .map(entry => ({ entry, depth: node.depth + 1, parentPath: node.entry.path, expanded: false, loaded: false, loading: false, error: '', children: [] }));
      node.loaded = true;
    } catch (error) {
      node.error = String(error);
    } finally {
      node.loading = false;
      projectRoot = projectRoot ? { ...projectRoot } : null;
    }
  }

  async function expandTreeNode(node: TreeNode, generation: number = projectRestoreGeneration): Promise<void> {
    if (!node.entry.is_dir) return;
    await loadTreeChildren(node, generation);
    if (generation === projectRestoreGeneration) {
      node.expanded = true;
      projectRoot = projectRoot ? { ...projectRoot } : null;
    }
  }

  function selectTreePathOrAncestor(nodePath: string): void {
    let candidate: string | null = nodePath;
    while (candidate) {
      const index = treeVisibleNodes.findIndex(node => node.entry.path === candidate);
      if (index >= 0) { selectByIndex(index); return; }
      const node = findTreeNode(candidate);
      candidate = node?.parentPath ?? null;
    }
    selectByIndex(0);
  }

  async function refreshProjectTree(version?: number): Promise<boolean> {
    const state = getProjectTreeState();
    if (state.rootPath) directoryCache.invalidate(normalizeDirectoryKey(state.rootPath));
    state.expandedPaths.forEach(nodePath => directoryCache.invalidate(normalizeDirectoryKey(nodePath)));
    const result = await setProjectMode(true, state);
    if (result && projectRoot) markDirectorySynchronized(projectRoot.entry.path, version);
    return result;
  }

  async function expandRecursively(node: TreeNode): Promise<void> {
    const queue: TreeNode[] = [node];
    let visited = 0;
    const limit = 500;
    while (queue.length > 0 && visited < limit) {
      const current = queue.shift()!;
      await expandTreeNode(current);
      visited += 1;
      queue.push(...current.children.filter(child => child.entry.is_dir));
    }
    if (queue.length > 0) onToast(`Project tree expansion stopped after ${limit} directories`);
  }

  function collapseDeepestTreeLevel(): void {
    const deepest = treeVisibleNodes.filter(node => node !== projectRoot && node.expanded)
      .sort((a, b) => b.depth - a.depth)[0];
    if (deepest) {
      deepest.expanded = false;
      projectRoot = projectRoot ? { ...projectRoot } : null;
    }
  }

  function selectByIndex(index: number) {
    if (index >= 0 && index < displayFiles.length) {
      selectedIndex = index;
      selectedPathInternal = displayFiles[index].path;

      // Debounce onSelect to avoid rapid file loading
      if (selectTimeout) {
        clearTimeout(selectTimeout);
      }
      const capturedIndex = index;
      selectTimeout = setTimeout(() => {
        // Only fire if user is still on the same item
        if (selectedIndex === capturedIndex) {
          onSelect(displayFiles[capturedIndex].path);
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

  function handleSearchSelect(filePath: string, isDir: boolean) {
    if (isDir) {
      onNavigate(filePath);
    } else {
      onSelect(filePath);
    }
  }

  function startRename() {
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    inputMode = 'rename';
    inputValue = entry.name;
    inputPlaceholder = '';
    inputPrompt = 'Rename:';
    inputVisible = true;
  }

  function startCreateFile() {
    inputMode = 'create-file';
    inputValue = '';
    inputPlaceholder = 'New file name';
    inputPrompt = 'New file:';
    inputVisible = true;
  }

  function startCreateDir() {
    inputMode = 'create-dir';
    inputValue = '';
    inputPlaceholder = 'New directory name';
    inputPrompt = 'New dir:';
    inputVisible = true;
  }

  function startBatchRename() {
    const entries = files.filter(f => selectedPaths.has(f.path));
    if (entries.length < 2) return;
    const renameFiles = entries.map(f => ({ path: f.path, name: f.name }));
    onBatchRenameStart(renameFiles);
  }

  async function handleInputConfirm(value: string) {
    inputVisible = false;
    try {
      if (inputMode === 'rename') {
        const entry = displayFiles[selectedIndex];
        if (entry.path.startsWith('ftp://')) {
          const parentBase = path.replace(/\/+$/, '');
          const newPath = await invoke<string>('ftp_rename', { oldPath: entry.path, newPath: parentBase + '/' + value });
          loadDirectory(path, true);
          onSelect(newPath);
          onToast(`Renamed to ${value}`);
        } else {
          const parentPath = path.replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('rename_file', { oldPath: entry.path, newName: value });
          await currentDirectoryPanel_refresh();
          onSelect(newPath);
          onToast(`Renamed to ${value}`);
        }
      } else if (inputMode === 'create-file') {
        if (path.startsWith('ftp://')) {
          const remotePath = path.replace(/\/+$/, '') + '/' + value;
          await invoke('ftp_create_file', { path: remotePath });
          loadDirectory(path, true);
          onToast(`Created ${value}`);
        } else {
          const parentPath = path.replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('create_file', { path: newPath, isDir: false });
          await currentDirectoryPanel_refresh();
          onSelect(newPath);
          onToast(`Created ${value}`);
        }
      } else if (inputMode === 'create-dir') {
        if (path.startsWith('ftp://')) {
          const remotePath = path.replace(/\/+$/, '') + '/' + value;
          await invoke('ftp_mkdir', { path: remotePath });
          loadDirectory(path, true);
          onToast(`Created ${value}/`);
        } else {
          const parentPath = path.replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('create_file', { path: newPath, isDir: true });
          await currentDirectoryPanel_refresh();
          onSelect(newPath);
          onToast(`Created ${value}/`);
        }
      } else if (inputMode === 'filter') {
        filterPattern = value;
        if (value) {
          onToast(`Filter: ${value}`);
        } else {
          onToast('Filter cleared');
        }
      }
    } catch (e) {
      onToast(`Error: ${e}`);
    }
    // Restore focus
    setTimeout(() => panelElement?.focus(), 0);
  }

  function handleInputCancel() {
    inputVisible = false;
    setTimeout(() => panelElement?.focus(), 0);
  }

  async function toggleFileInfo() {
    if (showFileInfo) {
      showFileInfo = false;
      fileInfo = null;
      return;
    }
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    try {
      fileInfo = await invoke('get_file_info', { path: entry.path });
      showFileInfo = true;
    } catch (e) {
      onToast(`Failed to get file info: ${e}`);
    }
  }

  function closeFileInfo() {
    showFileInfo = false;
    fileInfo = null;
    setTimeout(() => panelElement?.focus(), 0);
  }

  function setSort(mode: 'name' | 'size' | 'ext' | 'modified' | 'created', reverse: boolean = false) {
    sortBy = mode;
    sortReverse = reverse;
    const labels: Record<string, string> = { name: 'name', size: 'size', ext: 'extension', modified: 'modified time', created: 'created time' };
    onToast(`Sorted by ${labels[mode] || mode}${reverse ? ' (reversed)' : ''}`);
  }

  function toggleDirFirst() {
    dirFirst = !dirFirst;
    onToast(dirFirst ? 'Directories first' : 'Mixed order');
  }

  function startFilter(mode: 'prefix' | 'wildcard' = 'wildcard') {
    filterMode = mode;
    inputMode = 'filter';
    inputValue = filterPattern;
    if (mode === 'prefix') {
      inputPlaceholder = 'Enter prefix to filter...';
      inputPrompt = 'Prefix:';
    } else {
      inputPlaceholder = '*.txt, *.rs, *.{js,ts}';
      inputPrompt = 'Filter:';
    }
    inputVisible = true;
  }

  function clearFilter() {
    filterPattern = '';
    onToast('Filter cleared');
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

  function promptDelete(permanent: boolean): Promise<boolean> {
    return new Promise(resolve => {
      deleteIsPermanent = permanent;
      showDeleteConfirm = true;
      deleteResolve = resolve;
    });
  }

  function handleDeleteConfirm() {
    showDeleteConfirm = false;
    deleteResolve?.(true);
    deleteResolve = null;
    refocusPanel();
  }

  function handleDeleteCancel() {
    showDeleteConfirm = false;
    deleteResolve?.(false);
    deleteResolve = null;
    refocusPanel();
  }

  function refocusPanel() {
    setTimeout(() => {
      if (panelElement) panelElement.focus();
    }, 0);
  }

  async function handleDelete(permanent: boolean) {
    const entries = getEntriesToOperate();
    if (entries.length === 0) return;

    const confirmed = await promptDelete(permanent);
    if (!confirmed) return;

    // Route through TransferManager for unified progress display
    const tasks = entries.map((entry: ClipboardEntry) => {
      const isFtp = entry.path.startsWith('ftp://');
      return {
        op_type: 'delete' as const,
        source: entry.path,
        destination: '', // Delete has no destination
        total_bytes: entry.size || 0,
        conn_name: isFtp ? entry.path.slice(6).split('/')[0] : undefined,
      };
    });
    const ids = await transfer.enqueueTransfers(tasks);
    window.dispatchEvent(new CustomEvent('transfer:open'));

    selectedPaths = new Set();
    onToast(`Deleting ${ids.length} ${ids.length === 1 ? 'file' : 'files'}...`);
  }

  function getEntriesToOperate(): ClipboardEntry[] {
    if (selectedPaths.size > 0) {
      return files
        .filter(f => f.name !== '..' && selectedPaths.has(f.path))
        .map(f => ({ path: f.path, name: f.name, is_dir: f.is_dir, size: f.size ?? undefined }));
    }
    if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
      const f = displayFiles[selectedIndex];
      if (f.name === '..') return [];
      return [{ path: f.path, name: f.name, is_dir: f.is_dir, size: f.size ?? undefined }];
    }
    return [];
  }

  function handleKeydown(event: KeyboardEvent) {
    // Only handle if this panel has focus
    if (!isFocused) {
      return;
    }

    // Don't intercept keys when a modal or dialog is open
    if (showDeleteConfirm || inputVisible || isSearchModalOpen || showFileInfo) {
      event.stopPropagation();
      return;
    }

    // Stop event propagation to prevent other panels from handling
    event.stopPropagation();

    // Ignore standalone modifier key presses
    if (['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(event.code)) {
      return;
    }

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;
    const isGSlash = lastKey === 'KeyG' && event.code === 'Slash' && now - lastKeyTime < 500;
    // Sort prefix state machine (s + sub-key)
    const codeMap: Record<string, string> = {
      KeyN: 'name', KeyS: 'size', KeyE: 'ext',
      KeyM: 'modified', KeyC: 'created', KeyT: 'dirfirst',
    };
    if (sortPrefixPending && codeMap[event.code]) {
      event.preventDefault();
      sortPrefixPending = false;
      if (sortPrefixTimeout) { clearTimeout(sortPrefixTimeout); sortPrefixTimeout = null; }
      const mode = codeMap[event.code];
      if (mode === 'dirfirst') {
        toggleDirFirst();
      } else {
        setSort(mode as any, event.shiftKey);
      }
      return;
    }
    if (sortPrefixPending) {
      sortPrefixPending = false;
      if (sortPrefixTimeout) { clearTimeout(sortPrefixTimeout); sortPrefixTimeout = null; }
    }
    if (event.code === 'KeyS' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      sortPrefixPending = true;
      if (sortPrefixTimeout) clearTimeout(sortPrefixTimeout);
      sortPrefixTimeout = setTimeout(() => { sortPrefixPending = false; }, 1000);
      return;
    }

    switch (event.key) {
      case 'Escape':
        if (filterPattern) {
          event.preventDefault();
          clearFilter();
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
          const entry = displayFiles[selectedIndex];
          if (entry.is_dir) {
            if (projectMode) void expandTreeNode(treeVisibleNodes[selectedIndex]);
            else onNavigate(entry.path);
          } else {
            onActivate(entry.path);
          }
        }
        break;
      case 'R':
        event.preventDefault();
        if (path) loadDirectory(path, true); // Force refresh
        break;
      case 'r':
        event.preventDefault();
        if (selectedPaths.size > 1) {
          startBatchRename();
        } else {
          startRename();
        }
        break;
      case 'D':
        event.preventDefault();
        handleDelete(true);
        break;
      case 'E':
        event.preventDefault();
        onFullscreen();
        break;
      case 'i':
        event.preventDefault();
        toggleFileInfo();
        break;
      case 'f':
        event.preventDefault();
        startFilter('wildcard');
        break;
      case 'o':
        event.preventDefault();
        openSelectedFile();
        break;
      case 'O':
        event.preventDefault();
        openSelectedFileWith();
        break;
      default:
        // Use event.code for letter keys to support Chinese IME
        switch (event.code) {
          case 'KeyJ':
            event.preventDefault();
            selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
            break;
          case 'KeyK':
            event.preventDefault();
            if (projectMode) {
              if (event.shiftKey) {
                const parentPath = treeVisibleNodes[selectedIndex]?.parentPath;
                if (parentPath) selectTreePathOrAncestor(parentPath);
              } else {
                selectByIndex(Math.max(selectedIndex - 1, 0));
              }
            } else selectByIndex(Math.max(selectedIndex - 1, 0));
            break;
          case 'KeyG':
            if (isDoubleG) {
              event.preventDefault();
              selectByIndex(0);
              lastKey = '';
              return;
            }
            if (event.shiftKey) {
              event.preventDefault();
              selectByIndex(displayFiles.length - 1);
            }
            break;
          case 'KeyH':
            if (projectMode) {
              event.preventDefault();
              if (event.shiftKey) {
                collapseDeepestTreeLevel();
              } else {
                const node = treeVisibleNodes[selectedIndex];
                if (node?.expanded) {
                  node.expanded = false;
                  projectRoot = projectRoot ? { ...projectRoot } : null;
                } else if (node?.parentPath) selectTreePathOrAncestor(node.parentPath);
              }
              break;
            }
            if (type === 'current' || type === 'parent') {
              event.preventDefault();
              const shouldRestore = isFocused;
              const dirName = path.split(/[/\\]/).filter(Boolean).pop();
              if (dirName && type === 'current') pendingSelectName = dirName;
              onNavigateUp();
              if (shouldRestore) {
                setTimeout(() => { panelElement?.focus(); isFocused = true; }, 100);
              }
            }
            break;
          case 'KeyL':
            if (projectMode) {
              event.preventDefault();
              const node = treeVisibleNodes[selectedIndex];
              if (node?.entry.is_dir) {
                if (event.shiftKey) void expandRecursively(node);
                else if (node.expanded && node.children.length > 0) selectByIndex(selectedIndex + 1);
                else void expandTreeNode(node);
              } else if (node) onActivate(node.entry.path);
              break;
            }
            if (type === 'current' || detached || type === 'parent') {
              event.preventDefault();
              if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
                const entry = displayFiles[selectedIndex];
                if (entry.is_dir) {
                  onNavigate(entry.path);
                } else {
                  onActivate(entry.path);
                }
              }
            }
            break;
          case 'Slash':
            event.preventDefault();
            if (lastKey === 'KeyA' && now - lastKeyTime < 500) {
              lastKey = '';
              startCreateDir();
            } else if (isGSlash) {
              searchMode = 'recursive';
              lastKey = '';
              openSearchModal();
            } else {
              searchMode = 'current';
              openSearchModal();
            }
            break;
          case 'Space':
            event.preventDefault();
            if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
              const entry = displayFiles[selectedIndex];
              if (entry.name !== '..') {
                const newSet = new Set(selectedPaths);
                if (newSet.has(entry.path)) {
                  newSet.delete(entry.path);
                } else {
                  newSet.add(entry.path);
                }
                selectedPaths = newSet;
              }
              // Advance cursor
              selectByIndex(Math.min(selectedIndex + 1, displayFiles.length - 1));
            }
            break;
          case 'KeyV':
            event.preventDefault();
            if (selectedPaths.size > 0) {
              selectedPaths = new Set();
            } else {
              selectedPaths = new Set(displayFiles.filter(f => f.name !== '..').map(f => f.path));
            }
            break;
          case 'KeyY':
            event.preventDefault();
            {
              const entries = getEntriesToOperate();
              if (entries.length > 0) {
                clipboard.yank(entries);
                selectedPaths = new Set();
                onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} yanked`);
              }
            }
            break;
          case 'KeyX':
            event.preventDefault();
            {
              const entries = getEntriesToOperate();
              if (entries.length > 0) {
                clipboard.cut(entries);
                selectedPaths = new Set();
                onToast(`${entries.length} ${entries.length === 1 ? 'file' : 'files'} cut`);
              }
            }
            break;
          case 'Period':
            event.preventDefault();
            showHidden = !showHidden;
            onToast(showHidden ? 'Showing hidden files' : 'Hiding hidden files');
            break;
          case 'KeyA':
            event.preventDefault();
            // a alone = create file, a/ = create dir (handled via lastKey in Slash case)
            if (lastKey === 'KeyA' && now - lastKeyTime < 500) {
              // Double-a: do nothing special
            }
            // Delay to check if '/' follows
            setTimeout(() => {
              if (lastKey === 'KeyA') {
                startCreateFile();
              }
            }, 300);
            break;
          case 'KeyD':
            event.preventDefault();
            handleDelete(false);
            break;
        }
        break;
    }

    // Don't overwrite prefix state when modifier keys are pressed first
    // (e.g. user presses s, then Shift+N — Shift keydown fires before N)
    if (!['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(event.code)) {
      lastKey = event.code;
      lastKeyTime = now;
    }
  }

  function handleItemClick(index: number) {
    selectByIndex(index);
  }

  function handleItemDblClick(entry: FileEntry) {
    if (entry.is_dir) {
      if (projectMode) {
        const node = findTreeNode(entry.path);
        if (node) toggleTreeNode(node);
      } else onNavigate(entry.path);
    } else {
      onActivate(entry.path);
    }
  }

  function toggleTreeNode(node: TreeNode): void {
    if (node.expanded) {
      node.expanded = false;
      projectRoot = projectRoot ? { ...projectRoot } : null;
    } else void expandTreeNode(node);
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
  tabindex="0"
>
  <div class="panel-header">
    {#if path}
      {#if detached}
        <span class="detach-marker" title="Manual mode (detached)">&#9679;</span>
      {/if}
      <span class="panel-path" title={path}>{path === '/' ? '/' : path.split('\\').pop() || path.split('/').pop() || path}</span>
    {/if}
    {#if filterPattern}
      <span class="filter-badge" title={filterPattern}>{filterPattern}</span>
    {/if}
  </div>

  <div class="panel-content">
    <InputDialog
      visible={inputVisible}
      value={inputValue}
      placeholder={inputPlaceholder}
      prompt={inputPrompt}
      onConfirm={handleInputConfirm}
      onCancel={handleInputCancel}
    />
    {#if isLoading}
      <p class="placeholder">Loading...</p>
    {:else if errorMessage}
      <p class="error">{errorMessage}</p>
    {:else if displayFiles.length === 0}
      <p class="placeholder">Empty directory</p>
    {:else}
      <div class="file-list">
        {#each displayFiles as file, index (file.path)}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div
            class="file-item"
            class:selected={selectedFile?.path === file.path}
            class:multi-selected={selectedPaths.has(file.path)}
            class:cut-marked={cutPaths.has(file.path)}
            class:directory={file.is_dir}
            class:hidden-file={file.is_hidden}
            onclick={() => handleItemClick(index)}
            ondblclick={() => handleItemDblClick(file)}
            onkeydown={() => {}}
            data-path={file.path}
            data-index={index}
          >
            {#if selectedPaths.has(file.path)}
              <span class="select-marker">*</span>
            {:else if cutPaths.has(file.path)}
              <span class="cut-marker">x</span>
            {/if}
            {#if projectMode}
              <span class="tree-indent" style={`width: ${(treeVisibleNodes[index]?.depth ?? 0) * 16}px`}></span>
              <button
                type="button"
                aria-label={treeVisibleNodes[index]?.expanded ? 'Collapse directory' : 'Expand directory'}
                class="tree-toggle"
                class:expanded={treeVisibleNodes[index]?.expanded}
                class:directory-toggle={file.is_dir}
                onclick={(event) => {
                  if (!file.is_dir) return;
                  event.stopPropagation();
                  const node = treeVisibleNodes[index];
                  if (node) toggleTreeNode(node);
                }}
              ></button>
            {/if}
            <span class="file-name" class:is-dir={file.is_dir}>{file.name}{file.is_dir && !/[\\/]$/.test(file.name) ? '/' : ''}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <SearchModal
    visible={isSearchModalOpen}
    rootPath={path}
    mode={searchMode}
    allowRecursive={!path.startsWith('ftp://')}
    entries={path.startsWith('ftp://') ? files : null}
    onClose={closeSearchModal}
    onSelect={handleSearchSelect}
  />

  <ConfirmModal
    visible={showDeleteConfirm}
    title={deleteIsPermanent ? 'Permanent Delete' : 'Move to Trash'}
    fileName={deleteIsPermanent
      ? `Permanently delete ${getEntriesToOperate().length} item(s)?`
      : `Move ${getEntriesToOperate().length} item(s) to trash?`}
    buttons={[
      { key: 'D', label: 'elete', action: handleDeleteConfirm, style: 'danger' },
      { key: 'C', label: 'ancel', action: handleDeleteCancel },
    ]}
  />

  <FileInfoPanel
    visible={showFileInfo}
    info={fileInfo}
    onClose={closeFileInfo}
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
    background-color: var(--bg-hover);
    border-left: 2px solid var(--accent);
  }

  .file-item.cut-marked {
    opacity: 0.5;
  }

  .file-item.hidden-file {
    opacity: 0.6;
  }

  .select-marker {
    display: inline-block;
    width: 12px;
    font-size: 11px;
    color: var(--accent);
    flex-shrink: 0;
    text-align: center;
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
    width: 14px;
    position: relative;
    border: 0;
    padding: 0;
    background: transparent;
  }

  .tree-toggle::before {
    content: '';
    position: absolute;
    top: 50%;
    left: -8px;
    width: 8px;
    border-top: 1px solid var(--border);
    opacity: 0.65;
  }

  .tree-toggle.directory-toggle::after {
    content: '';
    position: absolute;
    top: calc(50% - 3px);
    left: 2px;
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
