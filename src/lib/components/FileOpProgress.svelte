<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';

  interface FileOpEntry {
    id: number;
    op_type: 'copy' | 'move' | 'delete';
    bytes_done: number;
    total_bytes: number;
    files_done: number;
    total_files: number;
    current_file: string;
    description: string;
    status: 'scanning' | 'running' | 'done' | 'cancelled' | 'failed';
    error?: string;
    // UI state
    dismissed?: boolean;
    dismissTimeout?: ReturnType<typeof setTimeout>;
  }

  let ops: FileOpEntry[] = $state([]);
  let unlistens: (() => void)[] = [];

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const val = bytes / Math.pow(1024, i);
    return `${val < 10 ? val.toFixed(1) : Math.round(val)} ${units[i]}`;
  }

  function findOp(id: number): FileOpEntry | undefined {
    return ops.find(o => o.id === id);
  }

  function upsertOp(id: number, partial: Partial<FileOpEntry>) {
    const existing = findOp(id);
    if (existing) {
      Object.assign(existing, partial);
    } else {
      ops = [...ops, { id, op_type: 'copy', bytes_done: 0, total_bytes: 0, files_done: 0, total_files: 0, current_file: '', description: '', status: 'scanning', ...partial }];
    }
  }

  function dismissOp(id: number) {
    const op = findOp(id);
    if (!op) return;
    op.dismissed = true;
    if (op.dismissTimeout) clearTimeout(op.dismissTimeout);
    // Remove after animation
    setTimeout(() => {
      ops = ops.filter(o => o.id !== id);
    }, 400);
  }

  function handleComplete(id: number) {
    const op = findOp(id);
    if (!op) return;
    op.status = 'done';
    op.dismissTimeout = setTimeout(() => dismissOp(id), 3000);
  }

  function handleFailed(id: number, error: string) {
    const op = findOp(id);
    if (!op) return;
    op.status = 'failed';
    op.error = error;
  }

  function handleCancelled(id: number) {
    const op = findOp(id);
    if (op) {
      op.status = 'cancelled';
      op.dismissTimeout = setTimeout(() => dismissOp(id), 3000);
    }
  }

  async function handleCancel(id: number) {
    await invoke('cancel_file_op', { id });
  }

  onMount(() => {
    Promise.all([
      listen<{ id: number; total_bytes: number; total_files: number }>('op-scan-complete', (event) => {
        upsertOp(event.payload.id, {
          total_bytes: event.payload.total_bytes,
          total_files: event.payload.total_files,
          status: 'running',
        });
      }),
      listen<FileOpEntry>('op-progress', (event) => {
        upsertOp(event.payload.id, {
          bytes_done: event.payload.bytes_done,
          total_bytes: event.payload.total_bytes,
          files_done: event.payload.files_done,
          total_files: event.payload.total_files,
          current_file: event.payload.current_file,
          description: event.payload.description,
          op_type: event.payload.op_type,
        });
      }),
      listen<{ id: number }>('op-complete', (event) => {
        handleComplete(event.payload.id);
      }),
      listen<{ id: number; error: string }>('op-failed', (event) => {
        handleFailed(event.payload.id, event.payload.error);
      }),
      listen<{ id: number }>('op-cancelled', (event) => {
        handleCancelled(event.payload.id);
      }),
    ]).then(results => {
      unlistens = results;
    });
  });

  onDestroy(() => {
    unlistens.forEach(fn => fn());
  });

  function progressPercent(op: FileOpEntry): number {
    if (op.total_bytes > 0) {
      return Math.min(100, (op.bytes_done / op.total_bytes) * 100);
    }
    if (op.total_files > 0) {
      return Math.min(100, (op.files_done / op.total_files) * 100);
    }
    return 0;
  }

  function progressLabel(op: FileOpEntry): string {
    const pct = progressPercent(op);
    if (op.total_bytes > 0) {
      return `${formatSize(op.bytes_done)} / ${formatSize(op.total_bytes)}`;
    }
    if (op.total_files > 0) {
      return `${op.files_done} / ${op.total_files} files`;
    }
    return `${pct.toFixed(0)}%`;
  }

  function statusIcon(op: FileOpEntry): string {
    switch (op.status) {
      case 'done': return '✓';
      case 'failed': return '✕';
      case 'cancelled': return '○';
      default: return op.op_type === 'copy' ? '⬇' : op.op_type === 'move' ? '⬆' : '✕';
    }
  }

  function statusClass(op: FileOpEntry): string {
    switch (op.status) {
      case 'done': return 'status-done';
      case 'failed': return 'status-failed';
      case 'cancelled': return 'status-cancelled';
      default: return 'status-active';
    }
  }

  let visibleOps = $derived(ops.filter(o => !o.dismissed));
</script>

{#if visibleOps.length > 0}
  <div class="file-op-bar">
    {#each visibleOps as op (op.id)}
      {@const pct = progressPercent(op)}
      <div class="op-row {statusClass(op)}">
        <span class="op-icon">{statusIcon(op)}</span>
        <div class="op-info">
          <div class="op-desc">{op.description}</div>
          {#if op.status === 'running' || op.status === 'scanning'}
            <div class="op-detail">{op.status === 'scanning' ? 'Scanning...' : progressLabel(op)}</div>
            <div class="op-progress-track">
              <div class="op-progress-fill" style="width: {pct}%"></div>
            </div>
          {:else if op.status === 'done'}
            <div class="op-detail done-msg">Done</div>
          {:else if op.status === 'failed'}
            <div class="op-detail error-msg">{op.error || 'Error'}</div>
          {:else if op.status === 'cancelled'}
            <div class="op-detail cancelled-msg">Cancelled</div>
          {/if}
        </div>
        {#if op.status === 'running' || op.status === 'scanning'}
          <button class="op-cancel" onclick={() => handleCancel(op.id)} title="Cancel">✕</button>
        {:else if op.status === 'failed'}
          <button class="op-cancel" onclick={() => dismissOp(op.id)} title="Dismiss">✕</button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .file-op-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 900;
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .op-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    transition: opacity 0.4s ease;
  }

  .op-row:global(.dismissed) {
    opacity: 0;
  }

  .op-row.status-done {
    border-top-color: var(--success);
  }

  .op-row.status-failed {
    border-top-color: var(--warning);
  }

  .op-icon {
    font-size: 13px;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
  }

  .status-active .op-icon { color: var(--accent); }
  .status-done .op-icon { color: var(--success); }
  .status-failed .op-icon { color: var(--warning); }
  .status-cancelled .op-icon { color: var(--text-muted); }

  .op-info {
    flex: 1;
    min-width: 0;
  }

  .op-desc {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .op-detail {
    color: var(--text-secondary);
    font-size: 11px;
    margin-top: 2px;
  }

  .done-msg { color: var(--success); }
  .error-msg { color: var(--warning); }
  .cancelled-msg { color: var(--text-muted); }

  .op-progress-track {
    height: 3px;
    background-color: var(--bg-primary);
    margin-top: 4px;
    border-radius: 1px;
    overflow: hidden;
  }

  .op-progress-fill {
    height: 100%;
    background-color: var(--accent);
    transition: width 0.15s ease;
    border-radius: 1px;
  }

  .op-cancel {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 11px;
    padding: 2px 6px;
    cursor: pointer;
    font-family: var(--font-mono);
    line-height: 1;
  }

  .op-cancel:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
