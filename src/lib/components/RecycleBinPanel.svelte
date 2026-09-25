<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, tick } from 'svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import { formatSize } from '$lib/utils/file-types';

  interface TrashItem {
    id: string;
    name: string;
    original_path: string;
    date_deleted: number;
    size: number | null;
  }

  let {
    onPreview = (id: string, item: TrashItem) => {},
    onExit = () => {},
    onToast = (message: string) => {},
  }: {
    onPreview?: (id: string, item: TrashItem) => void;
    onExit?: () => void;
    onToast?: (message: string) => void;
  } = $props();

  let items: TrashItem[] = $state([]);
  let isLoading: boolean = $state(true);
  let errorMessage: string = $state('');
  let selectedIndex: number = $state(-1);
  let panelElement: HTMLDivElement | undefined = $state(undefined);
  let isFocused: boolean = $state(false);

  // Multi-select
  let selectedIds: Set<string> = $state(new Set());

  // Confirmation dialogs
  let showPurgeConfirm: boolean = $state(false);
  let showEmptyConfirm: boolean = $state(false);
  let showRestoreCollision: boolean = $state(false);
  let collisionPath: string = $state('');
  let pendingRestoreIds: string[] = $state([]);

  // File info panel
  let showInfo: boolean = $state(false);
  let infoItem: TrashItem | null = $state(null);
  let infoOverlay: HTMLDivElement | undefined = $state(undefined);

  $effect(() => {
    if (showInfo && infoOverlay) {
      infoOverlay.focus();
    }
  });

  // g key prefix for gd (empty recycle bin)
  let waitingForG: boolean = $state(false);
  let gKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  function formatDate(timestamp: number): string {
    if (timestamp <= 0) return 'Unknown';
    const d = new Date(timestamp * 1000);
    return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
      + ' ' + d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  }

  function getTotalSize(): string {
    let total = 0;
    for (const item of items) {
      if (item.size) total += item.size;
    }
    return formatSize(total);
  }

  async function loadItems() {
    isLoading = true;
    errorMessage = '';
    try {
      items = await invoke<TrashItem[]>('list_recycle_bin');
      selectedIndex = items.length > 0 ? 0 : -1;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      isLoading = false;
    }
  }

  function getSelectedIds(): string[] {
    if (selectedIds.size > 0) {
      return Array.from(selectedIds);
    }
    if (selectedIndex >= 0 && selectedIndex < items.length) {
      return [items[selectedIndex].id];
    }
    return [];
  }

  function selectFile(index: number) {
    if (index < 0 || index >= items.length) return;
    selectedIndex = index;
    const item = items[index];
    onPreview(item.id, item);
    scrollToIndex(index);
  }

  function scrollToIndex(index: number) {
    tick().then(() => {
      const container = panelElement?.querySelector('.panel-content');
      const el = container?.querySelector(`[data-index="${index}"]`);
      el?.scrollIntoView({ block: 'nearest' });
    });
  }

  async function handleRestore() {
    const ids = getSelectedIds();
    if (ids.length === 0) return;
    try {
      await invoke('restore_recycle_items', { ids });
      onToast(`Restored ${ids.length} item(s)`);
      await loadItems();
      selectedIds = new Set();
      panelElement?.focus();
    } catch (e) {
      const msg = String(e);
      if (msg.includes('RestoreCollision')) {
        collisionPath = msg;
        pendingRestoreIds = ids;
        showRestoreCollision = true;
      } else {
        onToast(`Restore failed: ${msg}`);
        panelElement?.focus();
      }
    }
  }

  async function handlePurgeConfirmed() {
    showPurgeConfirm = false;
    const ids = getSelectedIds();
    if (ids.length === 0) { panelElement?.focus(); return; }
    try {
      await invoke('purge_recycle_items', { ids });
      onToast(`Permanently deleted ${ids.length} item(s)`);
      await loadItems();
      selectedIds = new Set();
    } catch (e) {
      onToast(`Delete failed: ${e}`);
    }
    panelElement?.focus();
  }

  async function handleEmptyConfirmed() {
    showEmptyConfirm = false;
    try {
      await invoke('empty_recycle_bin');
      onToast('Recycle bin emptied');
      await loadItems();
    } catch (e) {
      onToast(`Empty failed: ${e}`);
    }
    panelElement?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable) {
      return;
    }

    if (waitingForG) {
      waitingForG = false;
      if (gKeyTimeout) { clearTimeout(gKeyTimeout); gKeyTimeout = null; }
      if (event.key === 'd') {
        event.preventDefault();
        if (items.length > 0) showEmptyConfirm = true;
        return;
      }
      return;
    }

    // j/k navigation
    if (event.key === 'j' || event.key === 'ArrowDown') {
      event.preventDefault();
      if (items.length === 0) return;
      selectFile(Math.min(selectedIndex + 1, items.length - 1));
      return;
    }
    if (event.key === 'k' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (items.length === 0) return;
      selectFile(Math.max(selectedIndex - 1, 0));
      return;
    }

    // gg / G
    if (event.key === 'g' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      waitingForG = true;
      gKeyTimeout = setTimeout(() => { waitingForG = false; }, 1000);
      return;
    }
    if (event.key === 'G' && !event.ctrlKey && !event.shiftKey) {
      event.preventDefault();
      if (items.length === 0) return;
      selectFile(items.length - 1);
      return;
    }

    // Enter: preview
    if (event.key === 'Enter') {
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < items.length) {
        const item = items[selectedIndex];
        onPreview(item.id, item);
      }
      return;
    }

    // Space: multi-select toggle
    if (event.key === ' ') {
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < items.length) {
        const id = items[selectedIndex].id;
        if (selectedIds.has(id)) {
          selectedIds.delete(id);
        } else {
          selectedIds.add(id);
        }
        selectedIds = new Set(selectedIds);
      }
      return;
    }

    // i: file info
    if (event.key === 'i') {
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < items.length) {
        infoItem = items[selectedIndex];
        showInfo = true;
      }
      return;
    }

    // r: restore
    if (event.key === 'r') {
      event.preventDefault();
      handleRestore();
      return;
    }

    // d: permanent delete (purge)
    if (event.key === 'd' && !event.ctrlKey) {
      event.preventDefault();
      const ids = getSelectedIds();
      if (ids.length > 0) showPurgeConfirm = true;
      return;
    }

    // R: refresh
    if (event.key === 'R' && !event.ctrlKey && !event.altKey) {
      event.preventDefault();
      loadItems();
      return;
    }

    // h: exit recycle bin
    if (event.key === 'h') {
      event.preventDefault();
      onExit();
      return;
    }
  }

  function handleFocus() {
    isFocused = true;
    if (items.length > 0 && selectedIndex < 0) {
      selectedIndex = 0;
    }
  }

  function handleBlur() {
    isFocused = false;
  }

  export function focus() {
    panelElement?.focus();
    if (items.length > 0 && selectedIndex < 0) {
      selectedIndex = 0;
    }
  }

  export function reload() {
    loadItems();
  }

  onMount(() => {
    loadItems();
    // Auto-focus after mount
    tick().then(() => panelElement?.focus());
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="recycle-bin-panel"
  bind:this={panelElement}
  onkeydown={handleKeydown}
  onfocus={handleFocus}
  onblur={handleBlur}
  onclick={(e) => (e.currentTarget as HTMLDivElement).focus()}
  role="listbox"
  aria-label="Recycle Bin"
  tabindex="0"
>
  <div class="panel-header recycle-header">
    <span class="header-icon">&#x1F5D1;</span>
    <span class="header-title">Recycle Bin</span>
    {#if !isLoading && !errorMessage}
      <span class="header-stats">{items.length} items · {getTotalSize()}</span>
    {/if}
  </div>

  <div class="panel-content">
    {#if isLoading}
      <p class="placeholder">Loading...</p>
    {:else if errorMessage}
      <p class="error">{errorMessage}</p>
    {:else if items.length === 0}
      <p class="placeholder">Recycle bin is empty</p>
    {:else}
      <div class="file-list">
        {#each items as item, index (item.id)}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div
            class="file-item"
            class:selected={index === selectedIndex}
            class:multi-selected={selectedIds.has(item.id)}
            onclick={() => selectFile(index)}
            onkeydown={() => {}}
            data-index={index}
          >
            <span class="file-name">{item.name}</span>
            <span class="file-date">{formatDate(item.date_deleted)}</span>
            <span class="file-size">{formatSize(item.size) || '—'}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- File Info Panel -->
{#if showInfo && infoItem}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="info-overlay"
    bind:this={infoOverlay}
    onclick={() => { showInfo = false; panelElement?.focus(); }}
    onkeydown={(e) => { if (e.key === 'Escape' || e.key === 'i') { showInfo = false; panelElement?.focus(); } }}
    role="dialog"
    aria-label="File Info"
    tabindex="-1"
  >
    <div class="info-modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <div class="info-header">File Info</div>
      <div class="info-body">
        <div class="info-row">
          <span class="info-label">Name:</span>
          <span class="info-value">{infoItem.name}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Original path:</span>
          <span class="info-value" title={infoItem.original_path}>{infoItem.original_path}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Current path:</span>
          <span class="info-value" title={infoItem.id}>{infoItem.id}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Deleted:</span>
          <span class="info-value">{formatDate(infoItem.date_deleted)}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Size:</span>
          <span class="info-value">{formatSize(infoItem.size) || '—'}</span>
        </div>
      </div>
      <div class="info-footer">
        <span class="info-hint">Press <kbd>i</kbd> or <kbd>Esc</kbd> to close</span>
      </div>
    </div>
  </div>
{/if}

<ConfirmModal
  visible={showPurgeConfirm}
  title="Permanent Delete"
  fileName={selectedIds.size > 1 ? `${selectedIds.size} items` : items[selectedIndex]?.name ?? 'item'}
  buttons={[
    { key: 'D', label: 'elete', action: handlePurgeConfirmed, style: 'danger' as const },
    { key: 'C', label: 'ancel', action: () => { showPurgeConfirm = false; } },
  ]}
/>

<ConfirmModal
  visible={showEmptyConfirm}
  title="Empty Recycle Bin"
  fileName="all items"
  buttons={[
    { key: 'D', label: 'elete all', action: handleEmptyConfirmed, style: 'danger' as const },
    { key: 'C', label: 'ancel', action: () => { showEmptyConfirm = false; } },
  ]}
/>

<ConfirmModal
  visible={showRestoreCollision}
  title="Restore Collision"
  fileName={collisionPath}
  buttons={[
    { key: 'S', label: 'kip conflicting', action: async () => {
      showRestoreCollision = false;
      try {
        const filtered = pendingRestoreIds.filter(id => {
          const item = items.find(i => i.id === id);
          return item && item.original_path !== collisionPath;
        });
        if (filtered.length > 0) {
          await invoke('restore_recycle_items', { ids: filtered });
          onToast(`Restored ${filtered.length} item(s)`);
        }
        await loadItems();
        selectedIds = new Set();
      } catch (e) {
        onToast(`Restore failed: ${e}`);
      }
      panelElement?.focus();
    }, style: 'primary' as const },
    { key: 'A', label: 'bort', action: () => {
      showRestoreCollision = false;
      onToast('Restore cancelled');
      panelElement?.focus();
    } },
  ]}
/>

<style>
  .recycle-bin-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    color: var(--text-color);
    outline: none;
    overflow: hidden;
  }

  .panel-header {
    flex-shrink: 0;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 6px;
    user-select: none;
  }

  .recycle-header {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-left: 3px solid var(--warning);
  }

  .header-icon {
    font-size: 14px;
  }

  .header-stats {
    margin-left: auto;
    font-weight: 400;
    opacity: 0.8;
    font-size: 11px;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .placeholder {
    padding: 16px;
    color: var(--text-muted);
    text-align: center;
    font-size: 13px;
  }

  .error {
    padding: 16px;
    color: var(--error-color);
    text-align: center;
    font-size: 13px;
  }

  .file-list {
    display: flex;
    flex-direction: column;
  }

  .file-item {
    display: grid;
    grid-template-columns: 1fr 130px 70px;
    gap: 8px;
    align-items: center;
    padding: 2px 10px;
    font-size: 13px;
    cursor: pointer;
    user-select: none;
    transition: background-color 0.1s ease;
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

  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }

  .file-date {
    font-size: 11px;
    color: var(--text-muted);
    text-align: right;
    white-space: nowrap;
  }

  .file-size {
    font-size: 11px;
    color: var(--text-muted);
    text-align: right;
    white-space: nowrap;
  }

  /* File info overlay */
  .info-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
  }

  .info-modal {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    min-width: 480px;
    max-width: 640px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .info-header {
    padding: 10px 14px;
    font-size: 13px;
    font-weight: 600;
    border-bottom: 1px solid var(--border);
  }

  .info-body {
    padding: 12px 14px;
    max-height: 50vh;
    overflow-y: auto;
  }

  .info-row {
    display: flex;
    gap: 10px;
    margin-bottom: 8px;
    font-size: 13px;
  }

  .info-label {
    color: var(--text-muted);
    white-space: nowrap;
    min-width: 100px;
    flex-shrink: 0;
  }

  .info-value {
    word-break: break-all;
    white-space: normal;
    line-height: 1.4;
  }

  .info-footer {
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    text-align: right;
  }

  .info-hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  .info-hint kbd {
    display: inline-block;
    padding: 1px 4px;
    font-size: 10px;
    font-family: var(--font-mono);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 3px;
  }
</style>