export type DirectoryKey =
  | { backend: 'local'; path: string }
  | { backend: 'ftp'; connection: string; path: string };

export type DirectoryMutationOutcome = 'completed' | 'failed' | 'cancelled';

export interface DirectoryMutation {
  batchId: number;
  outcome: DirectoryMutationOutcome;
  affectedDirectories: DirectoryKey[];
}

export interface LegacyTransferTerminalEvent {
  batch_id: number;
  op_type: 'copy' | 'move' | 'delete' | 'ftp-download' | 'ftp-upload' | 'ftp-delete';
  source: string;
  destination: string;
}

export interface DirectoryBatchSettled {
  batchId: number;
  affectedDirectories: DirectoryKey[];
}

export interface DirectoryRefreshPanel {
  getDirectoryKey(): DirectoryKey;
  getObservedVersion(): number;
  synchronize(version: number): Promise<boolean>;
}

export interface DirectoryCacheInvalidator {
  invalidate(directory: DirectoryKey): void;
}

export interface DirectoryRefreshCoordinatorOptions {
  cache: DirectoryCacheInvalidator;
  getActivePanels: () => Array<DirectoryRefreshPanel | undefined>;
  debounceMs?: number;
  maxWaitMs?: number;
}

function normalizeSegments(path: string, separator: '\\' | '/'): string[] {
  const segments: string[] = [];
  for (const segment of path.split(separator)) {
    if (!segment || segment === '.') continue;
    if (segment === '..') {
      if (segments.length > 0) segments.pop();
      continue;
    }
    segments.push(segment);
  }
  return segments;
}

function normalizeLocalPath(path: string): string {
  const normalized = path.replace(/\//g, '\\');
  if (normalized === '' || normalized === '\\') return '\\';

  const drive = normalized.match(/^([A-Za-z]):(?:\\|$)/);
  if (drive) {
    const segments = normalizeSegments(normalized.slice(drive[0].length), '\\').map(segment => segment.toLowerCase());
    return `${drive[1].toUpperCase()}:\\${segments.join('\\')}`.replace(/\\$/, segments.length === 0 ? '\\' : '');
  }

  const segments = normalizeSegments(normalized, '\\').map(segment => segment.toLowerCase());
  return `\\${segments.join('\\')}`.replace(/\\$/, segments.length === 0 ? '\\' : '');
}

function parseFtpPath(path: string): { connection: string; path: string } | null {
  const match = path.match(/^ftp:\/\/([^/]+)(\/.*)?$/);
  if (!match) return null;
  const segments = normalizeSegments(match[2] || '/', '/');
  return { connection: match[1], path: `/${segments.join('/')}` || '/' };
}

export function normalizeDirectoryKey(location: DirectoryKey | string): DirectoryKey {
  if (typeof location !== 'string') {
    if (location.backend === 'ftp') {
      const parsed = parseFtpPath(`ftp://${location.connection}/${location.path}`);
      return parsed
        ? { backend: 'ftp', ...parsed }
        : { backend: 'ftp', connection: location.connection, path: '/' };
    }
    return { backend: 'local', path: normalizeLocalPath(location.path) };
  }

  const ftp = parseFtpPath(location);
  return ftp ? { backend: 'ftp', ...ftp } : { backend: 'local', path: normalizeLocalPath(location) };
}

export function directoryKeyId(location: DirectoryKey | string): string {
  const key = normalizeDirectoryKey(location);
  return key.backend === 'ftp'
    ? `ftp:${JSON.stringify(key.connection)}:${JSON.stringify(key.path)}`
    : `local:${JSON.stringify(key.path)}`;
}

export function parentDirectoryKey(location: DirectoryKey | string): DirectoryKey {
  const key = normalizeDirectoryKey(location);
  if (key.backend === 'ftp') {
    const segments = key.path.split('/').filter(Boolean);
    segments.pop();
    return { backend: 'ftp', connection: key.connection, path: `/${segments.join('/')}` || '/' };
  }

  if (key.path === '\\' || /^[A-Z]:\\$/.test(key.path)) return key;
  const drive = key.path.match(/^([A-Z]:)\\/);
  const withoutLeaf = key.path.slice(0, key.path.lastIndexOf('\\'));
  return { backend: 'local', path: withoutLeaf || (drive ? `${drive[1]}\\` : '\\') };
}

function uniqueDirectories(directories: DirectoryKey[]): DirectoryKey[] {
  return [...new Map(directories.map(directory => {
    const normalized = normalizeDirectoryKey(directory);
    return [directoryKeyId(normalized), normalized] as const;
  })).values()];
}

export function adaptLegacyTransferEvent(event: LegacyTransferTerminalEvent, outcome: DirectoryMutationOutcome): DirectoryMutation {
  const sourceParent = parentDirectoryKey(event.source);
  const destinationParent = parentDirectoryKey(event.destination);
  const affectedDirectories = event.op_type === 'move'
    ? [sourceParent, destinationParent]
    : event.op_type === 'copy' || event.op_type === 'ftp-download' || event.op_type === 'ftp-upload'
      ? [destinationParent]
      : [sourceParent];

  return {
    batchId: event.batch_id,
    outcome,
    affectedDirectories: uniqueDirectories(affectedDirectories),
  };
}

export class DirectoryRefreshCoordinator {
  private readonly cache: DirectoryCacheInvalidator;
  private readonly getActivePanels: () => Array<DirectoryRefreshPanel | undefined>;
  private readonly debounceMs: number;
  private readonly maxWaitMs: number;
  private readonly versions = new Map<string, number>();
  private readonly dirtyDirectories = new Map<string, DirectoryKey>();
  private readonly batchDirectories = new Map<number, Map<string, DirectoryKey>>();
  private readonly settledBatches = new Set<number>();
  private debounceTimer: ReturnType<typeof setTimeout> | undefined;
  private maxWaitTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(options: DirectoryRefreshCoordinatorOptions) {
    this.cache = options.cache;
    this.getActivePanels = options.getActivePanels;
    this.debounceMs = options.debounceMs ?? 250;
    this.maxWaitMs = options.maxWaitMs ?? 1000;
  }

  getVersion(directory: DirectoryKey | string): number {
    return this.versions.get(directoryKeyId(directory)) ?? 0;
  }

  acceptMutation(mutation: DirectoryMutation): void {
    const directories = uniqueDirectories(mutation.affectedDirectories);
    for (const directory of directories) {
      const id = directoryKeyId(directory);
      this.versions.set(id, this.getVersion(directory) + 1);
      this.cache.invalidate(directory);
      this.dirtyDirectories.set(id, directory);
      const batch = this.batchDirectories.get(mutation.batchId) ?? new Map<string, DirectoryKey>();
      batch.set(id, directory);
      this.batchDirectories.set(mutation.batchId, batch);
    }
    if (directories.length > 0) this.scheduleFlush();
  }

  acceptBatchSettled(signal: DirectoryBatchSettled): void {
    if (this.settledBatches.has(signal.batchId)) return;
    this.settledBatches.add(signal.batchId);
    const directories = new Map<string, DirectoryKey>();
    for (const directory of uniqueDirectories(signal.affectedDirectories)) {
      const id = directoryKeyId(directory);
      directories.set(id, directory);
      if (!this.dirtyDirectories.has(id)) {
        this.versions.set(id, this.getVersion(directory) + 1);
        this.cache.invalidate(directory);
        this.dirtyDirectories.set(id, directory);
      }
    }
    void this.flush(directories);
  }

  settleBatch(batchId: number): void {
    if (this.settledBatches.has(batchId)) return;
    this.settledBatches.add(batchId);
    const directories = this.batchDirectories.get(batchId);
    this.batchDirectories.delete(batchId);
    if (directories) void this.flush(directories);
  }

  async synchronizeActivePanels(): Promise<void> {
    const panels = new Set<DirectoryRefreshPanel>();
    for (const panel of this.getActivePanels()) {
      if (panel && panel.getObservedVersion() < this.getVersion(panel.getDirectoryKey())) {
        panels.add(panel);
      }
    }
    await Promise.all([...panels].map(panel => panel.synchronize(this.getVersion(panel.getDirectoryKey()))));
  }

  dispose(): void {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    if (this.maxWaitTimer) clearTimeout(this.maxWaitTimer);
    this.debounceTimer = undefined;
    this.maxWaitTimer = undefined;
    this.dirtyDirectories.clear();
    this.batchDirectories.clear();
  }

  private scheduleFlush(): void {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => void this.flush(), this.debounceMs);
    if (!this.maxWaitTimer) {
      this.maxWaitTimer = setTimeout(() => void this.flush(), this.maxWaitMs);
    }
  }

  private async flush(requestedDirectories?: Map<string, DirectoryKey>): Promise<void> {
    if (!requestedDirectories) {
      if (this.debounceTimer) clearTimeout(this.debounceTimer);
      if (this.maxWaitTimer) clearTimeout(this.maxWaitTimer);
      this.debounceTimer = undefined;
      this.maxWaitTimer = undefined;
    }

    const dirty = requestedDirectories
      ? new Map([...requestedDirectories].filter(([id]) => this.dirtyDirectories.has(id)))
      : new Map(this.dirtyDirectories);
    for (const id of dirty.keys()) this.dirtyDirectories.delete(id);
    await Promise.all(this.matchingPanels(dirty).map(panel =>
      panel.synchronize(this.getVersion(panel.getDirectoryKey()))
    ));
  }

  private matchingPanels(dirty: Map<string, DirectoryKey>): DirectoryRefreshPanel[] {
    const panels = new Set<DirectoryRefreshPanel>();
    for (const panel of this.getActivePanels()) {
      if (panel && dirty.has(directoryKeyId(panel.getDirectoryKey()))) panels.add(panel);
    }
    return [...panels];
  }
}
