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
  }: {
    type: 'parent' | 'current';
    path: string;
    selectedPath: string | null;
    onNavigate?: (path: string) => void;
    onSelect?: (path: string) => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onFullscreen?: () => void;
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
  let isSearchModalOpen: boolean = $state(false);
  let searchMode: 'current' | 'recursive' = $state('current');

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
    }
  }

  function handleFocus() {
    isFocused = true;
  }

  function handleBlur() {
    isFocused = false;
  }

  // Load directory content when path changes
  $effect(() => {
    if (path) {
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
    const isDoubleG = lastKey === 'g' && event.key === 'g' && now - lastKeyTime < 500;
    const isGSlash = lastKey === 'g' && event.key === '/' && now - lastKeyTime < 500;

    switch (event.key) {
      case 'j':
        event.preventDefault();
        selectByIndex(Math.min(selectedIndex + 1, files.length - 1));
        break;
      case 'k':
        event.preventDefault();
        selectByIndex(Math.max(selectedIndex - 1, 0));
        break;
      case 'g':
        if (isDoubleG) {
          event.preventDefault();
          selectByIndex(0);
          lastKey = '';
          return;
        }
        break;
      case 'G':
        event.preventDefault();
        selectByIndex(files.length - 1);
        break;
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
      case 'h':
        event.preventDefault();
        onSwitchPanel('left');
        break;
      case 'l':
        event.preventDefault();
        onSwitchPanel('right');
        break;
      case '/':
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

    lastKey = event.key;
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
    <span class="panel-title">{type === 'parent' ? 'Parent Directory' : 'Current Directory'}</span>
    {#if path}
      <span class="panel-path" title={path}>{path.split('\\').pop() || path}</span>
    {/if}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="search-trigger" onclick={openSearchModal} onkeydown={(e) => { if (e.key === '/') { e.preventDefault(); openSearchModal(); } }}>
      <span class="search-icon">🔍</span>
    </div>
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
            <span class="file-icon">{file.is_dir ? '📁' : '📄'}</span>
            <span class="file-name">{file.name}{file.is_dir && !/[\\/]$/.test(file.name) ? '/' : ''}</span>
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
    background-color: #252526;
    outline: none;
  }

  .panel-header {
    padding: 8px 12px;
    background-color: #252526;
    border-bottom: 1px solid #333333;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .panel-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #888888;
    flex-shrink: 0;
  }

  .panel-path {
    font-size: 11px;
    color: #666666;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .search-trigger {
    margin-left: auto;
    padding: 4px 6px;
    background-color: #3c3c3c;
    border: 1px solid #555555;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
  }

  .search-trigger:hover {
    border-color: #007acc;
    background-color: #4c4c4c;
  }

  .search-icon {
    font-size: 12px;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
    overflow-anchor: none;
  }

  .file-list {
    font-size: 13px;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
    gap: 8px;
  }

  .file-item:hover {
    background-color: #2a2d2e;
  }

  .file-item.selected {
    background-color: #094771;
  }

  .file-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .placeholder {
    color: #666666;
    font-size: 14px;
    text-align: center;
    margin-top: 40px;
  }

  .error {
    color: #f44747;
    font-size: 14px;
    text-align: center;
    margin-top: 40px;
  }
</style>
