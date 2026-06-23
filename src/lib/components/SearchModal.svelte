<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy, tick } from 'svelte';

  interface SearchResult {
    name: string;
    path: string;
    relative_path: string;
    is_dir: boolean;
  }

  let {
    visible = false,
    rootPath = '',
    mode = 'current',
    onClose = () => {},
    onSelect = (path: string, isDir: boolean) => {},
  }: {
    visible: boolean;
    rootPath: string;
    mode?: 'current' | 'recursive';
    onClose?: () => void;
    onSelect?: (path: string, isDir: boolean) => void;
  } = $props();

  let searchInput: HTMLInputElement | undefined = $state(undefined);
  let resultsContainer: HTMLDivElement | undefined = $state(undefined);
  let modalContent: HTMLDivElement | undefined = $state(undefined);
  let query: string = $state('');
  let currentMode: 'current' | 'recursive' = $state(mode);
  let results: SearchResult[] = $state([]);
  let selectedIndex: number = $state(-1);
  let isSearching: boolean = $state(false);
  let isValidRegex: boolean = $state(true);
  let showRgHint: boolean = $state(false);
  let reachedLimit: boolean = $state(false);
  let focusMode: 'input' | 'list' = $state('input');
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let searchGeneration: number = 0;  // 用于取消过期的搜索

  const MAX_RESULTS = 30;  // 减少结果数量，提升性能

  // Focus input when modal becomes visible
  $effect(() => {
    if (visible && searchInput) {
      tick().then(() => {
        searchInput?.focus();
      });
    }
  });

  // 确保输入框始终可聚焦
  $effect(() => {
    if (searchInput && focusMode === 'input') {
      searchInput.focus();
    }
  });

  // Reset state when modal opens
  $effect(() => {
    if (visible) {
      query = '';
      results = [];
      selectedIndex = -1;
      isSearching = false;
      isValidRegex = true;
      focusMode = 'input';
      currentMode = mode;
      checkSearchTools();
    }
  });

  async function checkSearchTools() {
    try {
      const tools = await invoke<{ fd: boolean; rg: boolean }>('check_search_tools');
      // 如果 fd 和 rg 都不可用，显示提示
      showRgHint = !tools.fd && !tools.rg;
    } catch {
      showRgHint = true;
    }
  }

  function validateRegex(pattern: string): boolean {
    if (!pattern) return true;
    try {
      new RegExp(pattern);
      return true;
    } catch {
      return false;
    }
  }

  function cancelSearch() {
    searchGeneration++;
    isSearching = false;
    if (debounceTimer) clearTimeout(debounceTimer);
    invoke('cancel_search').catch(() => {});
  }

  function handleInput() {
    isValidRegex = validateRegex(query);

    // 取消之前的搜索
    cancelSearch();

    if (!isValidRegex || !query) {
      results = [];
      selectedIndex = -1;
      return;
    }

    // 当前目录搜索无需防抖，递归搜索 400ms 防抖
    const delay = currentMode === 'current' ? 0 : 400;
    debounceTimer = setTimeout(() => {
      performSearch();
    }, delay);
  }

  async function performSearch() {
    if (!query || !rootPath) return;

    const currentGeneration = ++searchGeneration;
    isSearching = true;

    try {
      const searchResults = await invoke<SearchResult[]>('search_files', {
        rootPath,
        pattern: query,
        maxResults: MAX_RESULTS,
        recursive: currentMode === 'recursive',
      });

      // 检查是否是当前搜索（忽略过期的搜索结果）
      if (currentGeneration !== searchGeneration) return;

      results = searchResults;
      reachedLimit = searchResults.length >= MAX_RESULTS;
      selectedIndex = results.length > 0 ? 0 : -1;
    } catch (error) {
      if (currentGeneration !== searchGeneration) return;
      console.error('Search failed:', error);
      results = [];
    } finally {
      if (currentGeneration === searchGeneration) {
        isSearching = false;
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (focusMode === 'input') {
      handleInputKeydown(event);
    } else {
      handleListKeydown(event);
    }
  }

  function handleInputKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        if (isSearching) {
          cancelSearch();
        } else {
          onClose();
        }
        break;
      case 'Tab':
        event.preventDefault();
        currentMode = currentMode === 'current' ? 'recursive' : 'current';
        if (query && isValidRegex) {
          if (debounceTimer) clearTimeout(debounceTimer);
          performSearch();
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (results.length > 0) {
          focusMode = 'list';
          selectedIndex = 0;
          searchInput?.blur();
          modalContent?.focus();
          scrollSelectedIntoView();
        }
        break;
      default:
        // Use event.code for letter keys to support Chinese IME
        if (event.code === 'KeyJ' && event.ctrlKey) {
          event.preventDefault();
          if (results.length > 0) {
            focusMode = 'list';
            selectedIndex = 0;
            searchInput?.blur();
            modalContent?.focus();
            scrollSelectedIntoView();
          }
        }
        break;
    }
  }

  function handleListKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        focusMode = 'input';
        tick().then(() => searchInput?.focus());
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < results.length) {
          const result = results[selectedIndex];
          onSelect(result.path, result.is_dir);
          onClose();
        }
        break;
      case 'Tab':
        event.preventDefault();
        currentMode = currentMode === 'current' ? 'recursive' : 'current';
        focusMode = 'input';
        tick().then(() => searchInput?.focus());
        if (query && isValidRegex) {
          if (debounceTimer) clearTimeout(debounceTimer);
          performSearch();
        }
        break;
      default:
        // Use event.code for letter keys to support Chinese IME
        switch (event.code) {
          case 'KeyJ':
            event.preventDefault();
            selectNext();
            break;
          case 'KeyK':
            event.preventDefault();
            selectPrev();
            break;
          case 'KeyG':
            event.preventDefault();
            if (event.shiftKey) {
              // G - go to bottom
              selectedIndex = results.length - 1;
            } else {
              // g - go to top
              selectedIndex = 0;
            }
            scrollSelectedIntoView();
            break;
        }
        break;
    }
  }

  function selectNext() {
    if (selectedIndex < results.length - 1) {
      selectedIndex++;
      scrollSelectedIntoView();
    }
  }

  function selectPrev() {
    if (selectedIndex > 0) {
      selectedIndex--;
      scrollSelectedIntoView();
    }
  }

  function scrollSelectedIntoView() {
    tick().then(() => {
      if (!resultsContainer) return;
      const el = resultsContainer.querySelector(`[data-index="${selectedIndex}"]`);
      if (el) {
        el.scrollIntoView({ block: 'nearest' });
      }
    });
  }

  function handleResultClick(index: number) {
    selectedIndex = index;
    focusMode = 'list';
  }

  function handleResultDblClick(path: string, isDir: boolean) {
    onSelect(path, isDir);
    onClose();
  }

  onDestroy(() => {
    cancelSearch();
  });
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal-overlay" onkeydown={handleKeydown} onclick={onClose}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="modal-content" bind:this={modalContent} tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <div class="search-header">
        <input
          type="text"
          class="search-input"
          class:invalid={!isValidRegex}
          placeholder="Search files (regex supported)..."
          bind:value={query}
          bind:this={searchInput}
          oninput={handleInput}
        />
        <span class="search-mode-badge" class:recursive={currentMode === 'recursive'}>
          {currentMode === 'recursive' ? 'Recursive' : 'Current Directory'}
        </span>
        {#if isSearching}
          <button class="search-cancel" onclick={cancelSearch} title="Cancel search (Esc)">
            Cancel
          </button>
        {/if}
      </div>

      {#if showRgHint}
        <div class="rg-hint">
          Install fd for faster search: <code>winget install sharkdp.fd</code>
        </div>
      {/if}

      <div class="results-container" bind:this={resultsContainer}>
        {#if results.length === 0 && query && !isSearching}
          <div class="no-results">No results found</div>
        {:else}
          {#each results as result, index (result.path)}
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <div
              class="result-item"
              class:selected={index === selectedIndex}
              class:directory={result.is_dir}
              data-index={index}
              onclick={() => handleResultClick(index)}
              ondblclick={() => handleResultDblClick(result.path, result.is_dir)}
              onkeydown={() => {}}
            >
              <span class="result-name" class:is-dir={result.is_dir}>{result.name}{result.is_dir ? '/' : ''}</span>
              <span class="result-path">{result.relative_path}</span>
            </div>
          {/each}
        {/if}

        {#if reachedLimit}
          <div class="limit-hint">Showing first {MAX_RESULTS} results</div>
        {/if}
      </div>

      <div class="modal-footer">
        <span class="hint">Tab: switch mode | Enter: select | j/k: navigate | Esc: cancel/close</span>
        {#if focusMode === 'list'}
          <span class="mode-indicator">LIST MODE</span>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-content {
    width: 90%;
    max-width: 600px;
    max-height: 70vh;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
  }

  .search-header {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
    gap: 8px;
  }

  .search-input {
    flex: 1;
    padding: 6px 10px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    font-family: var(--font-mono);
  }

  .search-input:focus {
    border-color: var(--border-focus);
  }

  .search-input.invalid {
    border-color: var(--error);
  }

  .search-mode-badge {
    font-size: 10px;
    color: var(--accent);
    white-space: nowrap;
    padding: 1px 6px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
  }

  .search-mode-badge.recursive {
    color: var(--warning);
  }

  .search-cancel {
    font-size: 10px;
    color: var(--text-primary);
    white-space: nowrap;
    padding: 2px 8px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    cursor: pointer;
    font-family: var(--font-mono);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .search-cancel:hover {
    background-color: var(--bg-hover);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .rg-hint {
    padding: 6px 16px;
    background-color: var(--bg-primary);
    font-size: 11px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
  }

  .rg-hint code {
    background-color: var(--bg-tertiary);
    padding: 1px 4px;
    font-family: var(--font-mono);
  }

  .results-container {
    flex: 1;
    overflow-y: auto;
    padding: 2px 0;
    min-height: 200px;
    max-height: 400px;
  }

  .no-results {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }

  .result-item {
    display: flex;
    align-items: center;
    padding: 4px 16px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .result-item:hover {
    background-color: var(--bg-hover);
  }

  .result-item.selected {
    background-color: var(--bg-active);
  }

  .result-name {
    font-size: 13px;
    font-family: var(--font-mono);
    flex-shrink: 0;
    color: var(--file-color);
  }

  .result-name.is-dir {
    color: var(--dir-color);
  }

  .result-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    color: var(--text-muted);
    margin-left: auto;
  }

  .limit-hint {
    padding: 6px 16px;
    text-align: center;
    font-size: 11px;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 16px;
    border-top: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .hint {
    font-size: 10px;
    color: var(--text-muted);
  }

  .mode-indicator {
    font-size: 10px;
    color: var(--warning);
    font-weight: bold;
    animation: pulse 1.5s ease-in-out infinite;
  }
</style>
