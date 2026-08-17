import { directoryKeyId, type DirectoryKey } from './directory-refresh';

export interface DirCacheEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
  is_hidden?: boolean;
  modified?: number | null;
  created?: number | null;
}

class DirectoryCache extends Map<string, DirCacheEntry[]> {
  override has(path: string): boolean {
    return super.has(directoryKeyId(path));
  }

  override get(path: string): DirCacheEntry[] | undefined {
    return super.get(directoryKeyId(path));
  }

  override set(path: string, entries: DirCacheEntry[]): this {
    return super.set(directoryKeyId(path), entries);
  }

  override delete(path: string): boolean {
    return super.delete(directoryKeyId(path));
  }

  invalidate(directory: DirectoryKey): void {
    super.delete(directoryKeyId(directory));
  }
}

export const directoryCache = new DirectoryCache();
