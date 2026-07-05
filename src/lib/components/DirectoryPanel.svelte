<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy, untrack } from 'svelte';
  import { layout } from '$lib/stores/layout';
  import { clipboard, type ClipboardEntry } from '$lib/stores/clipboard';
  import SearchModal from './SearchModal.svelte';
  import InputDialog from './InputDialog.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import FileInfoPanel from './FileInfoPanel.svelte';
  import BatchRenameModal from './BatchRenameModal.svelte';

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size?: number | null;
    is_hidden?: boolean;
    modified?: number | null;
    created?: number | null;
  }

  let {
    type = 'current',
    path = '',
    selectedPath = null,
    onNavigate = (path: string) => {},
    onSelect = (path: string) => {},
    onActivate = (path: string) => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onFullscreen = () => {},
    onNavigateUp = () => {},
    onTabCommand = (cmd: string) => {},
    onToast = (message: string) => {},
  }: {
    type: 'parent' | 'current';
    path: string;
    selectedPath: string | null;
    onNavigate?: (path: string) => void;
    onSelect?: (path: string) => void;
    onActivate?: (path: string) => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onFullscreen?: () => void;
    onNavigateUp?: () => void;
    onTabCommand?: (cmd: string) => void;
    onToast?: (message: string) => void;
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

  // Filter state
  let filterPattern: string = $state('');

  // Derived values that depend on state declared above
  let displayFiles: FileEntry[] = $derived.by(() => {
    let result = showHidden ? files : files.filter(f => f.name === '..' || !f.is_hidden);

    // Apply filter
    if (filterPattern) {
      const pattern = filterPattern.replace(/\*/g, '.*').replace(/\?/g, '.');
      const regex = new RegExp(`^${pattern}$`, 'i');
      result = result.filter(f => f.name === '..' || regex.test(f.name));
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
        cmp = a.name.localeCompare(b.name);
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

  // Batch rename state
  let showBatchRename: boolean = $state(false);
  let batchRenameFiles: { path: string; name: string }[] = $state([]);

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

  // Directory content cache
  const directoryCache: Map<string, FileEntry[]> = new Map();

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

  export function refresh() {
    return loadDirectory(path, true);
  }

  function handleFocus() {
    isFocused = true;
  }

  function handleBlur() {
    isFocused = false;
  }

  // Load directory content when path changes
  let prevPath: string = '';
  $effect(() => {
    if (path && path !== prevPath) {
      prevPath = path;
      selectedIndex = -1;
      selectedPathInternal = null;
      selectedPaths = new Set();
      untrack(() => loadDirectory(path));
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

  async function loadDirectory(dirPath: string, forceRefresh: boolean = false) {
    isLoading = true;
    errorMessage = '';

    const isVirtual = isVirtualRoot(dirPath);

    // Check cache first
    if (!forceRefresh && directoryCache.has(dirPath)) {
      files = directoryCache.get(dirPath)!;
      // Inject .. for non-root directories (virtual root has no ..)
      if (!isVirtual) {
        const parentPath = getParentPath(dirPath);
        files = [{ name: '..', path: parentPath, is_dir: true }, ...files];
      }
      selectInitialEntry();
      isLoading = false;
      return;
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
        files = [{ name: '..', path: parentPath, is_dir: true }, ...files];
      }
      // Update cache (store without .., inject on read)
      directoryCache.set(dirPath, files.filter(f => f.name !== '..'));
      selectInitialEntry();
    } catch (error) {
      console.error('Failed to load directory:', error);
      errorMessage = `Failed to load: ${error}`;
    } finally {
      isLoading = false;
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
    batchRenameFiles = entries.map(f => ({ path: f.path, name: f.name }));
    showBatchRename = true;
  }

  async function handleBatchRenameConfirm(renames: { old_path: string; new_name: string }[]) {
    showBatchRename = false;
    try {
      const result = await invoke('batch_rename', { entries: renames });
      selectedPaths = new Set();
      await loadDirectory(path, true);
      onToast(`Renamed ${(result as string[]).length} file(s)`);
    } catch (e) {
      onToast(`Batch rename error: ${e}`);
      await loadDirectory(path, true);
    }
    setTimeout(() => panelElement?.focus(), 0);
  }

  function handleBatchRenameCancel() {
    showBatchRename = false;
    setTimeout(() => panelElement?.focus(), 0);
  }

  async function handleInputConfirm(value: string) {
    inputVisible = false;
    try {
      if (inputMode === 'rename') {
        const entry = displayFiles[selectedIndex];
        const parentPath = path.replace(/[\\\/]+$/, '');
        const newPath = parentPath + '\\' + value;
        await invoke('rename_file', { oldPath: entry.path, newName: value });
        await currentDirectoryPanel_refresh();
        onSelect(newPath);
        onToast(`Renamed to ${value}`);
      } else if (inputMode === 'create-file') {
        const parentPath = path.replace(/[\\\/]+$/, '');
        const newPath = parentPath + '\\' + value;
        await invoke('create_file', { path: newPath, isDir: false });
        await currentDirectoryPanel_refresh();
        onSelect(newPath);
        onToast(`Created ${value}`);
      } else if (inputMode === 'create-dir') {
        const parentPath = path.replace(/[\\\/]+$/, '');
        const newPath = parentPath + '\\' + value;
        await invoke('create_file', { path: newPath, isDir: true });
        await currentDirectoryPanel_refresh();
        onSelect(newPath);
        onToast(`Created ${value}/`);
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

  function startFilter() {
    inputMode = 'filter';
    inputValue = filterPattern;
    inputPlaceholder = '*.txt, *.rs, *.{js,ts}';
    inputPrompt = 'Filter:';
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

    const cmd = permanent ? 'permanent_delete' : 'delete_file';
    let deleted = 0;
    for (const entry of entries) {
      try {
        await invoke(cmd, { path: entry.path });
        deleted++;
      } catch (e) {
        onToast(`Failed to delete ${entry.name}: ${e}`);
      }
    }

    selectedPaths = new Set();
    if (deleted > 0) {
      onToast(`${deleted} ${deleted === 1 ? 'file' : 'files'} ${permanent ? 'permanently deleted' : 'moved to trash'}`);
      await loadDirectory(path, true);
    }
  }

  function getEntriesToOperate(): ClipboardEntry[] {
    if (selectedPaths.size > 0) {
      return files
        .filter(f => f.name !== '..' && selectedPaths.has(f.path))
        .map(f => ({ path: f.path, name: f.name, is_dir: f.is_dir }));
    }
    if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
      const f = displayFiles[selectedIndex];
      if (f.name === '..') return [];
      return [{ path: f.path, name: f.name, is_dir: f.is_dir }];
    }
    return [];
  }

  function handleKeydown(event: KeyboardEvent) {
    // Only handle if this panel has focus
    if (!isFocused) {
      return;
    }

    // Don't intercept keys when a modal or dialog is open
    if (showDeleteConfirm || inputVisible || isSearchModalOpen || showFileInfo || showBatchRename) {
      event.stopPropagation();
      return;
    }

    // Stop event propagation to prevent other panels from handling
    event.stopPropagation();

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;
    const isGSlash = lastKey === 'KeyG' && event.code === 'Slash' && now - lastKeyTime < 500;
    const isSortPrefix = lastKey === 'KeyS' && now - lastKeyTime < 500;

    // Handle sort prefix sub-keys before the main switch
    // Lowercase = ascending, uppercase (Shift) = descending
    if (isSortPrefix) {
      const baseKey = event.code === 'KeyN' ? 'name'
        : event.code === 'KeyS' ? 'size'
        : event.code === 'KeyE' ? 'ext'
        : event.code === 'KeyM' ? 'modified'
        : event.code === 'KeyC' ? 'created'
        : event.code === 'KeyT' ? 'dirfirst'
        : null;
      if (baseKey) {
        event.preventDefault();
        if (baseKey === 'dirfirst') {
          toggleDirFirst();
        } else {
          setSort(baseKey as any, event.shiftKey);
        }
        lastKey = '';
        return;
      }
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
            onNavigate(entry.path);
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
        startFilter();
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
            selectByIndex(Math.max(selectedIndex - 1, 0));
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
            if (type === 'current') {
              event.preventDefault();
              // Remember current directory name so parent highlights it
              const dirName = path.split(/[/\\]/).filter(Boolean).pop();
              if (dirName) pendingSelectName = dirName;
              onNavigateUp();
            }
            break;
          case 'KeyL':
            if (type === 'current') {
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
              // a/ = create directory
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

    lastKey = event.code;
    lastKeyTime = now;
  }

  function handleItemClick(index: number) {
    selectByIndex(index);
  }

  function handleItemDblClick(entry: FileEntry) {
    if (entry.is_dir) {
      onNavigate(entry.path);
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
  tabindex="0"
>
  <div class="panel-header">
    {#if path}
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
            onclick={() => handleItemClick(files.findIndex(f => f.path === file.path))}
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

  <BatchRenameModal
    visible={showBatchRename}
    files={batchRenameFiles}
    onConfirm={handleBatchRenameConfirm}
    onCancel={handleBatchRenameCancel}
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
