<script lang="ts">
  import type { PdfOutlineItem } from '$lib/utils/pdf-shared';

  let {
    outline = [],
    currentPage = -1,
    pageDimensions = [],
    onJump = (_page: number, _y: number) => {},
    onFocusChange = (_focused: boolean) => void 0,
    onExit = () => void 0,
  }: {
    outline: PdfOutlineItem[];
    currentPage?: number;
    pageDimensions?: { width: number; height: number }[];
    onJump?: (page: number, y: number) => void;
    onFocusChange?: (focused: boolean) => void;
    onExit?: () => void;
  } = $props();

  let selectedIndex: number = $state(-1);
  let isFocused: boolean = $state(false);
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  let lastKeyTime: number = 0;
  let lastKey: string = '';
  let ctrlWPending = false;

  // Search state
  let searchActive: boolean = $state(false);
  let searchQuery: string = $state('');
  let searchInput: HTMLInputElement | undefined = $state(undefined);

  // Tree expand state (managed locally since PdfOutlineItem doesn't have 'expanded')
  let expandedSet = $state(new Set<number>());

  interface FlatItem {
    item: PdfOutlineItem;
    depth: number;
    index: number; // unique index in flat list
    hasChildren: boolean;
    expanded: boolean;
  }

  let flatIndex = 0;
  function flattenOutline(items: PdfOutlineItem[], depth: number = 0): FlatItem[] {
    const result: FlatItem[] = [];
    for (const item of items) {
      if (searchQuery) {
        const matchesSelf = item.title.toLowerCase().includes(searchQuery.toLowerCase());
        const childResult = item.children.length > 0 ? flattenOutline(item.children, depth + 1) : [];
        if (!matchesSelf && childResult.length === 0) continue;
        const idx = flatIndex++;
        const expanded = expandedSet.has(idx) || !!searchQuery;
        result.push({ item, depth, index: idx, hasChildren: item.children.length > 0, expanded });
        if (childResult.length > 0) result.push(...childResult);
        continue;
      }
      const idx = flatIndex++;
      const expanded = expandedSet.has(idx);
      result.push({ item, depth, index: idx, hasChildren: item.children.length > 0, expanded });
      if (expanded && item.children.length > 0) {
        result.push(...flattenOutline(item.children, depth + 1));
      }
    }
    return result;
  }

  let flatItems = $derived.by(() => {
    flatIndex = 0;
    return flattenOutline(outline);
  });

  // Find active item based on currentPage
  let activeIndex = $derived.by(() => {
    if (currentPage < 0 || flatItems.length === 0) return -1;
    let idx = -1;
    for (let i = 0; i < flatItems.length; i++) {
      if (flatItems[i].item.page <= currentPage) {
        idx = i;
      } else {
        break;
      }
    }
    return idx;
  });

  // Reset selection when outline changes
  $effect(() => {
    if (outline.length > 0 && selectedIndex < 0) {
      selectedIndex = 0;
    }
    if (flatItems.length === 0) {
      selectedIndex = -1;
    } else if (selectedIndex >= flatItems.length) {
      selectedIndex = flatItems.length - 1;
    }
  });

  export function getSelectedIndex(): number { return selectedIndex; }

  export function focus() {
    if (panelElement) { panelElement.focus(); isFocused = true; }
  }

  function handleFocus() { isFocused = true; onFocusChange(true); }
  function handleBlur() { isFocused = false; onFocusChange(false); }

  function toggleExpand(index: number) {
    const item = flatItems[index];
    if (!item || !item.hasChildren) return;
    const newSet = new Set(expandedSet);
    if (newSet.has(item.index)) { newSet.delete(item.index); } else { newSet.add(item.index); }
    expandedSet = newSet;
  }

  function expandAll() {
    const newSet = new Set<number>();
    let idx = 0;
    function walk(items: PdfOutlineItem[]) {
      for (const item of items) {
        if (item.children.length > 0) {
          newSet.add(idx);
        }
        idx++;
        walk(item.children);
      }
    }
    walk(outline);
    expandedSet = newSet;
  }

  function collapseAll() { expandedSet = new Set<number>(); }

  function handleKeydown(event: KeyboardEvent) {
    if (!isFocused) return;
    if (event.ctrlKey && event.key === 'w') {
      ctrlWPending = true;
      return;
    }
    if (ctrlWPending) {
      ctrlWPending = false;
      if (event.code === 'KeyH' || event.code === 'KeyL') return;
    }
    event.stopPropagation();

    if (searchActive) {
      if (event.key === 'Escape') { event.preventDefault(); searchActive = false; searchQuery = ''; panelElement?.focus(); return; }
      if (event.key === 'Enter') { event.preventDefault(); searchActive = false; if (flatItems.length > 0) { jumpToItem(selectedIndex >= 0 ? selectedIndex : 0); } searchQuery = ''; return; }
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      onExit();
      return;
    }

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;

    switch (event.code) {
      case 'KeyJ': event.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, flatItems.length - 1); scrollToSelected(); break;
      case 'KeyK': event.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); scrollToSelected(); break;
      case 'KeyG':
        if (isDoubleG) { event.preventDefault(); selectedIndex = 0; scrollToSelected(); lastKey = ''; return; }
        if (event.shiftKey) { event.preventDefault(); selectedIndex = flatItems.length - 1; scrollToSelected(); }
        break;
      case 'KeyH':
        event.preventDefault();
        if (event.shiftKey) { collapseAll(); }
        else if (selectedIndex >= 0 && selectedIndex < flatItems.length) {
          const item = flatItems[selectedIndex];
          if (item.hasChildren && item.expanded) { toggleExpand(selectedIndex); }
          else if (item.depth > 0) {
            for (let i = selectedIndex - 1; i >= 0; i--) {
              if (flatItems[i].depth < item.depth) { selectedIndex = i; scrollToSelected(); break; }
            }
          }
        }
        break;
      case 'KeyL':
        event.preventDefault();
        if (event.shiftKey) { expandAll(); }
        else if (selectedIndex >= 0 && selectedIndex < flatItems.length) {
          const item = flatItems[selectedIndex];
          if (item.hasChildren && !item.expanded) { toggleExpand(selectedIndex); }
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < flatItems.length) { jumpToItem(selectedIndex); }
        break;
      case 'Slash':
        event.preventDefault(); searchActive = true; searchQuery = '';
        setTimeout(() => searchInput?.focus(), 0);
        break;
    }

    lastKey = event.code;
    lastKeyTime = now;
  }

  function jumpToItem(index: number) {
    const item = flatItems[index];
    if (item) { onJump(item.item.page, item.item.y); }
  }

  function scrollToSelected() {
    if (!panelElement || selectedIndex < 0) return;
    requestAnimationFrame(() => {
      const el = panelElement?.querySelector(`[data-toc-index="${selectedIndex}"]`);
      const container = panelElement?.querySelector('.toc-content');
      if (!el || !container) return;
      const cr = container.getBoundingClientRect();
      const er = el.getBoundingClientRect();
      if (er.top < cr.top) { container.scrollTop -= cr.top - er.top; }
      else if (er.bottom > cr.bottom) { container.scrollTop += er.bottom - cr.bottom; }
    });
  }

  function handleItemClick(index: number) { selectedIndex = index; jumpToItem(index); }

  function handleSearchKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') { event.preventDefault(); searchActive = false; searchQuery = ''; panelElement?.focus(); }
    else if (event.key === 'Enter') {
      event.preventDefault(); searchActive = false;
      if (flatItems.length > 0) { jumpToItem(selectedIndex >= 0 ? selectedIndex : 0); }
      searchQuery = '';
    }
  }

  function getExpandIcon(item: FlatItem): string {
    if (!item.hasChildren) return ' ';
    return item.expanded ? '▼' : '▶';
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
  aria-label="PDF Outline"
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
        placeholder="Search outline..."
        bind:value={searchQuery}
        bind:this={searchInput}
        onkeydown={handleSearchKeydown}
      />
    </div>
  {/if}

  <div class="toc-content">
    {#if flatItems.length === 0}
      <div class="toc-empty">No outline</div>
    {:else}
      {#each flatItems as item, i (item.index + ':' + item.item.title)}
        <div
          class="toc-item"
          class:selected={i === selectedIndex}
          class:active={i === activeIndex}
          style="padding-left: {8 + item.depth * 16}px"
          data-toc-index={i}
          onclick={() => handleItemClick(i)}
          onkeydown={() => {}}
          role="treeitem"
          aria-selected={i === selectedIndex}
        >
          <span class="expand-icon" class:clickable={item.hasChildren} role="button" tabindex="-1" onclick={(e) => { e.stopPropagation(); toggleExpand(i); }}>{getExpandIcon(item)}</span>
          <span class="toc-text" title={item.item.title}>{item.item.title}</span>
          <span class="toc-page">{item.item.page + 1}</span>
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
    user-select: none;
  }

  .expand-icon.clickable {
    cursor: pointer;
  }

  .expand-icon.clickable:hover {
    color: var(--text-primary);
  }

  .toc-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .toc-page {
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 10px;
    margin-left: 4px;
  }
</style>
