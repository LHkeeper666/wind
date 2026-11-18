import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';
import { clipboard } from '$lib/stores/clipboard';
import { transfer } from '$lib/stores/transfer';
import { firstAvailableCopyPath, buildLocalPasteTasks } from '$lib/utils/copy-naming';
import { ftpConflictDecision, queuedMoveMessage, summarizeMoveOutcomes } from '$lib/utils/ftp-move-policy';
import { invokeArchiveWithOptionalPassword, getCachedArchivePassword } from '$lib/utils/archive-password';
import { promptConflict, promptConflictStream, setScanningConflicts } from '$lib/composables/dialog-state.svelte';
import {
  setCompressMarkPaths, setCompressDialogValue, setCompressDialogVisible,
} from '$lib/composables/dialog-state.svelte';

// --- Dependency context ---
interface ClipboardDeps {
  showToast: (msg: string) => void;
  refreshPanels: (paths: string[]) => Promise<void>;
  onOpenTransfer: () => void;
}

let _deps: ClipboardDeps | null = null;

export function initClipboardDeps(deps: ClipboardDeps) {
  _deps = deps;
}

interface MoveTerminal {
  move_id: number;
  source: string;
  destination: string;
  status: 'done' | 'failed' | 'partial';
  error?: string;
}

async function registerAndWaitForMove(ids: number[], source: string, destination: string, isDir: boolean): Promise<MoveTerminal> {
  transfer.markMoveTasks(ids);
  let moveId: number | null = null;
  let resolveResult!: (result: MoveTerminal) => void;
  const result = new Promise<MoveTerminal>(resolve => { resolveResult = resolve; });
  const pending: MoveTerminal[] = [];
  const unlisten = await listen<MoveTerminal>('transfer-move-terminal', event => {
    if (moveId === null) { pending.push(event.payload); return; }
    if (event.payload.move_id === moveId) resolveResult(event.payload);
  });
  try {
    moveId = await invoke<number>('transfer_register_move', { ids, source, destination, isDir });
    const early = pending.find(event => event.move_id === moveId);
    if (early) resolveResult(early);
    const outcome = await result;
    transfer.setMoveOutcome(ids, outcome.status, outcome.error);
    return outcome;
  } finally { unlisten(); }
}

// --- FTP utility functions ---

export function isFtpPath(p: string): boolean { return p.startsWith('ftp://'); }

export function getFtpConnName(p: string): string {
  const rest = p.slice(6);
  const slash = rest.indexOf('/');
  return slash >= 0 ? rest.substring(0, slash) : rest;
}

export function getFtpRemotePath(p: string): string {
  const rest = p.slice(6);
  const slash = rest.indexOf('/');
  return slash >= 0 ? rest.substring(slash) : '/';
}

export function getFtpDestPath(dirPath: string, name: string): string {
  const base = dirPath.replace(/\/$/, '');
  return base + '/' + name;
}

function parentDirectory(path: string): string {
  const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return slash < 0 ? path : path.slice(0, slash);
}

// --- Conflict scanning ---

export async function scanConflicts(
  startScan: () => Promise<unknown>,
  local = false,
  reserved = new Set<string>(),
): Promise<{ dirSkipMap: Map<string, string[]>; fileSkipSet: Set<string>; dirRenameMap: Map<string, Record<string, string>>; keepCopyAll: boolean } | null> {
  const dirSkipMap = new Map<string, string[]>();
  const dirRenameMap = new Map<string, Record<string, string>>();
  const fileSkipSet = new Set<string>();
  let applyToAll: 'overwrite' | 'skip' | 'keep-copy' | null = null;
  let queue: Array<{ kind: string; dir_source: string; rel_path: string; destination: string; source: string }> = [];
  let processing = false;
  let finished = false;
  let aborted = false;

  const addSkip = (kind: string, dir_source: string, rel: string) => {
    if (kind === 'dir') {
      if (!dirSkipMap.has(dir_source)) dirSkipMap.set(dir_source, []);
      dirSkipMap.get(dir_source)!.push(rel);
    } else {
      fileSkipSet.add(rel);
    }
  };

  const addKeepCopy = async (c: { kind: string; dir_source: string; rel_path: string; destination: string; source: string }) => {
    if (!local || c.kind !== 'dir') return;
    const sourceDir = c.source.slice(0, Math.max(c.source.lastIndexOf('\\'), c.source.lastIndexOf('/')) + 1);
    const newPath = await firstAvailableCopyPath(c.destination, reserved, path => invoke<boolean>('file_exists', { path }), sourceDir);
    if (!dirRenameMap.has(c.dir_source)) dirRenameMap.set(c.dir_source, {});
    dirRenameMap.get(c.dir_source)![c.rel_path] = newPath.slice(Math.max(newPath.lastIndexOf('\\'), newPath.lastIndexOf('/')) + 1);
  };

  return new Promise((resolve) => {
    let unlistenFound: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;

    const maybeFinish = () => {
      if (finished && !processing && queue.length === 0) {
        unlistenFound?.();
        unlistenDone?.();
        resolve(aborted ? null : { dirSkipMap, fileSkipSet, dirRenameMap, keepCopyAll: applyToAll === 'keep-copy' });
      }
    };

    const processQueue = async () => {
      if (processing) return;
      processing = true;
      while (queue.length > 0 && !aborted) {
        const c = queue.shift()!;
        if (applyToAll === 'skip') {
          addSkip(c.kind, c.dir_source, c.rel_path);
        } else if (applyToAll === 'overwrite') {
          // 覆盖所有：不 skip
        } else if (applyToAll === 'keep-copy') {
          try { await addKeepCopy(c); } catch { aborted = true; }
        } else {
          const choice = await promptConflictStream(c.rel_path, local);
          if (choice === 'skip') addSkip(c.kind, c.dir_source, c.rel_path);
          else if (choice === 'keep-copy' || choice === 'keep-copy-all') {
            if (choice === 'keep-copy-all') applyToAll = 'keep-copy';
            try { await addKeepCopy(c); } catch { aborted = true; }
          }
          else if (choice === 'overwrite-all') applyToAll = 'overwrite';
          else if (choice === 'skip-all') { applyToAll = 'skip'; addSkip(c.kind, c.dir_source, c.rel_path); }
          else if (choice === 'abort') aborted = true;
        }
      }
      queue.length = 0;
      processing = false;
      maybeFinish();
    };

    Promise.all([
      listen<Record<string, unknown>>('transfer-conflict-found', (event) => {
        if (aborted) return;
        queue.push(event.payload as any);
        processQueue();
      }),
      listen('transfer-conflict-scan-done', () => {
        finished = true;
        maybeFinish();
      }),
    ]).then(([found, done]) => {
      unlistenFound = found;
      unlistenDone = done;
      startScan().catch(() => {
        finished = true;
        maybeFinish();
      });
    });
  });
}

// --- Paste operation ---

export async function handlePaste(
  currentPath: string,
  getOperationDirectory: () => string,
  force: boolean = false,
): Promise<void> {
  const deps = _deps!;
  const layoutVal = get(layout);

  // Check for extract mark first (from layout store)
  if (layoutVal.markType === 'extract' && layoutVal.markPaths.length > 0) {
    const archivePath = layoutVal.markPaths[0];
    const destDir = getOperationDirectory();
    try {
      // Validate by listing archive root (handles password dialog/retry)
      const result = await invokeArchiveWithOptionalPassword<unknown>(
        'read_archive_directory',
        { archivePath, internalPath: '' },
        'password'
      );
      if (result === null) return;
      const password = getCachedArchivePassword(archivePath);
      const id = await transfer.enqueueExtract({
        archivePath,
        destDir,
        password: password || undefined,
      });
      if (id === null) return;
      deps.showToast('Archive extraction queued');
      layout.clearMark();
      deps.onOpenTransfer();
    } catch (e) {
      deps.showToast(`Extract failed: ${e}`);
    }
    return;
  }

  // Check for compress mark (C key marks files, p shows compress dialog)
  if (layoutVal.markType === 'compress' && layoutVal.markPaths.length > 0) {
    setCompressMarkPaths(layoutVal.markPaths);
    const paths = layoutVal.markPaths;
    const defaultName = paths.length === 1
      ? (paths[0].split(/[/\\]/).pop() || 'file').replace(/\.\w+$/, '') + '.zip'
      : 'archive.zip';
    setCompressDialogValue(defaultName);
    setCompressDialogVisible(true);
    return;
  }

  let state: any;
  const unsub = clipboard.subscribe(v => state = v)();
  if (!state.entries || state.entries.length === 0) {
    deps.showToast('Clipboard empty');
    return;
  }

  // Check for archive clipboard entries (yanked from within an archive)
  if (state.archivePath) {
    const destDir = getOperationDirectory();
    const internalPaths = state.entries.map((e: any) => e.path);
    try {
      // Validate by listing archive root (handles password dialog/retry)
      const result = await invokeArchiveWithOptionalPassword<unknown>(
        'read_archive_directory',
        { archivePath: state.archivePath, internalPath: '' },
        'password'
      );
      if (result === null) return;
      const password = getCachedArchivePassword(state.archivePath);
      const id = await transfer.enqueueExtract({
        archivePath: state.archivePath,
        destDir,
        password: password || undefined,
        internalPaths,
      });
      if (id === null) return;
      deps.showToast(`${state.entries.length} ${state.entries.length === 1 ? 'file' : 'files'} extraction queued`);
      clipboard.clear();
      layout.clearMark();
      deps.onOpenTransfer();
    } catch (e) {
      deps.showToast(`Extract failed: ${e}`);
    }
    return;
  }

  // Check if we're in archive mode: paste files INTO the archive (ZIP only)
  const archiveState = layoutVal.archiveState;
  if (archiveState && !state.archivePath) {
    const sourcePaths = state.entries.map((e: any) => e.path);
    try {
      await invoke('archive_add_files', {
        archivePath: archiveState.archivePath,
        sourcePaths,
        internalPath: archiveState.internalPath,
      });
      deps.showToast(`${sourcePaths.length} ${sourcePaths.length === 1 ? 'file' : 'files'} added to archive`);
      clipboard.clear();
      layout.clearMark();
      // Need to refresh current directory panel - caller should handle this
    } catch (e) {
      deps.showToast(`Add to archive failed: ${e}`);
    }
    return;
  }

  const entries = state.entries;
  const operation = state.operation;
  const destinationPath = getOperationDirectory();
  const destIsFtp = isFtpPath(destinationPath);
  const srcIsFtp = entries.some((e: any) => isFtpPath(e.path));
  const isCrossBackend = destIsFtp || srcIsFtp;

  // Cross-backend paste: skip local conflict detection
  if (isCrossBackend) {
    const pendingMoves: Array<Promise<{ entry: (typeof entries)[number]; result: MoveTerminal }>> = [];
    const failedToQueue: Array<(typeof entries)[number]> = [];
    const queueMove = (entry: (typeof entries)[number], ids: number[], destination: string) => {
      if (ids.length === 0 && !entry.is_dir) { failedToQueue.push(entry); return; }
      pendingMoves.push(registerAndWaitForMove(ids, entry.path, destination, entry.is_dir)
        .then(result => ({ entry, result }))
        .catch(error => ({ entry, result: { move_id: -1, source: entry.path, destination, status: 'failed' as const, error: String(error) } })));
    };
    if (srcIsFtp && !destIsFtp) {
      // FTP → Local: download via TransferManager
      const destDir = destinationPath.replace(/[\\\/]+$/, '');

      const dirs = entries.filter((e: any) => e.is_dir);
      const files = entries.filter((e: any) => !e.is_dir);

      // Stream conflict detection on local targets
      const skipFiles = new Set<string>();
      const skipDirMap = new Map<string, string[]>();
      if (!force) {
        setScanningConflicts(true);
        const result = await scanConflicts(() => invoke('scan_ftp_download_conflicts', {
          connName: getFtpConnName(entries[0].path),
          files: files.map((e: any) => e.path),
          dirs: dirs.map((e: any) => e.path),
          localDir: destDir,
        }));
        setScanningConflicts(false);
        if (result === null) {
          deps.showToast('Download aborted');
          return;
        }
        for (const [dirSource, rels] of result.dirSkipMap) {
          skipDirMap.set(dirSource, rels);
        }
        for (const f of result.fileSkipSet) {
          skipFiles.add(f);
        }
      }

      let totalQueued = 0;

      // Download folders as batch downloads (skip internal conflicting files)
      for (const dir of dirs) {
        const skipRel = skipDirMap.get(dir.path) || [];
        try {
          const response = await invoke<string>('ftp_download_folder', {
            connName: getFtpConnName(dir.path),
            remotePath: getFtpRemotePath(dir.path),
            localPath: destDir + '\\' + dir.name,
            moveMode: false,
            skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
          });
          if (operation === 'cut') {
            if (skipRel.length > 0 || (dir.skip_rel_paths || []).length > 0) failedToQueue.push(dir);
            else queueMove(dir, JSON.parse(response).task_ids, destDir + '\\' + dir.name);
          }
          totalQueued++;
        } catch (error) {
          deps.showToast(`Download failed to queue: ${error}`);
          if (operation === 'cut') failedToQueue.push(dir);
        }
      }

      // Download files as individual transfers (skip conflicting files)
      const downloadFiles = files.filter((f: any) => !skipFiles.has(f.name));
      if (downloadFiles.length > 0) {
        const tasks = downloadFiles.map((entry: any) => ({
          op_type: 'ftp-download' as const,
          source: entry.path,
          destination: destDir + '\\' + entry.name,
          total_bytes: entry.size || 0,
          conn_name: getFtpConnName(entry.path),
        }));
        const ids = await transfer.enqueueTransfers(tasks);
        totalQueued += ids.length;
        if (operation === 'cut') downloadFiles.forEach((entry: any, index: number) => queueMove(entry, ids[index] === undefined ? [] : [ids[index]], destDir + '\\' + entry.name));
      }

      if (totalQueued > 0) {
        deps.showToast(`Queued ${totalQueued} download(s)`);
        deps.onOpenTransfer();
      }

      if (operation === 'cut') files.filter((f: any) => skipFiles.has(f.name)).forEach((entry: any) => failedToQueue.push(entry));
    } else if (!srcIsFtp && destIsFtp) {
      // Local → FTP: upload via TransferManager
      const destConn = getFtpConnName(destinationPath);
      const destBase = getFtpRemotePath(destinationPath).replace(/\/+$/, '');

      const dirs = entries.filter((e: any) => e.is_dir);
      const files = entries.filter((e: any) => !e.is_dir);

      // Stream conflict detection on remote targets
      const skipFiles = new Set<string>();
      const skipDirMap = new Map<string, string[]>();
      if (!force) {
        setScanningConflicts(true);
        const result = await scanConflicts(() => invoke('scan_ftp_upload_conflicts', {
          connName: destConn,
          sources: entries.map((e: any) => e.path),
          remoteDir: destBase,
        }));
        setScanningConflicts(false);
        if (result === null) {
          deps.showToast('Upload aborted');
          return;
        }
        for (const [dirSource, rels] of result.dirSkipMap) {
          const name = dirSource.split(/[/\\]/).pop() || dirSource;
          skipDirMap.set(name, rels);
        }
        for (const f of result.fileSkipSet) {
          skipFiles.add(f);
        }
      }

      let totalQueued = 0;

      // Upload folders as batch uploads (skip internal conflicting files)
      for (const dir of dirs) {
        const skipRel = skipDirMap.get(dir.name) || [];
        try {
          const response = await invoke<string>('ftp_upload_folder', {
            connName: destConn,
            localPath: dir.path,
            remotePath: `${destBase}/${dir.name}`,
            moveMode: false,
            skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
          });
          if (operation === 'cut') {
            if (skipRel.length > 0 || (dir.skip_rel_paths || []).length > 0) failedToQueue.push(dir);
            else queueMove(dir, JSON.parse(response).task_ids, `ftp://${destConn}${destBase}/${dir.name}`);
          }
          totalQueued++;
        } catch (error) {
          deps.showToast(`Upload failed to queue: ${error}`);
          if (operation === 'cut') failedToQueue.push(dir);
        }
      }

      // Upload files as individual transfers (skip conflicting files)
      const uploadFiles = files.filter((f: any) => !skipFiles.has(f.name));
      if (uploadFiles.length > 0) {
        const tasks = uploadFiles.map((entry: any) => ({
          op_type: 'ftp-upload' as const,
          source: entry.path,
          destination: `ftp://${destConn}${destBase}/${entry.name}`,
          total_bytes: 0,
          conn_name: destConn,
        }));
        const ids = await transfer.enqueueTransfers(tasks);
        totalQueued += ids.length;
        if (operation === 'cut') uploadFiles.forEach((entry: any, index: number) => queueMove(entry, ids[index] === undefined ? [] : [ids[index]], `ftp://${destConn}${destBase}/${entry.name}`));
      }

      if (totalQueued > 0) {
        deps.showToast(`Queued ${totalQueued} upload(s)`);
        deps.onOpenTransfer();
      }

      if (operation === 'cut') files.filter((f: any) => skipFiles.has(f.name)).forEach((entry: any) => failedToQueue.push(entry));
    } else if (srcIsFtp && destIsFtp) {
      // FTP → FTP
      const srcConn = getFtpConnName(entries[0].path);
      const destConn = getFtpConnName(destinationPath);
      if (entries.some((entry: any) => getFtpConnName(entry.path) !== srcConn)) {
        deps.showToast('Clipboard contains multiple FTP connections');
        return;
      }
      if (srcConn === destConn) {
        const destBase = getFtpDestPath(destinationPath, '').replace(/\/+$/, '');
        let listing: Array<{ name: string }>;
        try { listing = await invoke<Array<{ name: string }>>('ftp_read_directory', { path: destinationPath }); }
        catch (error) { deps.showToast(`Could not read destination: ${error}`); return; }
        const occupied = new Set(listing.map(item => item.name));
        if (operation === 'cut') {
          let moved = 0;
          for (const [index, entry] of entries.entries()) {
            let targetName = entry.name;
            if (occupied.has(targetName) && !force) {
              const choice = await promptConflict(entry.name, entries.length > 1);
              const decision = ftpConflictDecision(entry.name, occupied, choice);
              if (decision.action === 'abort') { failedToQueue.push(...entries.slice(index)); break; }
              if (decision.action === 'skip') { failedToQueue.push(entry); continue; }
              targetName = decision.targetName;
            }
            occupied.add(targetName);
            const newFullPath = destBase + '/' + targetName;
            if (entry.path === newFullPath) { deps.showToast('Cannot move an entry onto itself'); failedToQueue.push(entry); continue; }
            try {
              if (force && listing.some(item => item.name === targetName)) throw new Error('FTP_RENAME_REJECTED: destination exists');
              await invoke('ftp_rename', { oldPath: entry.path, newPath: newFullPath, isDir: entry.is_dir });
              moved++;
            } catch (e: any) {
              if (!String(e).includes('FTP_RENAME_REJECTED')) { deps.showToast(`Move failed: ${e}`); failedToQueue.push(entry); continue; }
              try {
                const ids = entry.is_dir
                  ? await invoke<number[]>('ftp_copy_folder', { connName: srcConn, sourceDir: getFtpRemotePath(entry.path), destinationDir: getFtpRemotePath(newFullPath) })
                  : await transfer.enqueueTransfers([{ op_type: 'ftp-copy', source: entry.path, destination: newFullPath, total_bytes: entry.size || 0, conn_name: srcConn }]);
                queueMove(entry, ids, newFullPath);
              } catch (copyError) { deps.showToast(`Move failed: ${copyError}`); failedToQueue.push(entry); }
            }
          }
          if (pendingMoves.length > 0) { deps.showToast(queuedMoveMessage(pendingMoves.length)); deps.onOpenTransfer(); }
          const results = await Promise.all(pendingMoves);
          const { failed, moved: completed } = summarizeMoveOutcomes(results, failedToQueue, moved);
          const current = get(clipboard);
          if (current.operation === 'cut' && current.entries.map(e => e.path).join('\n') === entries.map((e: any) => e.path).join('\n')) {
            if (failed.length > 0) { clipboard.cut(failed); layout.setMark('cut', failed.map((e: any) => e.path)); }
            else { clipboard.clear(); layout.clearMark(); }
          }
          for (const { result } of results) if (result.status !== 'done') deps.showToast(`Move ${result.status}: ${result.error || 'unknown error'}`);
          deps.showToast(`${completed} moved, ${failed.length} retained for retry`);
          await deps.refreshPanels([destinationPath, ...entries.map((e: any) => parentDirectory(e.path))]);
          return;
        } else {
          let queued = 0;
          for (const entry of entries) {
            const newRemote = destBase + '/' + entry.name;
            try {
              if (entry.is_dir) {
                const ids = await invoke<number[]>('ftp_copy_folder', { connName: srcConn, sourceDir: getFtpRemotePath(entry.path), destinationDir: getFtpRemotePath(newRemote) });
                queued += ids.length;
              } else {
                const ids = await transfer.enqueueTransfers([{ op_type: 'ftp-copy', source: entry.path, destination: newRemote, total_bytes: entry.size || 0, conn_name: srcConn }]);
                queued += ids.length;
              }
            } catch (e) { deps.showToast(`Copy failed: ${e}`); }
          }
          deps.showToast(`Queued ${queued} FTP copy task(s)`);
          if (queued > 0) deps.onOpenTransfer();
        }
      } else {
        deps.showToast('Cross-server transfer not yet supported');
      }
    }

    if (operation === 'cut' && (srcIsFtp !== destIsFtp)) {
      if (pendingMoves.length > 0) deps.showToast(queuedMoveMessage(pendingMoves.length));
      const results = await Promise.all(pendingMoves);
      const { failed, moved } = summarizeMoveOutcomes(results, failedToQueue);
      const current = get(clipboard);
      if (current.operation === 'cut' && current.entries.map(e => e.path).join('\n') === entries.map((e: any) => e.path).join('\n')) {
        if (failed.length > 0) { clipboard.cut(failed); layout.setMark('cut', failed.map((e: any) => e.path)); }
        else { clipboard.clear(); layout.clearMark(); }
      }
      for (const { result } of results) if (result.status !== 'done') deps.showToast(`Move ${result.status}: ${result.error || 'unknown error'}`);
      deps.showToast(`${moved} moved, ${failed.length} retained for retry`);
      await deps.refreshPanels([destinationPath, ...entries.map((e: any) => parentDirectory(e.path))]);
    } else if (operation === 'cut') clipboard.clear();
    return;
  }

  // Local-to-local paste
  const destDir = destinationPath.replace(/[\\\/]+$/, '');
  let processed = 0;
  let firstPastedPath: string | null = null;
  const resolvedSources: string[] = [];
  const skipMap = new Map<string, string[]>();
  const renameMap = new Map<string, Record<string, string>>();
  const destinations = new Map<string, string>();
  const keepCopySources = new Set<string>();
  let keepCopyAll = false;
  const reserved = new Set<string>();

  const dirEntries = entries.filter((e: any) => e.is_dir);
  const fileEntries = entries.filter((e: any) => !e.is_dir);
  for (const file of fileEntries) reserved.add((destDir + '\\' + file.name).toLowerCase());

  // Phase 1a: directories — stream conflict detection
  if (dirEntries.length > 0) {
    if (force) {
      for (const dir of dirEntries) {
        resolvedSources.push(dir.path);
        if (!firstPastedPath) firstPastedPath = destDir + '\\' + dir.name;
      }
    } else {
      const dirTasks = dirEntries.map((entry: any) => ({
        op_type: operation === 'copy' ? 'copy' : 'move',
        source: entry.path,
        destination: destDir + '\\' + entry.name,
        total_bytes: 0,
        skip_rel_paths: entry.skip_rel_paths || [],
      }));
      setScanningConflicts(true);
      const result = await scanConflicts(() => invoke('scan_transfer_conflicts', { tasks: dirTasks }), true, reserved);
      setScanningConflicts(false);
      if (result === null) {
        deps.showToast('Paste aborted');
        return;
      }
      keepCopyAll = result.keepCopyAll;
      for (const dir of dirEntries) {
        resolvedSources.push(dir.path);
        if (!firstPastedPath) firstPastedPath = destDir + '\\' + dir.name;
        const skip = result.dirSkipMap.get(dir.path);
        if (skip) skipMap.set(dir.path, skip);
        const renames = result.dirRenameMap.get(dir.path);
        if (renames) renameMap.set(dir.path, renames);
      }
    }
  }

  // Phase 1b: files — existing sync conflict check
  for (const entry of fileEntries) {
    let destPath = destDir + '\\' + entry.name;

    let exists = false;
    try {
      exists = await invoke<boolean>('file_exists', { path: destPath });
    } catch {
      exists = false;
    }

    if (exists) {
      if (!force) {
        const choice = keepCopyAll ? 'keep-copy' : await promptConflict(entry.name, entries.length > 1);
        if (choice === 'abort') {
          deps.showToast(`Paste aborted (${processed}/${entries.length} done)`);
          return;
        }
        if (choice === 'skip') {
          continue;
        }
        if (choice === 'keep-copy' || choice === 'keep-copy-all') {
          if (choice === 'keep-copy-all') keepCopyAll = true;
          destPath = await firstAvailableCopyPath(destPath, reserved, path => invoke<boolean>('file_exists', { path }));
          keepCopySources.add(entry.path);
        }
      }
      if (!keepCopySources.has(entry.path)) {
        // Overwrite: delete existing first
        try {
          await invoke('delete_file', { path: destPath });
        } catch (e) {
          deps.showToast(`Failed to overwrite ${entry.name}: ${e}`);
          continue;
        }
      }
    }

    resolvedSources.push(entry.path);
    destinations.set(entry.path, destPath);
    if (!firstPastedPath) firstPastedPath = destPath;
  }

  if (resolvedSources.length === 0) return;

  // Phase 2: execute via TransferManager
  const tasks = buildLocalPasteTasks(resolvedSources, entries, destDir, operation, destinations, skipMap, renameMap, keepCopySources);
  const ids = await transfer.enqueueTransfers(tasks);
  deps.showToast(`Queued ${ids.length} transfer(s)`);
  if (ids.length > 0) deps.onOpenTransfer();

  // Cut: clear clipboard since source files no longer exist
  if (operation === 'cut') {
    clipboard.clear();
  }
}
