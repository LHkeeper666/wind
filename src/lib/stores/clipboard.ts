import { writable, derived } from 'svelte/store';

export interface ClipboardEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size?: number;
  skip_rel_paths?: string[];
}

interface ClipboardState {
  entries: ClipboardEntry[];
  operation: 'copy' | 'cut' | null;
  archivePath?: string;
}

function createClipboardStore() {
  const { subscribe, set, update } = writable<ClipboardState>({
    entries: [],
    operation: null,
  });

  return {
    subscribe,

    yank(entries: ClipboardEntry[]) {
      update(state => ({
        ...state,
        entries,
        operation: 'copy',
        archivePath: undefined,
      }));
    },

    yankFromArchive(archivePath: string, entries: ClipboardEntry[]) {
      update(state => ({
        ...state,
        entries,
        operation: 'copy',
        archivePath,
      }));
    },

    cut(entries: ClipboardEntry[]) {
      update(state => ({
        ...state,
        entries,
        operation: 'cut',
      }));
    },

    clear() {
      set({ entries: [], operation: null, archivePath: undefined });
    },

    hasItems(): boolean {
      let result = false;
      const unsub = subscribe(state => { result = state.entries.length > 0; });
      unsub();
      return result;
    },

    getOperation(): 'copy' | 'cut' | null {
      let result: 'copy' | 'cut' | null = null;
      const unsub = subscribe(state => { result = state.operation; });
      unsub();
      return result;
    },
  };
}

export const clipboard = createClipboardStore();

export const clipboardSummary = derived(clipboard, ($clipboard) => {
  if ($clipboard.entries.length === 0) return '';
  const count = $clipboard.entries.length;
  const label = count === 1 ? 'file' : 'files';
  const op = $clipboard.operation === 'copy' ? 'yanked' : 'cut';
  const prefix = $clipboard.archivePath ? '[archive] ' : '';
  return `${prefix}${count} ${label} ${op}`;
});

// Extract mark status (from layout store, not clipboard)
import { layout } from './layout';

export const markSummary = derived(layout, ($layout) => {
  if ($layout.markType === 'extract') return 'Archive marked for extraction';
  return '';
});
