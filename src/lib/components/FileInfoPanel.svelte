<script lang="ts">
  import { registerModal } from '$lib/utils/modal-focus';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { formatSize } from '$lib/utils/file-types';
  import type { SelectionStatsProgress, SelectionStatsRoot } from '$lib/types/selection-stats';

  interface FileInfo {
    name: string;
    path: string;
    size: number;
    is_dir: boolean;
    created: number | null;
    modified: number | null;
    accessed: number | null;
    is_readonly: boolean;
    is_hidden: boolean;
    is_system: boolean;
    item_count: number | null;
  }

  let {
    visible = false,
    info = null as FileInfo | null,
    selectionRoots = null as SelectionStatsRoot[] | null,
    onClose = () => {},
  }: {
    visible?: boolean;
    info?: FileInfo | null;
    selectionRoots?: SelectionStatsRoot[] | null;
    onClose?: () => void;
  } = $props();

  let overlayEl: HTMLDivElement | undefined = $state(undefined);
  let folderSize: number = $state(0);
  let folderFileCount: number = $state(0);
  let isCalculating: boolean = $state(false);
  let selectionProgress: SelectionStatsProgress | null = $state(null);
  let selectionCalculating = $state(false);

  $effect(() => {
    const roots = selectionRoots;
    if (!visible || !roots?.length) return;
    const requestId = crypto.randomUUID();
    let active = true;
    let started = false;
    let unlistenTick: UnlistenFn | undefined;
    let unlistenDone: UnlistenFn | undefined;
    selectionProgress = {
      request_id: requestId, selected_items: roots.length, files: 0, dirs: 0,
      total_bytes: 0, incomplete: false, cancelled: false,
    };
    selectionCalculating = true;
    void (async () => {
      try {
        unlistenTick = await listen<SelectionStatsProgress>('selection-stats-tick', event => {
          if (active && event.payload.request_id === requestId) selectionProgress = event.payload;
        });
        if (!active) { unlistenTick(); return; }
        unlistenDone = await listen<SelectionStatsProgress>('selection-stats-done', event => {
          if (active && event.payload.request_id === requestId) {
            selectionProgress = event.payload;
            selectionCalculating = false;
          }
        });
        if (!active) { unlistenTick(); unlistenDone(); return; }
        started = true;
        await invoke('calculate_selection_stats', { requestId, roots });
      } catch (error) {
        if (active) {
          selectionCalculating = false;
          selectionProgress = { request_id: requestId, selected_items: roots.length,
            files: 0, dirs: 0, total_bytes: 0, incomplete: true, cancelled: false };
          console.error('Selection statistics failed:', error);
        }
      }
    })();
    return () => {
      active = false;
      if (started) void invoke('cancel_selection_stats', { requestId });
      unlistenTick?.();
      unlistenDone?.();
      selectionCalculating = false;
    };
  });

  $effect(() => {
    if (visible && info?.is_dir) {
      folderSize = 0;
      folderFileCount = 0;
      isCalculating = true;

      invoke('calculate_folder_size', { path: info.path });

      let unlistenTick: UnlistenFn | undefined;
      let unlistenDone: UnlistenFn | undefined;

      listen<{ path: string; total_bytes: number; files: number; dirs: number }>(
        'folder-size-tick',
        (event) => {
          if (event.payload.path === info.path) {
            folderSize = event.payload.total_bytes;
            folderFileCount = event.payload.files;
          }
        }
      ).then(fn => { unlistenTick = fn; });

      listen<{ path: string; total_bytes: number; files: number; dirs: number }>(
        'folder-size-done',
        (event) => {
          if (event.payload.path === info.path) {
            folderSize = event.payload.total_bytes;
            folderFileCount = event.payload.files;
            isCalculating = false;
          }
        }
      ).then(fn => { unlistenDone = fn; });

      return () => {
        invoke('cancel_folder_size', { path: info!.path });
        if (unlistenTick) unlistenTick();
        if (unlistenDone) unlistenDone();
        isCalculating = false;
      };
    }
  });

  $effect(() => {
    if (visible && overlayEl) {
      return registerModal(overlayEl);
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    onClose();
  }

  function formatTimestamp(value: number | null): string | null {
    if (value === null || !Number.isFinite(value)) return null;
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? null : date.toLocaleString('zh-CN', { hour12: false });
  }

  const createdTime = $derived(formatTimestamp(info?.created ?? null));
  const modifiedTime = $derived(formatTimestamp(info?.modified ?? null));
  const accessedTime = $derived(formatTimestamp(info?.accessed ?? null));
</script>

{#if visible && (info || selectionRoots)}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="file-info-overlay" bind:this={overlayEl} tabindex="-1" role="dialog" aria-modal="true" aria-label="File information" onkeydown={handleKeydown} onclick={onClose}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="file-info-panel" onclick={(e) => e.stopPropagation()}>
      {#if selectionRoots}
      <div class="info-header">Selected items</div>
      <div class="info-rows">
        <div class="info-row"><span class="info-label">Selected</span><span class="info-value">{selectionProgress?.selected_items ?? selectionRoots.length}</span></div>
        <div class="info-row"><span class="info-label">Files</span><span class="info-value">{selectionProgress?.files ?? 0}</span></div>
        <div class="info-row"><span class="info-label">Directories</span><span class="info-value">{selectionProgress?.dirs ?? 0}</span></div>
        <div class="info-row"><span class="info-label">Size</span><span class="info-value">{formatSize(selectionProgress?.total_bytes ?? 0)}</span></div>
        {#if selectionCalculating}<div class="info-row"><span class="info-label">Status</span><span class="info-value">Calculating…</span></div>{/if}
        {#if selectionProgress?.incomplete}<div class="info-row"><span class="info-label">Status</span><span class="info-value">Incomplete: some items could not be read</span></div>{/if}
      </div>
      {:else if info}
      <div class="info-header">{info.name}{info.is_dir ? '/' : ''}</div>
      <div class="info-rows">
        <div class="info-row">
          <span class="info-label">Path</span>
          <span class="info-value path">{info.path}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Size</span>
          <span class="info-value">
            {#if info.is_dir && isCalculating}
              {formatSize(folderSize)}
            {:else if info.is_dir && !isCalculating && folderSize > 0}
              {formatSize(folderSize)}
            {:else}
              {formatSize(info.size)}
            {/if}
          </span>
        </div>
        {#if info.is_dir && (isCalculating || folderSize > 0)}
          <div class="info-row">
            <span class="info-label">Files</span>
            <span class="info-value">{folderFileCount.toLocaleString()}</span>
          </div>
        {/if}
        {#if info.is_dir && info.item_count !== null}
          <div class="info-row">
            <span class="info-label">Items</span>
            <span class="info-value">{info.item_count}</span>
          </div>
        {/if}
        {#if createdTime}
          <div class="info-row">
            <span class="info-label">Created</span>
            <span class="info-value">{createdTime}</span>
          </div>
        {/if}
        {#if modifiedTime}
          <div class="info-row">
            <span class="info-label">Modified</span>
            <span class="info-value">{modifiedTime}</span>
          </div>
        {/if}
        {#if accessedTime}
          <div class="info-row">
            <span class="info-label">Accessed</span>
            <span class="info-value">{accessedTime}</span>
          </div>
        {/if}
        <div class="info-row">
          <span class="info-label">Type</span>
          <span class="info-value">{info.is_dir ? 'Directory' : 'File'}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Attributes</span>
          <span class="info-value">
            {#if info.is_readonly}<span class="attr">readonly</span>{/if}
            {#if info.is_hidden}<span class="attr">hidden</span>{/if}
            {#if info.is_system}<span class="attr">system</span>{/if}
            {#if !info.is_readonly && !info.is_hidden && !info.is_system}—{/if}
          </span>
        </div>
      </div>
      {/if}
      <div class="info-hint">Press any key to close</div>
    </div>
  </div>
{/if}

<style>
  .file-info-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .file-info-panel {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 16px 20px;
    min-width: 400px;
    max-width: 600px;
    font-family: var(--font-mono);
    font-size: 13px;
    zoom: var(--zoom-level);
  }

  .info-header {
    font-size: 14px;
    font-weight: bold;
    color: var(--accent);
    margin-bottom: 12px;
    word-break: break-all;
  }

  .info-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .info-row {
    display: flex;
    gap: 12px;
  }

  .info-label {
    color: var(--text-muted);
    min-width: 80px;
    flex-shrink: 0;
  }

  .info-value {
    color: var(--text-primary);
    word-break: break-all;
  }

  .info-value.path {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .attr {
    display: inline-block;
    padding: 0 6px;
    margin-right: 4px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .info-hint {
    margin-top: 12px;
    color: var(--text-muted);
    font-size: 11px;
    text-align: center;
  }
</style>
