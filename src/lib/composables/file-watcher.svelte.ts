import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface FileWatcherDeps {
  getFilePath: () => string | null;
  getMode: () => string;
  getRenderTabId: () => number;
  onFileChanged: (path: string) => Promise<void>;
}

export interface FileWatcherAPI {
  startWatching: (path: string) => void;
  stopWatching: () => void;
  getFileMtime: () => number;
  setFileMtime: (mtime: number) => void;
  setup: () => void;
  teardown: () => void;
}

export function createFileWatcher(deps: FileWatcherDeps): FileWatcherAPI {
  let currentFileMtime: number = $state(0);
  let fileChangedUnlisten: (() => void) | null = null;

  function startWatching(path: string) {
    stopWatching();
    invoke('start_watch_file', { path }).catch(e => console.error('[FileWatcher] start_watch_file error:', e));
  }

  function stopWatching() {
    invoke('stop_watch_file').catch(() => {});
  }

  function setup() {
    listen('file-changed', (event: any) => {
      const changedPath = typeof event.payload === 'string' ? event.payload : String(event.payload ?? '');
      deps.onFileChanged(changedPath);
    }).then(unlisten => { fileChangedUnlisten = unlisten; });
  }

  function teardown() {
    stopWatching();
    if (fileChangedUnlisten) {
      fileChangedUnlisten();
      fileChangedUnlisten = null;
    }
  }

  return {
    startWatching,
    stopWatching,
    getFileMtime: () => currentFileMtime,
    setFileMtime(mtime: number) { currentFileMtime = mtime; },
    setup,
    teardown,
  };
}