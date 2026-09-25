export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
  is_hidden?: boolean;
  modified?: number | null;
  created?: number | null;
}

export interface TreeNode {
  entry: FileEntry;
  depth: number;
  parentPath: string | null;
  expanded: boolean;
  loaded: boolean;
  loading: boolean;
  loadPromise: Promise<void> | null;
  error: string;
  children: TreeNode[];
}

export type SelectNotifyMode = 'debounced' | 'immediate' | 'silent';

export interface SelectOptions {
  notify?: SelectNotifyMode;
}

export interface ProjectTreeState {
  enabled: boolean;
  rootPath: string | null;
  expandedPaths: string[];
  selectedPath: string | null;
  scrollOffset: number;
  skipAutoSelect?: boolean;
}
