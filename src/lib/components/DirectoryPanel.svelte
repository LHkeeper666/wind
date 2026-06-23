<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, untrack } from 'svelte';
  import SearchModal from './SearchModal.svelte';

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  let {
    type = 'current',
    path = '',
    selectedPath = null,
    onNavigate = (path: string) => {},
    onSelect = (path: string) => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onFullscreen = () => {},
    onNavigateUp = () => {},
    onTabCommand = (cmd: string) => {},
  }: {
    type: 'parent' | 'current';
    path: string;
    selectedPath: string | null;
    onNavigate?: (path: string) => void;
    onSelect?: (path: string) => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onFullscreen?: () => void;
    onNavigateUp?: () => void;
    onTabCommand?: (cmd: string) => void;
  } = $props();

  let files: FileEntry[] = $state([]);
  let isLoading: boolean = $state(false);
  let errorMessage: string = $state('');
  let selectedIndex: number = $state(-1);
  let selectedPathInternal: string | null = $state(null);
  let lastKeyTime: number = 0;
  let lastKey: string = '';
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  // t prefix state for tab operations
  let waitingForTabKey: boolean = false;
  let isFocused: boolean = $state(false);
  let selectTimeout: ReturnType<typeof setTimeout> | null = null;
  let scrollRafId: number = 0;
  let isSearchModalOpen: boolean = $state(false);
  let searchMode: 'current' | 'recursive' = $state('current');

  const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico']);

  export function getImageFiles(): FileEntry[] {
    return files.filter(f => !f.is_dir && IMAGE_EXTENSIONS.has(
      f.name.split('.').pop()?.toLowerCase() || ''
    ));
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
    loadDirectory(path, true);
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
      untrack(() => loadDirectory(path));
    }
  });

  // Sync from selectedPath prop only when it actually changes
  let prevPropPath: string | null = null;
  $effect(() => {
    const sp = selectedPath;
    const f = files;
    if (sp !== prevPropPath && sp && f.length > 0) {
      prevPropPath = sp;
      const idx = f.findIndex(entry => entry.path === sp);
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
      if (selectedIndex < 0 && files.length > 0) {
        const firstReal = files.findIndex(f => f.name !== '..');
        selectedIndex = firstReal >= 0 ? firstReal : 0;
        selectedPathInternal = files[selectedIndex].path;
      }
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
      if (selectedIndex < 0 && files.length > 0) {
        const firstReal = files.findIndex(f => f.name !== '..');
        selectedIndex = firstReal >= 0 ? firstReal : 0;
        selectedPathInternal = files[selectedIndex].path;
      }
    } catch (error) {
      console.error('Failed to load directory:', error);
      errorMessage = `Failed to load: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  function selectByIndex(index: number) {
    if (index >= 0 && index < files.length) {
      selectedIndex = index;
      selectedPathInternal = files[index].path;

      // Debounce onSelect to avoid rapid file loading
      if (selectTimeout) {
        clearTimeout(selectTimeout);
      }
      const capturedIndex = index;
      selectTimeout = setTimeout(() => {
        // Only fire if user is still on the same item
        if (selectedIndex === capturedIndex) {
          onSelect(files[capturedIndex].path);
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

  function handleKeydown(event: KeyboardEvent) {
    // Only handle if this panel has focus
    if (!isFocused) {
      return;
    }

    // Stop event propagation to prevent other panels from handling
    event.stopPropagation();

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;
    const isGSlash = lastKey === 'KeyG' && event.code === 'Slash' && now - lastKeyTime < 500;

    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < files.length) {
          const entry = files[selectedIndex];
          if (entry.is_dir) {
            onNavigate(entry.path);
          } else {
            onSelect(entry.path);
          }
        }
        break;
      case 'R':
        event.preventDefault();
        if (path) loadDirectory(path, true); // Force refresh
        break;
      case 'E':
        event.preventDefault();
        onFullscreen();
        break;
      default:
        // t prefix for tab operations
        if (waitingForTabKey) {
          waitingForTabKey = false;
          const code = event.code;
          const key = event.key;
          event.preventDefault();
          if (code === 'KeyT') {
            onTabCommand('new');
          } else if (code === 'KeyC') {
            onTabCommand('close');
          } else if (code === 'KeyR') {
            onTabCommand('rename-hint');
          } else if (code === 'KeyN' || code === 'BracketRight') {
            onTabCommand('next');
          } else if (code === 'KeyP' || code === 'BracketLeft') {
            onTabCommand('prev');
          } else if (code === 'Comma') {
            onTabCommand('swap-prev');
          } else if (code === 'Period') {
            onTabCommand('swap-next');
          } else if (key >= '1' && key <= '9') {
            onTabCommand('switch-' + key);
          }
          return;
        }
        // Use event.code for letter keys to support Chinese IME
        switch (event.code) {
          case 'KeyT':
            event.preventDefault();
            waitingForTabKey = true;
            setTimeout(() => { waitingForTabKey = false; }, 1000);
            break;
          case 'KeyJ':
            event.preventDefault();
            selectByIndex(Math.min(selectedIndex + 1, files.length - 1));
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
              selectByIndex(files.length - 1);
            }
            break;
          case 'KeyH':
            if (type === 'current') {
              event.preventDefault();
              onNavigateUp();
            }
            break;
          case 'KeyL':
            if (type === 'current') {
              event.preventDefault();
              if (selectedIndex >= 0 && selectedIndex < files.length) {
                const entry = files[selectedIndex];
                if (entry.is_dir) {
                  onNavigate(entry.path);
                } else {
                  onSelect(entry.path);
                }
              }
            }
            break;
          case 'Slash':
            event.preventDefault();
            if (isGSlash) {
              searchMode = 'recursive';
              lastKey = '';
            } else {
              searchMode = 'current';
            }
            openSearchModal();
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
  </div>

  <div class="panel-content">
    {#if isLoading}
      <p class="placeholder">Loading...</p>
    {:else if errorMessage}
      <p class="error">{errorMessage}</p>
    {:else if files.length === 0}
      <p class="placeholder">Empty directory</p>
    {:else}
      <div class="file-list">
        {#each files as file, index (file.path)}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div
            class="file-item"
            class:selected={index === selectedIndex}
            class:directory={file.is_dir}
            onclick={() => handleItemClick(index)}
            ondblclick={() => handleItemDblClick(file)}
            onkeydown={() => {}}
            data-path={file.path}
            data-index={index}
          >
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
