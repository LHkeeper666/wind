import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';
import { clipboard } from '$lib/stores/clipboard';
import { transfer } from '$lib/stores/transfer';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';
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

// --- Conflict scanning ---

export async function scanConflicts(
  startScan: () => Promise<unknown>,
): Promise<{ dirSkipMap: Map<string, string[]>; fileSkipSet: Set<string> } | null> {
  const dirSkipMap = new Map<string, string[]>();
  const fileSkipSet = new Set<string>();
  let applyToAll: 'overwrite' | 'skip' | null = null;
  let queue: Array<{ kind: string; dir_source: string; rel_path: string }> = [];
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

  return new Promise((resolve) => {
    let unlistenFound: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;

    const maybeFinish = () => {
      if (finished && !processing && queue.length === 0) {
        unlistenFound?.();
        unlistenDone?.();
        resolve(aborted ? null : { dirSkipMap, fileSkipSet });
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
        } else {
          const choice = await promptConflictStream(c.rel_path);
          if (choice === 'skip') addSkip(c.kind, c.dir_source, c.rel_path);
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
      const extracted = await invokeArchiveWithOptionalPassword<number>(
        'extract_archive',
        { archivePath, destDir },
        'password'
      );
      if (extracted === null) return;
      deps.showToast('Archive extracted');
      layout.clearMark();
      await deps.refreshPanels([destDir]);
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
      const extracted = await invokeArchiveWithOptionalPassword<number>(
        'extract_archive_files',
        {
          archivePath: state.archivePath,
          internalPaths,
          destDir,
        },
        'password'
      );
      if (extracted === null) return;
      deps.showToast(`${state.entries.length} ${state.entries.length === 1 ? 'file' : 'files'} extracted from archive`);
      clipboard.clear();
      layout.clearMark();
      await deps.refreshPanels([destDir]);
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
        await invoke('ftp_download_folder', {
          connName: getFtpConnName(dir.path),
          remotePath: getFtpRemotePath(dir.path),
          localPath: destDir + '\\' + dir.name,
          moveMode: operation === 'cut',
          skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
        });
        totalQueued++;
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
      }

      if (totalQueued > 0) {
        deps.showToast(`Queued ${totalQueued} download(s)`);
        deps.onOpenTransfer();
      }

      if (operation === 'cut') {
        for (const entry of files) {
          if (skipFiles.has(entry.name)) continue;
          await invoke('ftp_delete', { path: entry.path, permanent: true }).catch(() => {});
        }
        for (const dir of dirs) {
          const hasSkipped = (skipDirMap.get(dir.path) || []).length > 0 || (dir.skip_rel_paths || []).length > 0;
          if (hasSkipped) continue;
          await invoke('ftp_delete', { path: dir.path, permanent: true }).catch(() => {});
        }
      }
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
        await invoke('ftp_upload_folder', {
          connName: destConn,
          localPath: dir.path,
          remotePath: `${destBase}/${dir.name}`,
          moveMode: operation === 'cut',
          skipRelPaths: [...skipRel, ...(dir.skip_rel_paths || [])],
        });
        totalQueued++;
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
      }

      if (totalQueued > 0) {
        deps.showToast(`Queued ${totalQueued} upload(s)`);
        deps.onOpenTransfer();
      }

      if (operation === 'cut') {
        const delEntries = entries.filter((e: any) => !e.is_dir && !skipFiles.has(e.name));
        if (delEntries.length > 0) {
          const delTasks = delEntries.map((entry: any) => ({
            op_type: 'delete' as const,
            source: entry.path,
            destination: '',
            total_bytes: entry.size || 0,
          }));
          await transfer.enqueueTransfers(delTasks);
        }
        for (const dir of dirs) {
          const hasSkipped = (skipDirMap.get(dir.name) || []).length > 0 || (dir.skip_rel_paths || []).length > 0;
          if (hasSkipped) continue;
          await invoke('delete_file', { path: dir.path }).catch(() => {});
        }
      }
    } else if (srcIsFtp && destIsFtp) {
      // FTP → FTP
      const srcConn = getFtpConnName(entries[0].path);
      const destConn = getFtpConnName(destinationPath);
      if (srcConn === destConn) {
        const destBase = getFtpDestPath(destinationPath, '').replace(/\/+$/, '');
        if (operation === 'cut') {
          for (const entry of entries) {
            const newFullPath = destBase + '/' + entry.name;
            try {
              await invoke('ftp_rename', { oldPath: entry.path, newPath: newFullPath });
            } catch (e: any) {
              deps.showToast(`Move failed: ${e}`);
            }
          }
          deps.showToast(`Moved ${entries.length} item(s) on ${srcConn}`);
        } else {
          let done = 0;
          for (const entry of entries) {
            deps.showToast(`Copying ${entry.name} (${done + 1}/${entries.length})...`);
            const newRemote = destBase + '/' + entry.name;
            await invoke('ftp_copy', {
              connName: srcConn,
              srcPath: getFtpRemotePath(entry.path),
              dstPath: newRemote,
            }).catch((e: any) => { deps.showToast(`Copy failed: ${e}`); });
            done++;
          }
          deps.showToast(`Copied ${entries.length} item(s) on ${srcConn}`);
        }
      } else {
        deps.showToast('Cross-server transfer not yet supported');
      }
    }

    if (operation === 'cut') {
      clipboard.clear();
    }
    return;
  }

  // Local-to-local paste
  const destDir = destinationPath.replace(/[\\\/]+$/, '');
  let processed = 0;
  let firstPastedPath: string | null = null;
  const resolvedSources: string[] = [];
  const skipMap = new Map<string, string[]>();

  const dirEntries = entries.filter((e: any) => e.is_dir);
  const fileEntries = entries.filter((e: any) => !e.is_dir);

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
      const result = await scanConflicts(() => invoke('scan_transfer_conflicts', { tasks: dirTasks }));
      setScanningConflicts(false);
      if (result === null) {
        deps.showToast('Paste aborted');
        return;
      }
      for (const dir of dirEntries) {
        resolvedSources.push(dir.path);
        if (!firstPastedPath) firstPastedPath = destDir + '\\' + dir.name;
        const skip = result.dirSkipMap.get(dir.path);
        if (skip) skipMap.set(dir.path, skip);
      }
    }
  }

  // Phase 1b: files — existing sync conflict check
  for (const entry of fileEntries) {
    const destPath = destDir + '\\' + entry.name;

    let exists = false;
    try {
      exists = await invoke<boolean>('file_exists', { path: destPath });
    } catch {
      exists = false;
    }

    if (exists) {
      if (!force) {
        const choice = await promptConflict(entry.name);
        if (choice === 'abort') {
          deps.showToast(`Paste aborted (${processed}/${entries.length} done)`);
          return;
        }
        if (choice === 'skip') {
          continue;
        }
      }
      // Overwrite: delete existing first
      try {
        await invoke('delete_file', { path: destPath });
      } catch (e) {
        deps.showToast(`Failed to overwrite ${entry.name}: ${e}`);
        continue;
      }
    }

    resolvedSources.push(entry.path);
    if (!firstPastedPath) firstPastedPath = destPath;
  }

  if (resolvedSources.length === 0) return;

  // Phase 2: execute via TransferManager
  const tasks = resolvedSources.map((src: string) => {
    const origEntry = entries.find((e: any) => e.path === src);
    return {
      op_type: operation === 'copy' ? 'copy' : 'move',
      source: src,
      destination: destDir + '\\' + (src.split(/[/\\]/).pop() || src),
      total_bytes: origEntry?.size || 0,
      skip_rel_paths: [...(origEntry?.skip_rel_paths || []), ...(skipMap.get(src) || [])],
    };
  });
  const ids = await transfer.enqueueTransfers(tasks);
  deps.showToast(`Queued ${ids.length} transfer(s)`);
  if (ids.length > 0) deps.onOpenTransfer();

  // Cut: clear clipboard since source files no longer exist
  if (operation === 'cut') {
    clipboard.clear();
  }
}