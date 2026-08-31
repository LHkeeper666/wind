/**
 * Normalize project tree paths for local Windows-style hierarchy comparisons.
 *
 * @param {string} path
 * @returns {string}
 */
export function projectTreePathKey(path) {
  const normalized = path.replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
  return /^[a-z]:$/.test(normalized) ? `${normalized}\\` : normalized;
}

/**
 * @param {string} nodePath
 * @param {string} ancestorPath
 * @returns {boolean}
 */
export function isProjectTreePathWithin(nodePath, ancestorPath) {
  const nodeKey = projectTreePathKey(nodePath);
  const ancestorKey = projectTreePathKey(ancestorPath);
  const ancestorPrefix = ancestorKey.endsWith('\\') ? ancestorKey : `${ancestorKey}\\`;
  return nodeKey === ancestorKey || nodeKey.startsWith(ancestorPrefix);
}

/**
 * Return the visible path that should become active after collapsing a directory.
 *
 * @param {string | null | undefined} selectedPath
 * @param {string} collapsedDirPath
 * @returns {string | null}
 */
export function getCollapseSelectionTarget(selectedPath, collapsedDirPath) {
  if (!selectedPath) return null;
  return isProjectTreePathWithin(selectedPath, collapsedDirPath) ? collapsedDirPath : selectedPath;
}
