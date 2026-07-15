export interface DirCacheEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
  is_hidden?: boolean;
  modified?: number | null;
  created?: number | null;
}

/** Shared directory listing cache used by DirectoryPanel and DirectoryPreviewer */
export const directoryCache: Map<string, DirCacheEntry[]> = new Map();
