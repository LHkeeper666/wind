import type { FileEntry } from '$lib/types/file-explorer';

export type SortMode = 'name' | 'size' | 'ext' | 'modified' | 'created';
export type FilterMode = 'prefix' | 'wildcard';

interface SortFilterDeps {
  getFiles: () => FileEntry[];
  getShowHidden: () => boolean;
  getIsArchiveMode: () => boolean;
  getArchiveState: () => { internalPath: string } | null;
  getPath: () => string;
  isVirtualRoot: (path: string) => boolean;
  getParentPath: (path: string) => string;
  onToast: (message: string) => void;
}

export interface SortFilterAPI {
  getSortBy: () => SortMode;
  getSortReverse: () => boolean;
  getDirFirst: () => boolean;
  getFilterPattern: () => string;
  getFilterMode: () => FilterMode;
  getSortPrefixPending: () => boolean;
  getNormalDisplayFiles: () => FileEntry[];
  setSort: (mode: SortMode, reverse?: boolean) => void;
  toggleDirFirst: () => void;
  setFilterPattern: (pattern: string) => void;
  setFilterMode: (mode: FilterMode) => void;
  clearFilter: () => void;
  handleSortPrefix: (event: KeyboardEvent) => boolean;
}

const SORT_LABELS: Record<string, string> = {
  name: 'name', size: 'size', ext: 'extension',
  modified: 'modified time', created: 'created time',
};

export function createDirectorySortFilter(opts: SortFilterDeps): SortFilterAPI {
  let sortBy: SortMode = $state('name');
  let sortReverse: boolean = $state(false);
  let dirFirst: boolean = $state(true);
  let filterPattern: string = $state('');
  let filterMode: FilterMode = $state('prefix');
  let sortPrefixPending: boolean = $state(false);
  let sortPrefixTimeout: ReturnType<typeof setTimeout> | null = null;

  const SORT_CODE_MAP: Record<string, SortMode | 'dirfirst'> = {
    KeyN: 'name', KeyS: 'size', KeyE: 'ext',
    KeyM: 'modified', KeyC: 'created', KeyT: 'dirfirst',
  };

  function setSort(mode: SortMode, reverse: boolean = false) {
    sortBy = mode;
    sortReverse = reverse;
    opts.onToast(`Sorted by ${SORT_LABELS[mode] || mode}${reverse ? ' (reversed)' : ''}`);
  }

  function toggleDirFirst() {
    dirFirst = !dirFirst;
    opts.onToast(dirFirst ? 'Directories first' : 'Mixed order');
  }

  function setFilterPattern(pattern: string) {
    filterPattern = pattern;
  }

  function setFilterMode(mode: FilterMode) {
    filterMode = mode;
  }

  function clearFilter() {
    filterPattern = '';
    opts.onToast('Filter cleared');
  }

  function handleSortPrefix(event: KeyboardEvent): boolean {
    if (sortPrefixPending && SORT_CODE_MAP[event.code]) {
      event.preventDefault();
      sortPrefixPending = false;
      if (sortPrefixTimeout) { clearTimeout(sortPrefixTimeout); sortPrefixTimeout = null; }
      const mode = SORT_CODE_MAP[event.code];
      if (mode === 'dirfirst') toggleDirFirst();
      else setSort(mode, event.shiftKey);
      return true;
    }
    if (sortPrefixPending) {
      sortPrefixPending = false;
      if (sortPrefixTimeout) { clearTimeout(sortPrefixTimeout); sortPrefixTimeout = null; }
    }
    if (event.code === 'KeyS' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      sortPrefixPending = true;
      if (sortPrefixTimeout) clearTimeout(sortPrefixTimeout);
      sortPrefixTimeout = setTimeout(() => { sortPrefixPending = false; }, 1000);
      return true;
    }
    return false;
  }

  function getNormalDisplayFiles(): FileEntry[] {
    const files = opts.getFiles();
    const showHidden = opts.getShowHidden();
    const isArchiveMode = opts.getIsArchiveMode();
    const archiveState = opts.getArchiveState();
    const path = opts.getPath();

    let result = showHidden ? files : files.filter(f => f.name === '..' || !f.is_hidden);

    // Add .. for parent directory navigation
    if (isArchiveMode && archiveState && archiveState.internalPath !== '') {
      const parentPath = archiveState.internalPath.split('/').slice(0, -1).join('/');
      if (!result.some(f => f.name === '..')) {
        result = [{ name: '..', path: parentPath, is_dir: true, size: null }, ...result];
      }
    } else if (!isArchiveMode && !opts.isVirtualRoot(path)) {
      const parentPath = opts.getParentPath(path);
      if (!result.some(f => f.name === '..')) {
        result = [{ name: '..', path: parentPath, is_dir: true, size: null }, ...result];
      }
    }

    // Apply filter
    if (filterPattern) {
      if (filterMode === 'prefix') {
        const lower = filterPattern.toLowerCase();
        result = result.filter(f => f.name === '..' || f.name.toLowerCase().startsWith(lower));
      } else {
        const pattern = filterPattern.replace(/\*/g, '.*').replace(/\?/g, '.');
        const regex = new RegExp(`^${pattern}$`, 'i');
        result = result.filter(f => f.name === '..' || regex.test(f.name));
      }
    }

    // Apply sort (.. always stays first)
    const dotdot = result.filter(f => f.name === '..');
    const rest = result.filter(f => f.name !== '..');

    rest.sort((a, b) => {
      if (dirFirst && a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

      let cmp = 0;
      if (sortBy === 'name') {
        cmp = a.name.localeCompare(b.name, undefined, { numeric: true });
      } else if (sortBy === 'size') {
        cmp = (a.size ?? 0) - (b.size ?? 0);
      } else if (sortBy === 'ext') {
        const extA = a.name.split('.').pop()?.toLowerCase() || '';
        const extB = b.name.split('.').pop()?.toLowerCase() || '';
        cmp = extA.localeCompare(extB);
      } else if (sortBy === 'modified') {
        cmp = (a.modified ?? 0) - (b.modified ?? 0);
      } else if (sortBy === 'created') {
        cmp = (a.created ?? 0) - (b.created ?? 0);
      }

      return sortReverse ? -cmp : cmp;
    });

    return [...dotdot, ...rest];
  }

  return {
    getSortBy: () => sortBy,
    getSortReverse: () => sortReverse,
    getDirFirst: () => dirFirst,
    getFilterPattern: () => filterPattern,
    getFilterMode: () => filterMode,
    getSortPrefixPending: () => sortPrefixPending,
    getNormalDisplayFiles,
    setSort,
    toggleDirFirst,
    setFilterPattern,
    setFilterMode,
    clearFilter,
    handleSortPrefix,
  };
}
