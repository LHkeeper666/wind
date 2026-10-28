<script module lang="ts">
  export interface OutlineNode {
    id: string;
    label: string;
    anchor: number;
    children: OutlineNode[];
  }

  export function makeOutlineId(parentId: string | null, index: number): string {
    return parentId === null ? `${index}` : `${parentId}.${index}`;
  }
</script>

<script lang="ts">
  let {
    nodes = [],
    expandedIds = new Set<string>(),
    activeAnchor = -1,
    onJump = (_node: OutlineNode) => {},
    onToggle = (_id: string) => {},
    onFocusChange = (_focused: boolean) => {},
    onExit = undefined,
    title = 'OUTLINE',
    searchPlaceholder = 'Search outline...',
    emptyText = 'No outline',
    showAnchor = false,
    passThroughCtrlW = false,
  }: {
    nodes: OutlineNode[];
    expandedIds: Set<string>;
    activeAnchor: number;
    onJump?: (node: OutlineNode) => void;
    onToggle?: (id: string) => void;
    onFocusChange?: (focused: boolean) => void;
    onExit?: () => void;
    title?: string;
    searchPlaceholder?: string;
    emptyText?: string;
    showAnchor?: boolean;
    passThroughCtrlW?: boolean;
  } = $props();

  interface FlatItem {
    node: OutlineNode;
    depth: number;
    hasChildren: boolean;
    expanded: boolean;
  }

  let selectedId: string | null = $state(null);
  let pendingSelectedIndex: number = $state(-1);
  let isFocused: boolean = $state(false);
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  let lastKeyTime: number = 0;
  let lastKey: string = '';
  let ctrlWPending = false;

  let searchActive: boolean = $state(false);
  let searchMode: 'input' | 'list' = $state('input');
  let searchQuery: string = $state('');
  let searchInput: HTMLInputElement | undefined = $state(undefined);

  function flattenTree(items: OutlineNode[], depth: number): FlatItem[] {
    const result: FlatItem[] = [];
    for (const node of items) {
      const hasChildren = node.children.length > 0;
      const expanded = hasChildren && expandedIds.has(node.id);
      result.push({ node, depth, hasChildren, expanded });
      if (expanded) result.push(...flattenTree(node.children, depth + 1));
    }
    return result;
  }

  function flattenSearch(items: OutlineNode[], depth: number, query: string): FlatItem[] {
    const result: FlatItem[] = [];
    for (const node of items) {
      const matchesSelf = node.label.toLowerCase().includes(query);
      const childItems = node.children.length > 0
        ? flattenSearch(node.children, depth + 1, query)
        : [];
      if (!matchesSelf && childItems.length === 0) continue;
      result.push({
        node,
        depth,
        hasChildren: node.children.length > 0,
        expanded: childItems.length > 0,
      });
      result.push(...childItems);
    }
    return result;
  }

  let flatItems = $derived.by(() => {
    const query = searchQuery.trim().toLowerCase();
    return query ? flattenSearch(nodes, 0, query) : flattenTree(nodes, 0);
  });

  let selectedIndex = $derived.by(() => {
    if (pendingSelectedIndex >= 0) return pendingSelectedIndex;
    if (!selectedId) return -1;
    return flatItems.findIndex(item => item.node.id === selectedId);
  });

  let activeIndex = $derived.by(() => {
    if (activeAnchor < 0 || flatItems.length === 0) return -1;
    let idx = -1;
    for (let i = 0; i < flatItems.length; i++) {
      if (flatItems[i].node.anchor <= activeAnchor) idx = i;
    }
    return idx;
  });

  function nearestVisibleAncestor(id: string): string | null {
    let path = id;
    while (path.includes('.')) {
      path = path.slice(0, path.lastIndexOf('.'));
      if (flatItems.some(item => item.node.id === path)) return path;
    }
    return null;
  }

  // Selection is tracked by stable id; the visible index is a projection so
  // restoring "item N" before expansions are applied is not clamped away.
  $effect(() => {
    const items = flatItems;
    if (items.length === 0) {
      selectedId = null;
      return;
    }
    if (pendingSelectedIndex >= 0) {
      if (pendingSelectedIndex < items.length) {
        selectedId = items[pendingSelectedIndex].node.id;
        pendingSelectedIndex = -1;
      }
      return;
    }
    if (selectedId === null) {
      selectedId = items[0].node.id;
      return;
    }
    if (items.some(item => item.node.id === selectedId)) return;
    const ancestor = nearestVisibleAncestor(selectedId);
    selectedId = ancestor ?? items[0].node.id;
  });

  export function getSelectedIndex(): number {
    return selectedIndex;
  }

  export function setSelectedIndex(index: number): void {
    if (index < 0) return;
    if (index < flatItems.length) {
      selectIndex(index);
    } else {
      pendingSelectedIndex = index;
    }
  }

  export function getSelectedId(): string | null {
    return selectedId;
  }

  export function setSelectedId(id: string): void {
    pendingSelectedIndex = -1;
    if (flatItems.some(item => item.node.id === id)) {
      selectedId = id;
      return;
    }
    selectedId = nearestVisibleAncestor(id) ?? flatItems[0]?.node.id ?? null;
  }

  export function focus(): boolean {
    if (!panelElement) return false;
    panelElement.focus({ preventScroll: true });
    return document.activeElement === panelElement;
  }

  export function getScrollTop(): number {
    return panelElement?.querySelector('.toc-content')?.scrollTop ?? 0;
  }

  export function setScrollTop(top: number): void {
    const content = panelElement?.querySelector('.toc-content');
    if (content) content.scrollTop = Math.max(0, top);
  }

  // Focus moving between the panel and its own search input is not a focus
  // change for the parent: reporting it makes the parent refocus the preview.
  function handleFocusIn() {
    if (isFocused) return;
    isFocused = true;
    onFocusChange(true);
  }

  function handleFocusOut(event: FocusEvent) {
    const next = event.relatedTarget as Node | null;
    if (next && panelElement?.contains(next)) return;
    if (!isFocused) return;
    isFocused = false;
    onFocusChange(false);
  }

  function selectIndex(index: number, scroll = false) {
    const item = flatItems[index];
    if (!item) return;
    selectedId = item.node.id;
    pendingSelectedIndex = -1;
    if (scroll) scrollToSelected();
  }

  function collectExpandableIds(items: OutlineNode[], out: string[]) {
    for (const node of items) {
      if (node.children.length > 0) {
        out.push(node.id);
        collectExpandableIds(node.children, out);
      }
    }
  }

  function expandAll() {
    const ids: string[] = [];
    collectExpandableIds(nodes, ids);
    for (const id of ids) {
      if (!expandedIds.has(id)) onToggle(id);
    }
  }

  function collapseAll() {
    const ids: string[] = [];
    collectExpandableIds(nodes, ids);
    for (const id of ids) {
      if (expandedIds.has(id)) onToggle(id);
    }
  }

  function exitSearch() {
    searchActive = false;
    searchMode = 'input';
    searchQuery = '';
    panelElement?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isFocused) return;

    if (passThroughCtrlW) {
      if (event.ctrlKey && event.key === 'w') {
        ctrlWPending = true;
        return;
      }
      if (ctrlWPending) {
        ctrlWPending = false;
        if (event.code === 'KeyH' || event.code === 'KeyL' || event.code === 'KeyM') return;
      }
    }
    event.stopPropagation();

    if (searchActive) {
      if (searchMode === 'input') return;
      if (event.key === 'Escape') {
        event.preventDefault();
        searchMode = 'input';
        setTimeout(() => searchInput?.focus(), 0);
        return;
      }
      // Enter and navigation keys fall through to the normal handling below.
    }

    if (event.key === 'Escape') {
      if (onExit) {
        event.preventDefault();
        onExit();
      }
      return;
    }

    const now = Date.now();
    const isDoubleG = lastKey === 'KeyG' && event.code === 'KeyG' && now - lastKeyTime < 500;
    const idx = getSelectedIndex();

    switch (event.code) {
      case 'KeyJ':
        event.preventDefault();
        selectIndex(Math.min(idx + 1, flatItems.length - 1), true);
        break;
      case 'KeyK':
        event.preventDefault();
        selectIndex(Math.max(idx - 1, 0), true);
        break;
      case 'KeyG':
        if (isDoubleG) {
          event.preventDefault();
          selectIndex(0, true);
          lastKey = '';
          return;
        }
        if (event.shiftKey) {
          event.preventDefault();
          selectIndex(flatItems.length - 1, true);
        }
        break;
      case 'KeyH':
        event.preventDefault();
        if (event.shiftKey) {
          collapseAll();
        } else if (idx >= 0 && idx < flatItems.length) {
          const item = flatItems[idx];
          if (item.hasChildren && item.expanded) {
            onToggle(item.node.id);
          } else if (item.depth > 0) {
            for (let i = idx - 1; i >= 0; i--) {
              if (flatItems[i].depth < item.depth) {
                selectIndex(i, true);
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
        } else if (idx >= 0 && idx < flatItems.length) {
          const item = flatItems[idx];
          if (item.hasChildren && !item.expanded) {
            onToggle(item.node.id);
          }
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (idx >= 0 && idx < flatItems.length) {
          const node = flatItems[idx].node;
          if (searchActive) {
            searchActive = false;
            searchMode = 'input';
            searchQuery = '';
          }
          onJump(node);
        }
        break;
      case 'Slash':
        event.preventDefault();
        if (!searchActive) searchQuery = '';
        searchActive = true;
        searchMode = 'input';
        setTimeout(() => searchInput?.focus(), 0);
        break;
    }

    lastKey = event.code;
    lastKeyTime = now;
  }

  function jumpToIndex(index: number) {
    const item = flatItems[index];
    if (item) onJump(item.node);
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
    selectIndex(index);
    jumpToIndex(index);
  }

  function handleSearchKeydown(event: KeyboardEvent) {
    // Keep keystrokes typed into the filter from reaching panel/global handlers.
    event.stopPropagation();
    if (event.key === 'Escape') {
      event.preventDefault();
      exitSearch();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (flatItems.length === 0) return;
      searchMode = 'list';
      selectIndex(0, true);
      panelElement?.focus();
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
  onfocusin={handleFocusIn}
  onfocusout={handleFocusOut}
  role="tree"
  aria-label={title}
  tabindex="0"
>
  <div class="toc-header">
    <span class="toc-title">{title}</span>
  </div>

  {#if searchActive}
    <div class="toc-search">
      <span class="search-icon">/</span>
      <input
        type="text"
        class="search-input"
        placeholder={searchPlaceholder}
        bind:value={searchQuery}
        bind:this={searchInput}
        onkeydown={handleSearchKeydown}
      />
    </div>
  {/if}

  <div class="toc-content">
    {#if flatItems.length === 0}
      <div class="toc-empty">{emptyText}</div>
    {:else}
      {#each flatItems as item, index (item.node.id)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
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
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <span
            class="expand-icon"
            class:clickable={item.hasChildren}
            role="button"
            tabindex="-1"
            onclick={(e) => { e.stopPropagation(); if (item.hasChildren) onToggle(item.node.id); }}
          >{getExpandIcon(item)}</span>
          <span class="toc-text" title={item.node.label}>{item.node.label}</span>
          {#if showAnchor}
            <span class="toc-page">{item.node.anchor + 1}</span>
          {/if}
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
