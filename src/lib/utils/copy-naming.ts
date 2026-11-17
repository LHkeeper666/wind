export async function firstAvailableCopyPath(
  path: string,
  reserved: Set<string>,
  exists: (path: string) => Promise<boolean>,
  sourceDirectory?: string,
): Promise<string> {
  const slash = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'));
  const dir = path.slice(0, slash + 1);
  const name = path.slice(slash + 1);
  const dot = name.lastIndexOf('.');
  const stem = dot > 0 ? name.slice(0, dot) : name;
  const ext = dot > 0 ? name.slice(dot) : '';
  for (let n = 1; ; n++) {
    const candidateName = `${stem} (${n})${ext}`;
    const candidate = dir + candidateName;
    if (!reserved.has(candidate.toLowerCase()) && !(await exists(candidate)) &&
        !(sourceDirectory && await exists(sourceDirectory + candidateName))) {
      reserved.add(candidate.toLowerCase());
      return candidate;
    }
  }
}

export function buildLocalPasteTasks(
  sources: string[],
  entries: Array<{ path: string; size?: number; skip_rel_paths?: string[] }>,
  destDir: string,
  operation: 'copy' | 'cut' | null,
  destinations: Map<string, string>,
  skipMap: Map<string, string[]>,
  renameMap: Map<string, Record<string, string>>,
  keepCopySources: Set<string>,
) {
  return sources.map(source => {
    const entry = entries.find(item => item.path === source);
    return {
      op_type: operation === 'copy' ? 'copy' : 'move',
      source,
      destination: destinations.get(source) || destDir + '\\' + (source.split(/[/\\]/).pop() || source),
      total_bytes: entry?.size || 0,
      skip_rel_paths: [...(entry?.skip_rel_paths || []), ...(skipMap.get(source) || [])],
      rename_rel_paths: renameMap.get(source) || {},
      keep_copy: keepCopySources.has(source),
    };
  });
}
