<script lang="ts">
  import type { TocHeading } from '$lib/previewers';

  let {
    headings = [],
    activeLine = -1,
    onJump = (line: number) => {},
    onFocusChange = (focused: boolean) => void 0,
  }: {
    headings: TocHeading[];
    activeLine?: number;
    onJump?: (line: number) => void;
    onFocusChange?: (focused: boolean) => void;
  } = $props();

  let selectedIndex: number = $state(-1);
  let isFocused: boolean = $state(false);
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  let lastKeyTime: number = 0;
  let lastKey: string = '';

  // Search state
  let searchActive: boolean = $state(false);
  let searchQuery: string = $state('');
  let searchInput: HTMLInputElement | undefined = $state(undefined);

  // Flatten visible headings for navigation
  interface FlatHeading {
    heading: TocHeading;
    depth: number;
    parentIndex: number; // index of parent in flat list, -1 for root
  }

  function flattenHeadings(items: TocHeading[], depth: number = 0): FlatHeading[] {
    const result: FlatHeading[] = [];
    for (const item of items) {
      // Filter by search query
      if (searchQuery && !item.text.toLowerCase().includes(searchQuery.toLowerCase())) {
        // Still include if any child matches
        const matchingChildren = searchQuery
          ? flattenHeadings(item.children, depth + 1).length > 0
          : true;
        if (!matchingChildren && item.children.length > 0) {
          // Skip this item but include matching children? No, skip entire subtree if no match
          continue;
        }
        if (!matchingChildren) continue;
      }
      result.push({ heading: item, depth, parentIndex: -1 });
      if (item.expanded && item.children.length > 0) {
        result.push(...flattenHeadings(item.children, depth + 1));
      }
    }
    return result;
  }

  let flatItems = $derived(flattenHeadings(headings));

  // Find active item index based on activeLine
  let activeIndex = $derived.by(() => {
    if (activeLine < 0 || flatItems.length === 0) return -1;
    // Find the last heading whose line <= activeLine
    let idx = -1;
    for (let i = 0; i < flatItems.length; i++) {
      if (flatItems[i].heading.line <= activeLine) {
        idx = i;
      } else {
        break;
      }
    }
    return idx;
  });

  // Reset selection when headings change
  $effect(() => {
    if (headings.length > 0 && selectedIndex < 0) {
      selectedIndex = 0;
    }
    if (flatItems.length === 0) {
      selectedIndex = -1;
    } else if (selectedIndex >= flatItems.length) {
      selectedIndex = flatItems.length - 1;
    }
  });

  export function focus() {
    if (panelElement) {
      panelElement.focus();
      isFocused = true;
    }
  }

  function handleFocus() {
    isFocused = true;
    onFocusChange(true);
  }

  function handleBlur() {
    isFocused = false;
    onFocusChange(false);
  }

  function toggleExpand(index: number) {
    const item = flatItems[index];
    if (!item || item.heading.children.length === 0) return;
    item.heading.expanded = !item.heading.expanded;
    // Trigger reactivity
    headings = [...headings];
  }

  function expandAll() {
    function setExpanded(items: TocHeading[], val: boolean) {
      for (const item of items) {
        if (item.children.length > 0) {
          item.expanded = val;
          setExpanded(item.children, val);
        }
      }
    }
    setExpanded(headings, true);
    headings = [...headings];
  }

  function collapseAll() {
    function setExpanded(items: TocHeading[], val: boolean) {
      for (const item of items) {
        if (item.children.length > 0) {
          item.expanded = val;
          setExpanded(item.children, val);
        }
      }
    }
    setExpanded(headings, false);
    headings = [...headings];
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isFocused) return;
    event.stopPropagation();

    // Search mode
    if (searchActive) {
      if (event.key === 'Escape') {
        event.preventDefault();
        searchActive = false;
        searchQuery = '';
        panelElement?.focus();
        return;
      }
      if (event.key === 'Enter') {
        event.preventDefault();
        searchActive = false;
        if (flatItems.length > 0) {
          const idx = selectedIndex >= 0 ? selectedIndex : 0;
          jumpToItem(idx);
        }
        searchQuery = '';
        return;
      }
      // Let input handle other keys
      return;
    }

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;

    switch (event.code) {
      case 'KeyJ':
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, flatItems.length - 1);
        scrollToSelected();
        break;
      case 'KeyK':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        scrollToSelected();
        break;
      case 'KeyG':
        if (isDoubleG) {
          event.preventDefault();
          selectedIndex = 0;
          scrollToSelected();
          lastKey = '';
          return;
        }
        if (event.shiftKey) {
          event.preventDefault();
          selectedIndex = flatItems.length - 1;
          scrollToSelected();
        }
        break;
      case 'KeyH':
        event.preventDefault();
        if (event.shiftKey) {
          collapseAll();
        } else if (selectedIndex >= 0 && selectedIndex < flatItems.length) {
          const item = flatItems[selectedIndex];
          if (item.heading.children.length > 0 && item.heading.expanded) {
            toggleExpand(selectedIndex);
          } else if (item.depth > 0) {
            // Move to parent
            for (let i = selectedIndex - 1; i >= 0; i--) {
              if (flatItems[i].depth < item.depth) {
                selectedIndex = i;
                scrollToSelected();
                break;
              }
            }
          }
        }
        break;
      case 'KeyL':
        event.preventDefault();
        if (event.shiftKey) {
          expandAll();
        } else if (selectedIndex >= 0 && selectedIndex < flatItems.length) {
          const item = flatItems[selectedIndex];
          if (item.heading.children.length > 0 && !item.heading.expanded) {
            toggleExpand(selectedIndex);
          }
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < flatItems.length) {
          jumpToItem(selectedIndex);
        }
        break;
      case 'Slash':
        event.preventDefault();
        searchActive = true;
        searchQuery = '';
        setTimeout(() => searchInput?.focus(), 0);
        break;
    }

    lastKey = event.code;
    lastKeyTime = now;
  }

  function jumpToItem(index: number) {
    const item = flatItems[index];
    if (item) {
      onJump(item.heading.line);
    }
  }

  function scrollToSelected() {
    if (!panelElement || selectedIndex < 0) return;
    requestAnimationFrame(() => {
      const el = panelElement?.querySelector(`[data-toc-index="${selectedIndex}"]`);
      const container = panelElement?.querySelector('.toc-content');
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

  function handleItemClick(index: number) {
    selectedIndex = index;
    jumpToItem(index);
  }

  function handleSearchKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      searchActive = false;
      searchQuery = '';
      panelElement?.focus();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      searchActive = false;
      if (flatItems.length > 0) {
        const idx = selectedIndex >= 0 ? selectedIndex : 0;
        jumpToItem(idx);
      }
      searchQuery = '';
    }
  }

  function getExpandIcon(item: FlatHeading): string {
    if (item.heading.children.length === 0) return ' ';
    return item.heading.expanded ? '▼' : '▶';
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="toc-sidebar"
  bind:this={panelElement}
  onkeydown={handleKeydown}
  onfocus={handleFocus}
  onblur={handleBlur}
  role="tree"
  aria-label="Table of Contents"
  tabindex="0"
>
  <div class="toc-header">
    <span class="toc-title">OUTLINE</span>
  </div>

  {#if searchActive}
    <div class="toc-search">
      <span class="search-icon">/</span>
      <input
        type="text"
        class="search-input"
        placeholder="Search headings..."
        bind:value={searchQuery}
        bind:this={searchInput}
        onkeydown={handleSearchKeydown}
      />
    </div>
  {/if}

  <div class="toc-content">
    {#if flatItems.length === 0}
      <div class="toc-empty">No headings</div>
    {:else}
      {#each flatItems as item, index (item.heading.line + ':' + item.heading.text)}
        <div
          class="toc-item"
          class:selected={index === selectedIndex}
          class:active={index === activeIndex}
          style="padding-left: {8 + item.depth * 16}px"
          data-toc-index={index}
          onclick={() => handleItemClick(index)}
          onkeydown={() => {}}
          role="treeitem"
          aria-selected={index === selectedIndex}
        >
          <span class="expand-icon">{getExpandIcon(item)}</span>
          <span class="toc-text" title={item.heading.text}>{item.heading.text}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .toc-sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-primary);
    outline: none;
    font-family: var(--font-mono);
    width: 240px;
    border-left: 1px solid var(--border);
    flex-shrink: 0;
  }

  .toc-header {
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
  }

  .toc-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .toc-search {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 4px;
  }

  .search-icon {
    color: var(--text-muted);
    font-size: 12px;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    outline: none;
    padding: 2px 0;
  }

  .toc-content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .toc-empty {
    color: var(--text-muted);
    font-size: 12px;
    text-align: center;
    margin-top: 20px;
  }

  .toc-item {
    display: flex;
    align-items: center;
    padding: 3px 8px;
    cursor: pointer;
    gap: 4px;
    transition: background-color 0.1s ease;
    font-size: 12px;
  }

  .toc-item:hover {
    background-color: var(--bg-hover);
  }

  .toc-item.selected {
    background-color: var(--bg-active);
  }

  .toc-item.active {
    border-left: 2px solid var(--accent);
    padding-left: 6px;
  }

  .expand-icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-size: 10px;
    color: var(--text-muted);
  }

  .toc-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }
</style>
