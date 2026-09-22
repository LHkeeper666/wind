/**
 * Normalize project tree paths for local Windows-style hierarchy comparisons.
 */
export function projectTreePathKey(path: string): string {
  const normalized = path.replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
  return /^[a-z]:$/.test(normalized) ? `${normalized}\\` : normalized;
}

export function isProjectTreePathWithin(nodePath: string, ancestorPath: string): boolean {
  const nodeKey = projectTreePathKey(nodePath);
  const ancestorKey = projectTreePathKey(ancestorPath);
  const ancestorPrefix = ancestorKey.endsWith('\\') ? ancestorKey : `${ancestorKey}\\`;
  return nodeKey === ancestorKey || nodeKey.startsWith(ancestorPrefix);
}

/**
 * Return the visible path that should become active after collapsing a directory.
 */
export function getCollapseSelectionTarget(
  selectedPath: string | null | undefined,
  collapsedDirPath: string,
): string | null {
  if (!selectedPath) return null;
  return isProjectTreePathWithin(selectedPath, collapsedDirPath) ? collapsedDirPath : selectedPath;
}