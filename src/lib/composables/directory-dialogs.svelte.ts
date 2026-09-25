import { invoke } from '@tauri-apps/api/core';
import type { FileEntry } from '$lib/types/file-explorer';

interface ArchiveState {
  archivePath: string;
  internalPath: string;
  format: string;
}

interface DialogDeps {
  getSelectedIndex: () => number;
  getDisplayFiles: () => FileEntry[];
  isArchiveMode: () => boolean;
  getArchiveState: () => ArchiveState | null;
  getPath: () => string;
  getOperationDirectory: () => string;
  refresh: () => void;
  loadDirectory: (path: string, force: boolean) => Promise<void>;
  onSelect: (path: string, isDir?: boolean) => void;
  onToast: (message: string) => void;
  onBatchRenameStart: (files: { path: string; name: string }[]) => void;
  refocusPanel: () => void;
  collectEntriesToOperate: () => { path: string; name: string }[];
  setFilterPattern: (pattern: string) => void;
  setFilterMode: (mode: 'prefix' | 'wildcard') => void;
}

export interface DirectoryDialogsAPI {
  // Input dialog state
  getInputVisible: () => boolean;
  getInputValue: () => string;
  getInputPlaceholder: () => string;
  getInputPrompt: () => string;
  // Delete confirm state
  getShowDeleteConfirm: () => boolean;
  getDeleteIsPermanent: () => boolean;
  // File info state
  getShowFileInfo: () => boolean;
  getFileInfo: () => any;

  // Actions
  startRename: () => void;
  startCreateFile: () => void;
  startCreateDir: () => void;
  startCompress: (defaultName: string) => void;
  startFilter: (mode?: 'prefix' | 'wildcard') => void;
  handleInputConfirm: (value: string) => Promise<void>;
  handleInputCancel: () => void;
  promptDelete: (permanent: boolean) => Promise<boolean>;
  handleDeleteConfirm: () => void;
  handleDeleteCancel: () => void;
  toggleFileInfo: () => Promise<void>;
  closeFileInfo: () => void;
}

export function createDirectoryDialogs(deps: DialogDeps): DirectoryDialogsAPI {
  // Input dialog state
  let inputVisible: boolean = $state(false);
  let inputValue: string = $state('');
  let inputPlaceholder: string = $state('');
  let inputPrompt: string = $state('');
  let inputMode: 'rename' | 'create-file' | 'create-dir' | 'filter' | 'compress' = $state('rename');

  // Delete confirmation state
  let showDeleteConfirm: boolean = $state(false);
  let deleteIsPermanent: boolean = $state(false);
  let deleteResolve: ((confirm: boolean) => void) | null = null;

  // File info state
  let showFileInfo: boolean = $state(false);
  let fileInfo: any = $state(null);

  function startRename() {
    const selectedIndex = deps.getSelectedIndex();
    const displayFiles = deps.getDisplayFiles();
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    inputMode = 'rename';
    inputValue = entry.name;
    inputPlaceholder = '';
    inputPrompt = 'Rename:';
    inputVisible = true;
  }

  function startCreateFile() {
    inputMode = 'create-file';
    inputValue = '';
    inputPlaceholder = 'New file name';
    inputPrompt = 'New file:';
    inputVisible = true;
  }

  function startCreateDir() {
    inputMode = 'create-dir';
    inputValue = '';
    inputPlaceholder = 'New directory name';
    inputPrompt = 'New dir:';
    inputVisible = true;
  }

  function startCompress(defaultName: string) {
    inputMode = 'compress';
    inputValue = defaultName;
    inputPlaceholder = 'Archive name';
    inputPrompt = 'Archive name:';
    inputVisible = true;
  }

  function startFilter(mode: 'prefix' | 'wildcard' = 'wildcard') {
    deps.setFilterMode(mode);
    inputMode = 'filter';
    inputValue = '';
    if (mode === 'prefix') {
      inputPlaceholder = 'Enter prefix to filter...';
      inputPrompt = 'Prefix:';
    } else {
      inputPlaceholder = '*.txt, *.rs, *.{js,ts}';
      inputPrompt = 'Filter:';
    }
    inputVisible = true;
  }

  async function handleInputConfirm(value: string) {
    inputVisible = false;
    const selectedIndex = deps.getSelectedIndex();
    const displayFiles = deps.getDisplayFiles();
    const isArchive = deps.isArchiveMode();
    const archiveState = deps.getArchiveState();
    const path = deps.getPath();

    try {
      if (inputMode === 'rename') {
        const entry = displayFiles[selectedIndex];
        if (isArchive && archiveState) {
          const oldInternalPath = entry.path;
          const parentParts = oldInternalPath.split('/');
          parentParts.pop();
          const newInternalPath = parentParts.length > 0 ? parentParts.join('/') + '/' + value : value;
          await invoke('archive_rename_entry', {
            archivePath: archiveState.archivePath,
            oldPath: oldInternalPath,
            newPath: newInternalPath,
          });
          deps.refresh();
          deps.onToast(`Renamed to ${value}`);
        } else if (entry.path.startsWith('ftp://')) {
          const parentBase = path.replace(/\/+$/, '');
          const newPath = await invoke<string>('ftp_rename', { oldPath: entry.path, newPath: parentBase + '/' + value });
          deps.loadDirectory(path, true);
          deps.onSelect(newPath, false);
          deps.onToast(`Renamed to ${value}`);
        } else {
          const parentPath = path.replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('rename_file', { oldPath: entry.path, newName: value });
          deps.refresh();
          deps.onSelect(newPath, false);
          deps.onToast(`Renamed to ${value}`);
        }
      } else if (inputMode === 'create-file') {
        if (isArchive && archiveState) {
          if (archiveState.format !== 'zip') {
            deps.onToast('Create file is only supported for ZIP archives');
            return;
          }
          const parentPath = archiveState.internalPath;
          const newInternalPath = parentPath ? parentPath + '/' + value : value;
          await invoke('archive_create_entry', {
            archivePath: archiveState.archivePath,
            internalPath: newInternalPath,
            isDir: false,
          });
          deps.refresh();
          deps.onToast(`Created ${value}`);
        } else if (path.startsWith('ftp://')) {
          const remotePath = path.replace(/\/+$/, '') + '/' + value;
          await invoke('ftp_create_file', { path: remotePath });
          deps.loadDirectory(path, true);
          deps.onToast(`Created ${value}`);
        } else {
          const parentPath = deps.getOperationDirectory().replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('create_file', { path: newPath, isDir: false });
          deps.refresh();
          deps.onSelect(newPath, false);
          deps.onToast(`Created ${value}`);
        }
      } else if (inputMode === 'create-dir') {
        if (isArchive && archiveState) {
          if (archiveState.format !== 'zip') {
            deps.onToast('Create directory is only supported for ZIP archives');
            return;
          }
          const parentPath = archiveState.internalPath;
          const newInternalPath = parentPath ? parentPath + '/' + value : value;
          await invoke('archive_create_entry', {
            archivePath: archiveState.archivePath,
            internalPath: newInternalPath,
            isDir: true,
          });
          deps.refresh();
          deps.onToast(`Created ${value}/`);
        } else if (path.startsWith('ftp://')) {
          const remotePath = path.replace(/\/+$/, '') + '/' + value;
          await invoke('ftp_mkdir', { path: remotePath });
          deps.loadDirectory(path, true);
          deps.onToast(`Created ${value}/`);
        } else {
          const parentPath = deps.getOperationDirectory().replace(/[\\\/]+$/, '');
          const newPath = parentPath + '\\' + value;
          await invoke('create_file', { path: newPath, isDir: true });
          deps.refresh();
          deps.onSelect(newPath, true);
          deps.onToast(`Created ${value}/`);
        }
      } else if (inputMode === 'filter') {
        deps.setFilterPattern(value);
        if (value) {
          deps.onToast(`Filter: ${value}`);
        } else {
          deps.onToast('Filter cleared');
        }
      } else if (inputMode === 'compress') {
        const entries = deps.collectEntriesToOperate();
        if (entries.length === 0) return;
        const destPath = path.replace(/[\\\/]+$/, '') + '\\' + value;
        try {
          await invoke('compress_files', { sources: entries.map(e => e.path), destPath });
          deps.onToast(`Archive created: ${value}`);
          deps.refresh();
        } catch (e) {
          deps.onToast(`Compress failed: ${e}`);
        }
      }
    } catch (e) {
      deps.onToast(`Error: ${e}`);
    }
    deps.refocusPanel();
  }

  function handleInputCancel() {
    inputVisible = false;
    deps.refocusPanel();
  }

  function promptDelete(permanent: boolean): Promise<boolean> {
    return new Promise(resolve => {
      deleteIsPermanent = permanent;
      showDeleteConfirm = true;
      deleteResolve = resolve;
    });
  }

  function handleDeleteConfirm() {
    showDeleteConfirm = false;
    deleteResolve?.(true);
    deleteResolve = null;
    deps.refocusPanel();
  }

  function handleDeleteCancel() {
    showDeleteConfirm = false;
    deleteResolve?.(false);
    deleteResolve = null;
    deps.refocusPanel();
  }

  async function toggleFileInfo() {
    if (showFileInfo) {
      showFileInfo = false;
      fileInfo = null;
      return;
    }
    const selectedIndex = deps.getSelectedIndex();
    const displayFiles = deps.getDisplayFiles();
    if (selectedIndex < 0 || selectedIndex >= displayFiles.length) return;
    const entry = displayFiles[selectedIndex];
    if (entry.name === '..') return;
    try {
      fileInfo = await invoke('get_file_info', { path: entry.path });
      showFileInfo = true;
    } catch (e) {
      deps.onToast(`Failed to get file info: ${e}`);
    }
  }

  function closeFileInfo() {
    showFileInfo = false;
    fileInfo = null;
    deps.refocusPanel();
  }

  return {
    getInputVisible: () => inputVisible,
    getInputValue: () => inputValue,
    getInputPlaceholder: () => inputPlaceholder,
    getInputPrompt: () => inputPrompt,
    getShowDeleteConfirm: () => showDeleteConfirm,
    getDeleteIsPermanent: () => deleteIsPermanent,
    getShowFileInfo: () => showFileInfo,
    getFileInfo: () => fileInfo,

    startRename,
    startCreateFile,
    startCreateDir,
    startCompress,
    startFilter,
    handleInputConfirm,
    handleInputCancel,
    promptDelete,
    handleDeleteConfirm,
    handleDeleteCancel,
    toggleFileInfo,
    closeFileInfo,
  };
}
