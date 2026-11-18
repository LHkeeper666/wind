export interface MoveOutcome<T> {
  entry: T;
  result: { status: 'done' | 'failed' | 'partial'; error?: string };
}

export function summarizeMoveOutcomes<T>(queued: MoveOutcome<T>[], failedToQueue: T[], immediateSuccess = 0) {
  const failed = [...failedToQueue, ...queued.filter(item => item.result.status !== 'done').map(item => item.entry)];
  const moved = immediateSuccess + queued.filter(item => item.result.status === 'done').length;
  return { moved, failed };
}

export function nextFtpCopyName(name: string, occupied: ReadonlySet<string>): string {
  const dot = name.lastIndexOf('.');
  const stem = dot > 0 ? name.slice(0, dot) : name;
  const extension = dot > 0 ? name.slice(dot) : '';
  let n = 1;
  while (occupied.has(`${stem} (copy ${n})${extension}`)) n++;
  return `${stem} (copy ${n})${extension}`;
}

export function queuedMoveMessage(count: number): string {
  return `${count} move(s) queued`;
}

export function ftpConflictDecision(name: string, occupied: ReadonlySet<string>, choice: string):
  { action: 'abort' } | { action: 'skip' } | { action: 'target'; targetName: string } {
  if (choice === 'abort') return { action: 'abort' };
  if (choice === 'skip') return { action: 'skip' };
  if (choice === 'keep-copy' || choice === 'keep-copy-all') {
    return { action: 'target', targetName: nextFtpCopyName(name, occupied) };
  }
  return { action: 'target', targetName: name };
}
