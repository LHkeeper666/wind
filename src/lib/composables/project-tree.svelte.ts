import { invoke } from '@tauri-apps/api/core';
import { tick } from 'svelte';
import { directoryCache } from '$lib/utils/directory-cache';
import { directoryKeyId, normalizeDirectoryKey, type DirectoryKey } from '$lib/utils/directory-refresh';
import {
  getCollapseSelectionTarget,
  isProjectTreePathWithin as isTreePathWithin,
  projectTreePathKey as treePathKey,
} from '$lib/utils/project-tree-focus';
import type { FileEntry, TreeNode, SelectOptions, ProjectTreeState } from '$lib/types/file-explorer';

interface ProjectTreeDeps {
  getPath: () => string;
  getSelectedFile: () => FileEntry | null;
  getSelectedIndex: () => number;
  getShowHidden: () => boolean;
  selectByIndex: (index: number, options?: SelectOptions) => void;
  clearSelection: () => void;
  onToast: (message: string) => void;
  onDirectorySynchronized: (directory: DirectoryKey, version: number) => void;
  getDirectoryVersion: (directory: DirectoryKey) => number;
  getScrollOffset: () => number;
  setScrollOffset: (offset: number) => void;
  getParentPath: (dirPath: string) => string;
}

export interface ProjectTreeAPI {
  getProjectMode: () => boolean;
  getProjectRoot: () => TreeNode | null;
  getTreeVisibleNodes: () => TreeNode[];

  setProjectMode: (enabled: boolean, state?: ProjectTreeState) => Promise<boolean>;
  getProjectTreeState: () => ProjectTreeState;
  getOperationDirectory: () => string;

  findTreeNode: (nodePath: string) => TreeNode | null;
  toggleTreeNode: (node: TreeNode) => void;
  collapseTreeNode: (node: TreeNode) => void;
  expandTreeNode: (node: TreeNode, generation?: number, includeHidden?: boolean) => Promise<void>;
  selectTreePathOrAncestor: (nodePath: string, options?: SelectOptions) => void;
  expandRecursively: (node: TreeNode) => Promise<void>;
  collapseDeepestTreeLevel: () => void;
  revealTreePath: (targetPath: string, includeHidden: boolean) => Promise<void>;

  refreshProjectTree: (version?: number) => Promise<boolean>;
  refreshProjectDirectories: (paths: string[]) => Promise<void>;
}

export function createProjectTree(deps: ProjectTreeDeps): ProjectTreeAPI {
  let projectMode: boolean = $state(false);
  let projectRoot: TreeNode | null = $state(null);
  let projectRestoreGeneration = 0;

  // --- Derived ---

  let treeVisibleNodes: TreeNode[] = $derived.by(() => {
    if (!projectMode || !projectRoot) return [];
    const result: TreeNode[] = [];
    const visit = (node: TreeNode) => {
      result.push(node);
      if (node.expanded) node.children.forEach(visit);
    };
    visit(projectRoot);
    return result;
  });

  // --- Internal helpers ---

  function markDirectorySynchronized(dirPath: string, version?: number) {
    const directory = normalizeDirectoryKey(dirPath);
    deps.onDirectorySynchronized(directory, version ?? deps.getDirectoryVersion(directory));
  }

  function findTreeNode(nodePath: string, node: TreeNode | null = projectRoot): TreeNode | null {
    if (!node) return null;
    if (directoryKeyId(node.entry.path) === directoryKeyId(nodePath)) return node;
    for (const child of node.children) {
      const found = findTreeNode(nodePath, child);
      if (found) return found;
    }
    return null;
  }

  async function loadTreeChildren(node: TreeNode, generation: number, forceRefresh: boolean = false, includeHidden: boolean = false): Promise<void> {
    if (!node.entry.is_dir) return;
    if (node.loading) {
      await node.loadPromise;
      if (forceRefresh) await loadTreeChildren(node, generation, true, includeHidden);
      return;
    }
    if (node.loaded && !forceRefresh) return;
    node.loading = true;
    node.error = '';
    node.loadPromise = (async () => {
      try {
        const entries = !forceRefresh && directoryCache.has(node.entry.path)
          ? directoryCache.get(node.entry.path)!
          : await invoke<FileEntry[]>('read_directory', { path: node.entry.path });
        if (generation !== projectRestoreGeneration) return;
        directoryCache.set(node.entry.path, entries);
        const existingChildren = new Map(node.children.map(child => [child.entry.path, child]));
        node.children = entries
          .filter(entry => deps.getShowHidden() || includeHidden || !entry.is_hidden)
          .sort((a, b) => a.is_dir === b.is_dir ? a.name.localeCompare(b.name, undefined, { numeric: true }) : a.is_dir ? -1 : 1)
          .map(entry => {
            const existing = existingChildren.get(entry.path);
            return existing
              ? { ...existing, entry, depth: node.depth + 1, parentPath: node.entry.path }
              : { entry, depth: node.depth + 1, parentPath: node.entry.path, expanded: false, loaded: false, loading: false, loadPromise: null, error: '', children: [] };
          });
        node.loaded = true;
      } catch (error) {
        node.error = String(error);
      } finally {
        node.loading = false;
        node.loadPromise = null;
        projectRoot = projectRoot ? { ...projectRoot } : null;
      }
    })();
    await node.loadPromise;
  }

  async function expandTreeNode(node: TreeNode, generation: number = projectRestoreGeneration, includeHidden: boolean = false): Promise<void> {
    if (!node.entry.is_dir) return;
    await loadTreeChildren(node, generation, includeHidden, includeHidden);
    if (generation === projectRestoreGeneration) {
      node.expanded = true;
      projectRoot = projectRoot ? { ...projectRoot } : null;
    }
  }

  function selectTreePathOrAncestor(nodePath: string, options: SelectOptions = {}): void {
    let candidate: string | null = nodePath;
    while (candidate) {
      const candidatePath = candidate;
      const index = treeVisibleNodes.findIndex(node => directoryKeyId(node.entry.path) === directoryKeyId(candidatePath));
      if (index >= 0) { deps.selectByIndex(index, options); return; }
      const node = findTreeNode(candidate);
      const parent: string | null = node?.parentPath ?? deps.getParentPath(candidate);
      candidate = parent === candidate ? null : parent;
    }
    deps.selectByIndex(0, options);
  }

  async function refreshLoadedTreeNode(node: TreeNode): Promise<void> {
    const loadedChildPaths = node.children.filter(child => child.loaded).map(child => child.entry.path);
    directoryCache.invalidate(normalizeDirectoryKey(node.entry.path));
    await loadTreeChildren(node, projectRestoreGeneration, true);
    for (const childPath of loadedChildPaths) {
      const child = node.children.find(item => item.entry.path === childPath);
      if (child?.loaded) await refreshLoadedTreeNode(child);
    }
  }

  async function expandRecursively(node: TreeNode): Promise<void> {
    const queue: TreeNode[] = [node];
    let visited = 0;
    const limit = 500;
    while (queue.length > 0 && visited < limit) {
      const current = queue.shift()!;
      await expandTreeNode(current);
      visited += 1;
      queue.push(...current.children.filter(child => child.entry.is_dir));
    }
    if (queue.length > 0) deps.onToast(`Project tree expansion stopped after ${limit} directories`);
  }

  function collapseDeepestTreeLevel(): void {
    const deepest = treeVisibleNodes.filter(node => node !== projectRoot && node.expanded)
      .sort((a, b) => b.depth - a.depth)[0];
    if (deepest) collapseTreeNode(deepest);
  }

  function collapseTreeNode(node: TreeNode): void {
    const previousSelectedPath = deps.getSelectedFile()?.path ?? null;
    node.expanded = false;
    projectRoot = projectRoot ? { ...projectRoot } : null;
    ensureSelectionVisibleAfterCollapse(node.entry.path, previousSelectedPath);
  }

  function toggleTreeNode(node: TreeNode): void {
    if (node.expanded) {
      collapseTreeNode(node);
    } else void expandTreeNode(node);
  }

  function ensureSelectionVisibleAfterCollapse(collapsedDirPath: string, previousSelectedPath: string | null = null): void {
    const targetPath = getCollapseSelectionTarget(previousSelectedPath, collapsedDirPath);
    if (!targetPath) return;
    const selectedChanged = !previousSelectedPath || treePathKey(previousSelectedPath) !== treePathKey(targetPath);
    selectTreePathOrAncestor(targetPath, { notify: selectedChanged ? 'immediate' : 'silent' });
    void restoreSelectedTreeNodeFocusAfterRender();
  }

  async function restoreSelectedTreeNodeFocusAfterRender(): Promise<void> {
    await tick();
    if (!projectMode || deps.getSelectedIndex() < 0) return;
    // Focus restoration is handled by DirectoryPanel which owns panelElement
  }

  // --- Exported API ---

  async function setProjectMode(enabled: boolean, state?: ProjectTreeState): Promise<boolean> {
    if (!enabled) {
      projectMode = false;
      projectRoot = null;
      deps.clearSelection();
      return true;
    }
    const rootPath = state?.rootPath ?? deps.getPath();
    if (!rootPath || rootPath === '/' || rootPath === '\\' || rootPath.startsWith('ftp://')) {
      deps.onToast('Project mode requires a local directory');
      return false;
    }
    const generation = ++projectRestoreGeneration;
    projectMode = true;
    deps.clearSelection();
    projectRoot = { entry: { name: rootPath.split(/[/\\]/).filter(Boolean).pop() || rootPath, path: rootPath, is_dir: true, size: null }, depth: 0, parentPath: null, expanded: true, loaded: false, loading: false, loadPromise: null, error: '', children: [] };
    await loadTreeChildren(projectRoot, generation);
    if (generation !== projectRestoreGeneration) return false;
    for (const expandedPath of state?.expandedPaths ?? []) {
      const node = findTreeNode(expandedPath);
      if (node?.entry.is_dir) await expandTreeNode(node, generation);
    }
    if (state?.selectedPath) selectTreePathOrAncestor(state.selectedPath);
    else if (!state?.skipAutoSelect) deps.selectByIndex(0);
    if (state?.scrollOffset) deps.setScrollOffset(state.scrollOffset);
    return true;
  }

  async function refreshProjectTree(version?: number): Promise<boolean> {
    if (!projectRoot) return false;
    const selectedPath = deps.getSelectedFile()?.path ?? null;
    await refreshLoadedTreeNode(projectRoot);
    if (selectedPath) selectTreePathOrAncestor(selectedPath);
    markDirectorySynchronized(projectRoot.entry.path, version);
    return true;
  }

  async function refreshProjectDirectories(paths: string[]): Promise<void> {
    if (!projectMode || !projectRoot) return;
    const selectedPath = deps.getSelectedFile()?.path ?? null;
    const directories = new Set<string>();
    for (const changedPath of paths) {
      directories.add(deps.getParentPath(changedPath));
      if (findTreeNode(changedPath)?.entry.is_dir) directories.add(changedPath);
    }
    for (const directory of directories) {
      directoryCache.invalidate(normalizeDirectoryKey(directory));
      const node = findTreeNode(directory);
      if (node?.loaded) await loadTreeChildren(node, projectRestoreGeneration, true);
    }
    if (selectedPath) selectTreePathOrAncestor(selectedPath);
  }

  function getProjectTreeState(): ProjectTreeState {
    return {
      enabled: projectMode,
      rootPath: projectRoot?.entry.path ?? null,
      expandedPaths: treeVisibleNodes.filter(node => node.expanded && node !== projectRoot).map(node => node.entry.path),
      selectedPath: deps.getSelectedFile()?.path ?? null,
      scrollOffset: deps.getScrollOffset(),
    };
  }

  function getOperationDirectory(): string {
    if (!projectMode) return deps.getPath();
    const node = treeVisibleNodes[deps.getSelectedIndex()];
    if (!node) return projectRoot?.entry.path ?? deps.getPath();
    return node.entry.is_dir ? node.entry.path : (node.parentPath ?? projectRoot?.entry.path ?? deps.getPath());
  }

  async function revealTreePath(targetPath: string, includeHidden: boolean): Promise<void> {
    if (!projectRoot) return;
    const rootPath = projectRoot.entry.path.replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
    const normalizedTarget = targetPath.replace(/\//g, '\\').toLowerCase();
    if (!normalizedTarget.startsWith(`${rootPath}\\`)) return;
    const segments = normalizedTarget.slice(rootPath.length + 1).split('\\').filter(Boolean);
    let node = projectRoot;
    for (const segment of segments.slice(0, -1)) {
      await expandTreeNode(node, projectRestoreGeneration, includeHidden);
      await tick();
      const child = node.children.find(item => item.entry.is_dir && item.entry.name.toLowerCase() === segment);
      if (!child) return;
      node = child;
    }
    await expandTreeNode(node, projectRestoreGeneration, includeHidden);
    await tick();
    const normalizedKey = directoryKeyId(targetPath);
    const index = treeVisibleNodes.findIndex(n => directoryKeyId(n.entry.path) === normalizedKey);
    if (index >= 0) {
      deps.selectByIndex(index);
    } else {
      selectTreePathOrAncestor(targetPath);
    }
  }

  return {
    getProjectMode: () => projectMode,
    getProjectRoot: () => projectRoot,
    getTreeVisibleNodes: () => treeVisibleNodes,

    setProjectMode,
    getProjectTreeState,
    getOperationDirectory,

    findTreeNode,
    toggleTreeNode,
    collapseTreeNode,
    expandTreeNode,
    selectTreePathOrAncestor,
    expandRecursively,
    collapseDeepestTreeLevel,
    revealTreePath,

    refreshProjectTree,
    refreshProjectDirectories,
  };
}
