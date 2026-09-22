import type { ClipboardEntry } from '$lib/stores/clipboard';
import {
  isProjectTreePathWithin as isTreePathWithin,
  projectTreePathKey as treePathKey,
} from '$lib/utils/project-tree-focus';

export interface SelectionState {
  selectedPaths: Set<string>;
  selectedTreeRoots: Set<string>;
  deselectedTreePaths: Set<string>;
}

export function createSelectionState(): SelectionState {
  return {
    selectedPaths: new Set(),
    selectedTreeRoots: new Set(),
    deselectedTreePaths: new Set(),
  };
}

export function hasSelection(state: SelectionState): boolean {
  return state.selectedPaths.size > 0 || state.selectedTreeRoots.size > 0;
}

export function clearSelection(): SelectionState {
  return createSelectionState();
}

export function togglePathSelection(state: SelectionState, entryPath: string): SelectionState {
  const nextPaths = new Set(state.selectedPaths);
  if (nextPaths.has(entryPath)) nextPaths.delete(entryPath);
  else nextPaths.add(entryPath);
  return { ...state, selectedPaths: nextPaths };
}

export function selectTreeSubtree(state: SelectionState, nodePath: string): SelectionState {
  const nextRoots = new Set(state.selectedTreeRoots);
  nextRoots.add(nodePath);
  const nextPaths = new Set([...state.selectedPaths].filter(path => !isTreePathWithin(path, nodePath)));
  const nextDeselected = new Set([...state.deselectedTreePaths].filter(path => !isTreePathWithin(path, nodePath)));
  return {
    selectedPaths: nextPaths,
    selectedTreeRoots: nextRoots,
    deselectedTreePaths: nextDeselected,
  };
}

export function deselectTreeSubtree(state: SelectionState, nodePath: string): SelectionState {
  const ancestorRoot = [...state.selectedTreeRoots].find(rootPath =>
    rootPath !== nodePath
    && isTreePathWithin(nodePath, rootPath)
    && !isTreePathExcluded(state, nodePath, rootPath)
  );
  const nextRoots = new Set([...state.selectedTreeRoots].filter(path => !isTreePathWithin(path, nodePath)));
  const nextDeselected = new Set([...state.deselectedTreePaths].filter(path => !isTreePathWithin(path, nodePath)));
  if (ancestorRoot) nextDeselected.add(nodePath);
  const nextPaths = new Set([...state.selectedPaths].filter(path => !isTreePathWithin(path, nodePath)));
  return {
    selectedPaths: nextPaths,
    selectedTreeRoots: nextRoots,
    deselectedTreePaths: nextDeselected,
  };
}

export function deselectTreeNode(state: SelectionState, nodePath: string): SelectionState {
  const rootPath = getSelectedTreeRoot(state, nodePath);
  const nextDeselected = new Set(state.deselectedTreePaths);
  if (rootPath) {
    nextDeselected.add(nodePath);
  }
  const nextPaths = new Set(state.selectedPaths);
  nextPaths.delete(nodePath);
  return {
    ...state,
    selectedPaths: nextPaths,
    deselectedTreePaths: nextDeselected,
  };
}

export function selectTreeNode(state: SelectionState, nodePath: string): SelectionState {
  const rootPath = getSelectedTreeRoot(state, nodePath);
  if (rootPath) {
    const nextDeselected = new Set([...state.deselectedTreePaths].filter(path => !isTreePathWithin(path, nodePath)));
    return { ...state, deselectedTreePaths: nextDeselected };
  }
  return togglePathSelection(state, nodePath);
}

export function toggleTreeSelection(state: SelectionState, nodePath: string, isDir: boolean): SelectionState {
  if (isTreeNodeSelected(state, nodePath)) {
    if (isDir) return deselectTreeSubtree(state, nodePath);
    return deselectTreeNode(state, nodePath);
  }
  if (isDir) return selectTreeSubtree(state, nodePath);
  return selectTreeNode(state, nodePath);
}

export function getSelectedTreeRoot(state: SelectionState, nodePath: string): string | null {
  let root: string | null = null;
  for (const candidate of state.selectedTreeRoots) {
    if (isTreePathWithin(nodePath, candidate)
      && (!root || treePathKey(candidate).length > treePathKey(root).length)) {
      root = candidate;
    }
  }
  return root;
}

export function isTreePathExcluded(state: SelectionState, nodePath: string, rootPath: string): boolean {
  return [...state.deselectedTreePaths].some(excludedPath =>
    isTreePathWithin(excludedPath, rootPath) && isTreePathWithin(nodePath, excludedPath)
  );
}

export function isTreePathPartiallyDeselected(state: SelectionState, nodePath: string, rootPath: string): boolean {
  return [...state.deselectedTreePaths].some(excludedPath =>
    isTreePathWithin(excludedPath, rootPath) && isTreePathWithin(excludedPath, nodePath)
  );
}

export function isTreeNodeSelected(state: SelectionState, nodePath: string): boolean {
  if (state.selectedPaths.has(nodePath)) return true;
  const rootPath = getSelectedTreeRoot(state, nodePath);
  if (!rootPath) return state.selectedPaths.has(nodePath);
  return !!rootPath
    && !isTreePathExcluded(state, nodePath, rootPath)
    && !isTreePathPartiallyDeselected(state, nodePath, rootPath);
}

export function getTreeEntry(nodePath: string, nodeName: string | undefined, isDir: boolean, size: number | undefined): ClipboardEntry {
  const name = nodeName ?? nodePath.split(/[/\\]/).filter(Boolean).pop() ?? nodePath;
  return { path: nodePath, name, is_dir: isDir, size };
}

export function getTreeSkipPaths(state: SelectionState, rootPath: string): string[] {
  return [...state.deselectedTreePaths]
    .filter(excludedPath => isTreePathWithin(excludedPath, rootPath))
    .filter(excludedPath => ![...state.deselectedTreePaths].some(otherPath =>
      otherPath !== excludedPath && isTreePathWithin(excludedPath, otherPath) && isTreePathWithin(otherPath, rootPath)
    ))
    .map(excludedPath => {
      const normalizedRoot = rootPath.replace(/\//g, '\\').replace(/\\+$/, '');
      const normalizedExcluded = excludedPath.replace(/\//g, '\\').replace(/\\+$/, '');
      return normalizedExcluded.slice(normalizedRoot.length).replace(/^\\+/, '');
    })
    .filter(Boolean);
}

export function getSelectedProjectEntries(
  state: SelectionState,
  getTreeNode: (path: string) => { entry: { name: string; is_dir: boolean; size: number | null } } | null
): ClipboardEntry[] {
  const entries: ClipboardEntry[] = [];
  for (const rootPath of state.selectedTreeRoots) {
    const selectedByAncestor = [...state.selectedTreeRoots].some(ancestorPath =>
      ancestorPath !== rootPath
      && isTreePathWithin(rootPath, ancestorPath)
      && !isTreePathExcluded(state, rootPath, ancestorPath)
    );
    if (selectedByAncestor) continue;
    const node = getTreeNode(rootPath);
    entries.push({
      ...getTreeEntry(rootPath, node?.entry.name, node?.entry.is_dir ?? true, node?.entry.size ?? undefined),
      skip_rel_paths: getTreeSkipPaths(state, rootPath),
    });
  }
  // Include individual selectedPaths not covered by any tree root
  for (const path of state.selectedPaths) {
    const covered = [...state.selectedTreeRoots].some((rootPath: string) =>
      rootPath !== path && isTreePathWithin(path, rootPath) && !isTreePathExcluded(state, path, rootPath)
    );
    if (covered) continue;
    const node = getTreeNode(path);
    entries.push(getTreeEntry(path, node?.entry.name, node?.entry.is_dir ?? false, node?.entry.size ?? undefined));
  }
  return entries;
}

export function getEntriesToOperate(
  state: SelectionState,
  files: { name: string; path: string; is_dir: boolean; size: number | null }[],
  displayFiles: { name: string; path: string; is_dir: boolean; size: number | null }[],
  selectedIndex: number,
  projectMode: boolean,
  getTreeNode: (path: string) => { entry: { name: string; is_dir: boolean; size: number | null } } | null
): ClipboardEntry[] {
  if (hasSelection(state)) {
    if (projectMode) return getSelectedProjectEntries(state, getTreeNode);
    return files
      .filter(entry => entry.name !== '..' && state.selectedPaths.has(entry.path))
      .map(entry => ({ path: entry.path, name: entry.name, is_dir: entry.is_dir, size: entry.size ?? undefined }));
  }
  if (selectedIndex >= 0 && selectedIndex < displayFiles.length) {
    const f = displayFiles[selectedIndex];
    if (f.name === '..') return [];
    return [{ path: f.path, name: f.name, is_dir: f.is_dir, size: f.size ?? undefined }];
  }
  return [];
}