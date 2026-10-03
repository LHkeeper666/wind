<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { transfer, transferBatches, activeTransferCount } from '$lib/stores/transfer';
  import type { TransferEntry, TransferBatch } from '$lib/stores/transfer';

  let {
    visible = false,
    onClose = () => {},
  }: {
    visible: boolean;
    onClose?: () => void;
  } = $props();

  let panelHeight: number = $state(250);
  let isDragging: boolean = $state(false);
  let dragStartY: number = 0;
  let dragStartHeight: number = 0;
  let container: HTMLDivElement | undefined = $state(undefined);
  let selectedId: number | null = $state(null);
  let autoScroll: boolean = $state(true);
  let dragItemId: number | null = $state(null);
  let dragOverIndex: number | null = $state(null);

  let entries = $state<TransferEntry[]>([]);
  let batches = $state<TransferBatch[]>([]);
  let activeCount = $state(0);

  $effect(() => {
    const unsub1 = transfer.subscribe(v => entries = v);
    const unsub2 = transferBatches.subscribe(v => batches = v);
    const unsub3 = activeTransferCount.subscribe(v => activeCount = v);
    return () => { unsub1(); unsub2(); unsub3(); };
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;
    e.stopPropagation();

    const queuedEntries = entries.filter(en => en.status === 'queued');

    if (e.key === 'Escape') {
      onClose();
      return;
    }

    if (e.key === 'j' || e.key === 'k') {
      e.preventDefault();
      const active = entries.filter(en => en.status === 'queued' || en.status === 'running');
      const allItems = [...entries];
      const currentIdx = allItems.findIndex(en => en.id === selectedId);
      let nextIdx: number;
      if (e.key === 'j') {
        nextIdx = currentIdx < 0 ? 0 : Math.min(allItems.length - 1, currentIdx + 1);
      } else {
        nextIdx = currentIdx < 0 ? allItems.length - 1 : Math.max(0, currentIdx - 1);
      }
      if (allItems[nextIdx]) selectedId = allItems[nextIdx].id;
      return;
    }

    if (e.key === 'g' && !e.ctrlKey && !e.metaKey) {
      if ((window as any).__transferGGPending) {
        selectedId = entries[0]?.id ?? null;
        (window as any).__transferGGPending = false;
      } else {
        (window as any).__transferGGPending = true;
        setTimeout(() => { (window as any).__transferGGPending = false; }, 500);
      }
      return;
    }

    if (e.key === 'G') {
      e.preventDefault();
      selectedId = entries[entries.length - 1]?.id ?? null;
      scrollToBottom();
      return;
    }

    if (e.key === 'c') {
      const entry = entries.find(en => en.id === selectedId);
      if (entry && (entry.status === 'queued' || entry.status === 'running')) {
        transfer.cancelTransfer(entry.id);
      }
      return;
    }

    if (e.key === 'x') {
      const entry = entries.find(en => en.id === selectedId);
      if (entry && (entry.status === 'done' || entry.status === 'failed' || entry.status === 'cancelled')) {
        transfer.clearTransfer(entry.id);
        selectedId = null;
      }
      return;
    }

    if (e.ctrlKey && e.shiftKey && (e.key === 'J' || e.key === 'K')) {
      e.preventDefault();
      const entry = entries.find(en => en.id === selectedId);
      if (!entry || entry.status !== 'queued') return;
      const curIdx = queuedEntries.findIndex(en => en.id === entry.id);
      if (curIdx < 0) return;
      let newIdx = e.key === 'J' ? curIdx + 1 : curIdx - 1;
      newIdx = Math.max(0, Math.min(queuedEntries.length - 1, newIdx));
      if (newIdx === curIdx) return;
      const newOrder = [...queuedEntries];
      const [moved] = newOrder.splice(curIdx, 1);
      newOrder.splice(newIdx, 0, moved);
      transfer.reorderTransfers(newOrder.map(en => en.id));
      return;
    }
  }

  function startDrag(event: MouseEvent) {
    isDragging = true;
    dragStartY = event.clientY;
    dragStartHeight = panelHeight;
    window.addEventListener('mousemove', handleDrag);
    window.addEventListener('mouseup', stopDrag);
  }

  let dragRAF: number | null = null;
  function handleDrag(event: MouseEvent) {
    if (!isDragging || dragRAF !== null) return;
    dragRAF = requestAnimationFrame(() => {
      dragRAF = null;
      const delta = dragStartY - event.clientY;
      panelHeight = Math.max(100, Math.min(window.innerHeight * 0.8, dragStartHeight + delta));
    });
  }

  function stopDrag() {
    isDragging = false;
    if (dragRAF !== null) { cancelAnimationFrame(dragRAF); dragRAF = null; }
    window.removeEventListener('mousemove', handleDrag);
    window.removeEventListener('mouseup', stopDrag);
  }

  async function cancelAll() {
    await transfer.cancelAllTransfers();
  }

  function scrollToBottom() {
    autoScroll = true;
    if (container) container.scrollTop = container.scrollHeight;
  }

  function handleScroll() {
    if (!container) return;
    const distFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight;
    autoScroll = distFromBottom < 30;
  }

  // HTML5 drag-and-drop for queue reordering
  function handleItemDragStart(e: DragEvent, entry: TransferEntry) {
    if (entry.status !== 'queued') {
      e.preventDefault();
      return;
    }
    dragItemId = entry.id;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(entry.id));
    }
  }

  function handleItemDragOver(e: DragEvent, targetEntry: TransferEntry) {
    if (dragItemId === null || targetEntry.status !== 'queued' || targetEntry.id === dragItemId) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dragOverIndex = entries.findIndex(en => en.id === targetEntry.id);
  }

  function handleItemDrop(e: DragEvent, targetEntry: TransferEntry) {
    e.preventDefault();
    if (dragItemId === null || targetEntry.status !== 'queued') return;
    const queuedEntries = entries.filter(en => en.status === 'queued');
    const draggedIdx = queuedEntries.findIndex(en => en.id === dragItemId);
    const targetIdx = queuedEntries.findIndex(en => en.id === targetEntry.id);
    if (draggedIdx < 0 || targetIdx < 0 || draggedIdx === targetIdx) return;
    const newOrder = [...queuedEntries];
    const [moved] = newOrder.splice(draggedIdx, 1);
    newOrder.splice(targetIdx, 0, moved);
    transfer.reorderTransfers(newOrder.map(en => en.id));
    dragItemId = null;
    dragOverIndex = null;
  }

  function handleItemDragEnd() {
    dragItemId = null;
    dragOverIndex = null;
  }

  function formatElapsed(ms: number): string {
    if (ms < 1000) return '<1s';
    const sec = Math.floor(ms / 1000);
    if (sec < 60) return `${sec}s`;
    const min = Math.floor(sec / 60);
    return `${min}m ${sec % 60}s`;
  }

  function getStatusIcon(entry: TransferEntry): string {
    switch (entry.status) {
      case 'done': return '✓';
      case 'failed': return '✕';
      case 'cancelled': return '⊘';
      case 'running': return entry.opType === 'extract' ? '📦' : entry.opType.includes('download') || entry.opType === 'copy' ? '↓' : '↑';
      case 'queued': return entry.opType === 'extract' ? '📦' : '○';
    }
  }

  function progressPercent(entry: TransferEntry): number {
    if (entry.totalBytes > 0) return Math.min(100, (entry.bytesDone / entry.totalBytes) * 100);
    return 0;
  }

  let visibleEntries = $derived(entries);
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="transfer-panel"
    style="height: {panelHeight}px;"
    onkeydown={handleKeydown}
    tabindex="-1"
    bind:this={container}
    onscroll={handleScroll}
  >
    <!-- Drag handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="transfer-handle" onmousedown={startDrag} role="separator" aria-orientation="horizontal"></div>

    <!-- Header -->
    <div class="transfer-header">
      <span class="transfer-title">
        Transfer Manager
        {#if activeCount > 0}
          <span class="active-count">{activeCount} active</span>
        {/if}
      </span>
      <div class="transfer-header-actions">
        {#if activeCount > 0}
          <button class="header-btn cancel-all-btn" onclick={cancelAll}>Cancel all</button>
        {/if}
        <button class="header-btn" onclick={() => transfer.clearHistory()}>Clear history</button>
        <button class="header-btn close-btn" onclick={onClose}>✕</button>
      </div>
    </div>

    <!-- Transfer list -->
    <div class="transfer-list">
      {#if visibleEntries.length === 0}
        <div class="empty-state">No transfers</div>
      {/if}

      {#each batches as batch (batch.batchId)}
        <!-- Batch divider -->
        <div class="batch-divider" class:active={batch.isActive}>
          {#if batch.isActive}
            ──── Now · {transfer.util.opTypeLabel(batch.opType)} ({
              batch.entries.filter(e => e.status === 'done' || e.status === 'failed').length
            }/{batch.entries.length} done) ────
          {:else}
            ──── {new Date(batch.entries[0]?.startTime || Date.now()).toLocaleDateString()} {new Date(batch.entries[0]?.startTime || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} · {transfer.util.opTypeLabel(batch.opType)} ────
          {/if}
        </div>

        {#each batch.entries as entry (entry.id)}
          {@const pct = progressPercent(entry)}
          {@const isSelected = selectedId === entry.id}
          {@const showDragOver = dragOverIndex !== null && dragItemId !== null &&
            entry.status === 'queued' && entries.findIndex(en => en.id === dragItemId) < entries.findIndex(en => en.id === entry.id)}

          <!-- Drop indicator above -->
          {#if showDragOver}
            <div class="drop-indicator"></div>
          {/if}

          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="transfer-row"
            class:selected={isSelected}
            class:status-done={entry.status === 'done'}
            class:status-failed={entry.status === 'failed'}
            class:status-cancelled={entry.status === 'cancelled'}
            class:status-running={entry.status === 'running'}
            class:status-queued={entry.status === 'queued'}
            class:dragging={dragItemId === entry.id}
            draggable={entry.status === 'queued'}
            onclick={() => selectedId = entry.id}
            onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') selectedId = entry.id; }}
            ondragstart={(e: DragEvent) => handleItemDragStart(e, entry)}
            ondragover={(e: DragEvent) => handleItemDragOver(e, entry)}
            ondrop={(e: DragEvent) => handleItemDrop(e, entry)}
            ondragend={handleItemDragEnd}
          >
            <!-- Unified single-line layout for all states -->
            <span class="drag-handle" class:disabled={entry.status !== 'queued'}>≡</span>
            <span class="row-icon">{getStatusIcon(entry)}</span>
            <span class="row-source">{transfer.util.shortPath(entry.source)}</span>
            <span class="row-arrow">→</span>
            <span class="row-dest">{transfer.util.shortPath(entry.destination)}</span>
            <span class="row-meta">
              {#if entry.status === 'running'}
                <span class="progress-text">{Math.round(pct)}%</span>
                <div class="mini-progress">
                  <div class="mini-progress-fill" style="width: {pct}%"></div>
                </div>
                <span class="size-text">{transfer.util.formatSize(entry.bytesDone)}/{transfer.util.formatSize(entry.totalBytes)}</span>
                {#if entry.speedBps > 0}
                  <span class="speed-text">{transfer.util.formatSpeed(entry.speedBps)}</span>
                {/if}
                {#if entry.etaSecs && entry.etaSecs > 0}
                  <span class="eta-text">ETA {formatElapsed(entry.etaSecs * 1000)}</span>
                {/if}
              {:else if entry.status === 'queued'}
                {#if entry.totalBytes > 0}
                  <span class="size-text">{transfer.util.formatSize(entry.totalBytes)}</span>
                {:else}
                  <span class="size-text scanning-text">scanning...</span>
                {/if}
                <span class="queued-label">queued</span>
              {:else if entry.status === 'done'}
                <span class="done-size">{transfer.util.formatSize(entry.totalBytes)}</span>
                <span class="done-speed">{entry.speedBps > 0 ? transfer.util.formatSpeed(entry.speedBps) : ''}</span>
                <span class="done-time">{entry.elapsedMs ? formatElapsed(entry.elapsedMs) : ''}</span>
              {:else if entry.status === 'failed'}
                <span class="error-text" title={entry.error}>{entry.error || 'Failed'}</span>
              {:else if entry.status === 'cancelled'}
                <span class="meta-text">Cancelled</span>
              {/if}
            </span>
            {#if entry.status === 'running' || entry.status === 'queued'}
              <button class="row-action" onclick={(e: MouseEvent) => { e.stopPropagation(); transfer.cancelTransfer(entry.id); }} title="Cancel">✕</button>
            {:else}
              <button class="row-action" onclick={(e: MouseEvent) => { e.stopPropagation(); transfer.clearTransfer(entry.id); }} title="Dismiss">✕</button>
            {/if}
          </div>
        {/each}
      {/each}
    </div>

    <!-- Floating "back to bottom" button -->
    {#if !autoScroll && activeCount > 0}
      <button class="scroll-bottom-btn" onclick={scrollToBottom}>
        ↓ {activeCount} active
      </button>
    {/if}
  </div>
{/if}

<style>
  .transfer-panel {
    position: relative;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 12px;
    outline: none;
    min-height: 100px;
    flex-shrink: 0;
  }

  .transfer-handle {
    height: 3px;
    cursor: row-resize;
    background-color: var(--border);
    flex-shrink: 0;
  }
  .transfer-handle:hover {
    background-color: var(--accent);
  }

  .transfer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background-color: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .transfer-title {
    font-weight: 600;
    font-size: 12px;
    color: var(--text-primary);
  }

  .active-count {
    color: var(--accent);
    margin-left: 8px;
    font-weight: 400;
    font-size: 11px;
  }

  .transfer-header-actions {
    display: flex;
    gap: 6px;
  }

  .header-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 2px 8px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .header-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }
  .cancel-all-btn {
    border-color: var(--warning);
    color: var(--warning);
  }
  .cancel-all-btn:hover {
    background-color: var(--warning);
    color: var(--bg-primary);
  }

  .transfer-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .empty-state {
    padding: 20px;
    text-align: center;
    color: var(--text-muted);
  }

  /* Batch divider */
  .batch-divider {
    padding: 8px 12px 4px;
    color: var(--text-muted);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .batch-divider.active {
    color: var(--accent);
  }

  /* Transfer row */
  .transfer-row {
    display: flex;
    align-items: center;
    padding: 3px 12px;
    cursor: pointer;
    transition: background-color 0.1s;
    gap: 4px;
    min-height: 24px;
  }

  .transfer-row:hover {
    background-color: var(--bg-hover);
  }
  .transfer-row.selected {
    background-color: var(--bg-active);
  }
  .transfer-row.dragging {
    opacity: 0.4;
  }

  .status-done { color: var(--text-secondary); }
  .status-done .row-icon { color: var(--success); }
  .status-failed .row-icon { color: var(--warning); }
  .status-running .row-icon { color: var(--accent); }
  .status-queued .row-icon { color: var(--text-muted); }

  .row-icon {
    width: 14px;
    text-align: center;
    flex-shrink: 0;
    font-size: 12px;
  }

  .drag-handle {
    cursor: grab;
    color: var(--text-muted);
    font-size: 11px;
    width: 14px;
    text-align: center;
    flex-shrink: 0;
  }
  .drag-handle.disabled {
    cursor: default;
    opacity: 0.3;
  }
  .drag-handle:not(.disabled):hover {
    color: var(--text-primary);
  }

  .row-source, .row-dest {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 300px;
  }
  .row-source { color: var(--text-primary); }
  .row-dest { color: var(--text-secondary); }
  .row-arrow {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .row-meta {
    margin-left: auto;
    color: var(--text-secondary);
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .error-text {
    color: var(--warning);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    display: inline-block;
    vertical-align: bottom;
  }

  .row-action {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    padding: 1px 5px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 10px;
    margin-left: 4px;
    opacity: 0;
  }
  .transfer-row:hover .row-action {
    opacity: 1;
  }
  .row-action:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .meta-text {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .progress-text {
    color: var(--accent);
    font-size: 11px;
    width: 36px;
    text-align: right;
    flex-shrink: 0;
  }

  .mini-progress {
    width: 120px;
    height: 4px;
    background-color: var(--bg-primary);
    border-radius: 2px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .mini-progress-fill {
    height: 100%;
    background-color: var(--accent);
    transition: width 0.15s ease;
    border-radius: 2px;
  }

  .size-text {
    color: var(--text-secondary);
    font-size: 11px;
    width: 120px;
    text-align: right;
    flex-shrink: 0;
  }

  .speed-text {
    color: var(--text-muted);
    font-size: 11px;
    width: 70px;
    text-align: right;
    flex-shrink: 0;
  }

  .eta-text {
    color: var(--text-muted);
    font-size: 11px;
    width: 80px;
    text-align: right;
    flex-shrink: 0;
  }

  .done-size {
    color: var(--text-secondary);
    font-size: 11px;
    width: 70px;
    text-align: right;
    flex-shrink: 0;
    display: inline-block;
  }
  .done-speed {
    color: var(--text-muted);
    font-size: 11px;
    width: 70px;
    text-align: right;
    flex-shrink: 0;
    display: inline-block;
  }
  .done-time {
    color: var(--text-muted);
    font-size: 11px;
    width: 50px;
    text-align: right;
    flex-shrink: 0;
    display: inline-block;
  }

  .queued-label {
    color: var(--text-muted);
    font-size: 11px;
    font-style: italic;
    flex-shrink: 0;
  }

  .scanning-text {
    color: var(--text-muted);
    font-style: italic;
  }

  .drop-indicator {
    height: 2px;
    background-color: var(--accent);
    margin: 0 12px;
  }

  .scroll-bottom-btn {
    position: absolute;
    bottom: 8px;
    right: 12px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--accent);
    color: var(--accent);
    padding: 4px 10px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 11px;
    border-radius: 3px;
    z-index: 10;
  }
  .scroll-bottom-btn:hover {
    background-color: var(--accent);
    color: var(--bg-primary);
  }
</style>
