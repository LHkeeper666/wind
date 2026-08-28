import { writable, derived, get } from 'svelte/store';
import type { DirectoryMutationOutcome, LegacyTransferTerminalEvent } from '$lib/utils/directory-refresh';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface TransferEntry {
  id: number;
  batchId: number;
  opType: 'copy' | 'move' | 'delete' | 'ftp-download' | 'ftp-upload';
  source: string;
  destination: string;
  totalBytes: number;
  bytesDone: number;
  speedBps: number;
  status: 'queued' | 'running' | 'done' | 'failed' | 'cancelled';
  error?: string;
  startTime: number;
  elapsedMs?: number;
  etaSecs?: number;
}

export interface TransferBatch {
  batchId: number;
  entries: TransferEntry[];
  opType: string;
  isActive: boolean;
  completedAt?: number;
}

function shortPath(path: string): string {
  if (path.startsWith('ftp://')) {
    const parts = path.split('/');
    const name = parts[parts.length - 1] || parts[parts.length - 2] || path;
    const conn = parts[2] || '';
    return `ftp://${conn}/.../${name}`;
  }
  const parts = path.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || path;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val < 10 ? val.toFixed(1) : Math.round(val)} ${units[i]}`;
}

function formatSpeed(bps: number): string {
  if (bps === 0) return '';
  return `${formatSize(bps)}/s`;
}

function opTypeLabel(opType: string): string {
  switch (opType) {
    case 'copy': return 'Copy';
    case 'move': return 'Move';
    case 'delete': return 'Delete';
    case 'ftp-download': return 'FTP Download';
    case 'ftp-upload': return 'FTP Upload';
    default: return opType;
  }
}

const speedSamples = new Map<number, { lastBytes: number; lastTime: number; emaSpeed: number }>();

function computeEta(id: number, bytesDone: number, totalBytes: number, status: string): number {
  if (status !== 'running') {
    speedSamples.delete(id);
    return 0;
  }
  const now = Date.now();
  const prev = speedSamples.get(id);
  let emaSpeed = 0;
  if (prev) {
    const dt = now - prev.lastTime;
    const db = bytesDone - prev.lastBytes;
    if (dt > 0 && db >= 0) {
      const inst = (db * 1000) / dt;
      emaSpeed = prev.emaSpeed > 0 ? prev.emaSpeed * 0.7 + inst * 0.3 : inst;
    } else {
      emaSpeed = prev.emaSpeed;
    }
  }
  speedSamples.set(id, { lastBytes: bytesDone, lastTime: now, emaSpeed });
  if (emaSpeed > 0 && totalBytes > 0 && bytesDone < totalBytes) {
    return Math.round((totalBytes - bytesDone) / emaSpeed);
  }
  return 0;
}

interface EnqueueTask {
  op_type: string;
  source: string;
  destination: string;
  total_bytes: number;
  conn_name?: string;
  skip_rel_paths?: string[];
  permanent?: boolean;
}

function createTransferStore() {
  const { subscribe, set, update } = writable<TransferEntry[]>([]);
  let unlistens: (() => void)[] = [];
  const terminalListeners = new Set<(event: LegacyTransferTerminalEvent, outcome: DirectoryMutationOutcome) => void>();

  function notifyTerminal(payload: Record<string, unknown>, outcome: DirectoryMutationOutcome) {
    const event: LegacyTransferTerminalEvent = {
      batch_id: payload.batch_id as number,
      op_type: payload.op_type as LegacyTransferTerminalEvent['op_type'],
      source: payload.source as string,
      destination: payload.destination as string,
    };
    terminalListeners.forEach(listener => listener(event, outcome));
  }

  function init() {
    Promise.all([
      listen<Record<string, unknown>>('transfer-progress', (event) => {
        const p = event.payload;
        const id = p.id as number;
        const bytesDone = p.bytes_done as number;
        const totalBytes = p.total_bytes as number;
        const status = p.status as TransferEntry['status'];
        const etaSecs = computeEta(id, bytesDone, totalBytes, status);
        update(entries => {
          const idx = entries.findIndex(e => e.id === id);
          const updated = { ...(idx >= 0 ? entries[idx] : {}),
            id,
            batchId: p.batch_id as number,
            opType: p.op_type as TransferEntry['opType'],
            source: p.source as string,
            destination: p.destination as string,
            totalBytes: totalBytes || (idx >= 0 ? entries[idx].totalBytes : 0),
            bytesDone,
            speedBps: p.speed_bps as number,
            status: status || (idx >= 0 ? entries[idx].status : 'queued'),
            startTime: (idx >= 0 ? entries[idx].startTime : 0) || Date.now(),
            etaSecs,
          } as TransferEntry;
          if (idx >= 0) {
            const newEntries = [...entries];
            newEntries[idx] = updated;
            return newEntries;
          }
          return [...entries, updated];
        });
      }),

      listen<Record<string, unknown>>('transfer-complete', (event) => {
        const p = event.payload;
        update(entries => {
          const idx = entries.findIndex(e => e.id === p.id as number);
          if (idx < 0) return entries;
          const newEntries = [...entries];
          newEntries[idx] = {
            ...newEntries[idx],
            status: 'done' as const,
            bytesDone: (p.bytes_done as number) || newEntries[idx].totalBytes,
            totalBytes: (p.bytes_done as number) || newEntries[idx].totalBytes,
            elapsedMs: p.elapsed_ms as number,
            speedBps: (p.avg_speed_bps as number) || 0,
          };
          return newEntries;
        });
        notifyTerminal(p, 'completed');
      }),

      listen<Record<string, unknown>>('transfer-failed', (event) => {
        const p = event.payload;
        update(entries => {
          const idx = entries.findIndex(e => e.id === p.id as number);
          if (idx < 0) return entries;
          const newEntries = [...entries];
          newEntries[idx] = { ...newEntries[idx], status: 'failed' as const, error: p.error as string };
          return newEntries;
        });
        notifyTerminal(p, 'failed');
      }),

      listen<Record<string, unknown>>('transfer-cancelled', (event) => {
        const p = event.payload;
        update(entries => {
          const idx = entries.findIndex(e => e.id === p.id as number);
          if (idx < 0) return entries;
          const newEntries = [...entries];
          newEntries[idx] = { ...newEntries[idx], status: 'cancelled' as const };
          return newEntries;
        });
        notifyTerminal(p, 'cancelled');
      }),

      listen<Record<string, unknown>>('transfer-cancelled-batch', (event) => {
        const ids = event.payload.ids as number[];
        if (!Array.isArray(ids) || ids.length === 0) return;
        const idSet = new Set(ids);
        update(entries => entries.map(e => idSet.has(e.id) ? { ...e, status: 'cancelled' as const } : e));
      }),

      listen<Record<string, unknown>>('transfer-queue-updated', (event) => {
        const ids = event.payload.ids as number[];
        update(entries => {
          // Reorder queued entries to match backend queue order
          const nonQueued = entries.filter(e => e.status !== 'queued');
          const queued = entries.filter(e => e.status === 'queued');
          const reordered = ids
            .map(id => queued.find(e => e.id === id))
            .filter((e): e is TransferEntry => !!e);
          // Append any queued entries not in the reorder list
          for (const e of queued) {
            if (!reordered.find(r => r.id === e.id)) {
              reordered.push(e);
            }
          }
          return [...nonQueued, ...reordered];
        });
      }),
    ]).then(results => {
      unlistens = results;
    });
  }

  async function enqueueTransfers(tasks: EnqueueTask[]): Promise<number[]> {
    // Backend emits transfer-progress with status "queued" for each task,
    // which creates the entries via the event listener. No optimistic creation needed.
    try {
      return await invoke<number[]>('transfer_enqueue', { tasks });
    } catch (e) {
      console.error('[transfer] enqueue failed:', e);
      return [];
    }
  }

  async function cancelTransfer(id: number) {
    try {
      await invoke('transfer_cancel', { id });
    } catch (e) {
      console.error('[transfer] cancel failed:', e);
    }
  }

  async function cancelAllTransfers() {
    try {
      await invoke('transfer_cancel_all');
    } catch (e) {
      console.error('[transfer] cancel all failed:', e);
    }
  }

  async function retryTransfer(entry: TransferEntry) {
    const task: EnqueueTask = {
      op_type: entry.opType,
      source: entry.source,
      destination: entry.destination,
      total_bytes: entry.totalBytes,
    };
    await enqueueTransfers([task]);
  }

  function clearTransfer(id: number) {
    update(entries => entries.filter(e => e.id !== id));
  }

  function addSyntheticEntry(opts: {
    opType: TransferEntry['opType'];
    source: string;
    destination: string;
    totalBytes: number;
    status: 'done' | 'failed';
    error?: string;
  }) {
    const id = Date.now() * 1000 + Math.floor(Math.random() * 1000);
    update(entries => [...entries, {
      id,
      batchId: id,
      opType: opts.opType,
      source: opts.source,
      destination: opts.destination,
      totalBytes: opts.totalBytes,
      bytesDone: opts.status === 'done' ? opts.totalBytes : 0,
      speedBps: 0,
      status: opts.status,
      error: opts.error,
      startTime: Date.now(),
      elapsedMs: 0,
    }]);
  }

  async function reorderTransfers(ids: number[]) {
    // Optimistic local update
    update(entries => {
      const nonQueued = entries.filter(e => e.status !== 'queued');
      const queued = entries.filter(e => e.status === 'queued');
      const reordered = ids
        .map(id => queued.find(e => e.id === id))
        .filter((e): e is TransferEntry => !!e);
      return [...nonQueued, ...reordered];
    });
    try {
      await invoke('transfer_reorder', { ids });
    } catch (e) {
      console.error('[transfer] reorder failed:', e);
    }
  }

  async function loadHistory(): Promise<TransferEntry[]> {
    try {
      const records = await invoke<Array<Record<string, unknown>>>('transfer_get_history');
      return records.map(r => ({
        id: r.id as number,
        batchId: r.batch_id as number,
        opType: r.transfer_type as TransferEntry['opType'],
        source: r.source as string,
        destination: r.destination as string,
        totalBytes: r.total_bytes as number,
        bytesDone: r.bytes_transferred as number,
        speedBps: (r.avg_speed_bps as number) || 0,
        status: r.status as TransferEntry['status'],
        error: (r.error_message as string) || undefined,
        startTime: (r.started_at as number) * 1000,
        elapsedMs: ((r.completed_at as number) - (r.started_at as number)) * 1000,
      }));
    } catch (e) {
      console.error('[transfer] load history failed:', e);
      return [];
    }
  }

  async function clearHistory() {
    try {
      await invoke('transfer_clear_history');
      update(entries => entries.filter(e => e.status === 'queued' || e.status === 'running'));
    } catch (e) {
      console.error('[transfer] clear history failed:', e);
    }
  }

  function onTerminal(listener: (event: LegacyTransferTerminalEvent, outcome: DirectoryMutationOutcome) => void) {
    terminalListeners.add(listener);
    return () => terminalListeners.delete(listener);
  }

  function isBatchSettled(batchId: number): boolean {
    const entries = get({ subscribe }).filter(entry => entry.batchId === batchId);
    return entries.length > 0 && entries.every(entry =>
      entry.status === 'done' || entry.status === 'failed' || entry.status === 'cancelled'
    );
  }

  // Initialize listeners
  init();

  return {
    subscribe,
    enqueueTransfers,
    cancelTransfer,
    cancelAllTransfers,
    retryTransfer,
    clearTransfer,
    addSyntheticEntry,
    reorderTransfers,
    loadHistory,
    clearHistory,
    onTerminal,
    isBatchSettled,
    util: { shortPath, formatSize, formatSpeed, opTypeLabel },
  };
}

export const transfer = createTransferStore();

// Derived: batches grouped by batchId, newest at bottom
export const transferBatches = derived(transfer, ($transfer) => {
  const batchMap = new Map<number, TransferEntry[]>();
  for (const entry of $transfer) {
    const list = batchMap.get(entry.batchId) || [];
    list.push(entry);
    batchMap.set(entry.batchId, list);
  }

  const batches: TransferBatch[] = [];
  for (const [batchId, entries] of batchMap) {
    const hasActive = entries.some(e => e.status === 'queued' || e.status === 'running');
    const opType = entries[0]?.opType || '';
    batches.push({
      batchId,
      entries,
      opType,
      isActive: hasActive,
      completedAt: hasActive ? undefined : Math.max(...entries.map(e => e.startTime)),
    });
  }

  // Sort: active batches at bottom, then by time
  batches.sort((a, b) => {
    if (a.isActive !== b.isActive) return a.isActive ? 1 : -1;
    return (a.completedAt || 0) - (b.completedAt || 0);
  });

  return batches;
});

// Derived: count of active transfers
export const activeTransferCount = derived(transfer, ($transfer) =>
  $transfer.filter(e => e.status === 'queued' || e.status === 'running').length
);
