import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';

// --- State ---

// Compress dialog
let compressDialogVisible: boolean = $state(false);
let compressDialogValue: string = $state('');
let compressMarkPaths: string[] = $state([]);

// Paste conflict
let showConfirmModal: boolean = $state(false);
let confirmFileName: string = $state('');
let pasteResolve: ((choice: 'overwrite' | 'skip' | 'abort') => void) | null = null;

// Streaming conflict
let streamConflictVisible: boolean = $state(false);
let streamConflictName: string = $state('');
let streamConflictResolve: ((choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') => void) | null = null;
let scanningConflicts: boolean = $state(false);

// Unsaved changes
let showUnsavedConfirm: boolean = $state(false);
let pendingActionPath: string = $state('');
let pendingAction: 'navigate' | 'activate' = $state('activate');

// Dependency references (set once at init)
let _selectedFile: (() => string | null) | null = null;
let _previewEditor: (() => any) | null = null;
let _showToast: ((msg: string) => void) | null = null;
let _focusPanel: ((panel: 'parent' | 'current' | 'preview' | 'terminal') => void) | null = null;
let _currentPath: (() => string) | null = null;

// --- Getters ---

export function getCompressDialogVisible() { return compressDialogVisible; }
export function getCompressDialogValue() { return compressDialogValue; }
export function getCompressMarkPaths() { return compressMarkPaths; }
export function getShowConfirmModal() { return showConfirmModal; }
export function getConfirmFileName() { return confirmFileName; }
export function getStreamConflictVisible() { return streamConflictVisible; }
export function getStreamConflictName() { return streamConflictName; }
export function getScanningConflicts() { return scanningConflicts; }
export function getShowUnsavedConfirm() { return showUnsavedConfirm; }
export function getPendingActionPath() { return pendingActionPath; }
export function getPendingAction() { return pendingAction; }

// --- Setters ---

export function setCompressDialogVisible(v: boolean) { compressDialogVisible = v; }
export function setCompressDialogValue(v: string) { compressDialogValue = v; }
export function setCompressMarkPaths(v: string[]) { compressMarkPaths = v; }
export function setShowConfirmModal(v: boolean) { showConfirmModal = v; }
export function setConfirmFileName(v: string) { confirmFileName = v; }
export function setStreamConflictVisible(v: boolean) { streamConflictVisible = v; }
export function setStreamConflictName(v: string) { streamConflictName = v; }
export function setScanningConflicts(v: boolean) { scanningConflicts = v; }
export function setShowUnsavedConfirm(v: boolean) { showUnsavedConfirm = v; }
export function setPendingActionPath(v: string) { pendingActionPath = v; }
export function setPendingAction(v: 'navigate' | 'activate') { pendingAction = v; }

// --- Dependency injection ---

export function initDialogDeps(deps: {
  getSelectedFile: () => string | null;
  getPreviewEditor: () => any;
  showToast: (msg: string) => void;
  focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void;
  getCurrentPath: () => string;
}) {
  _selectedFile = deps.getSelectedFile;
  _previewEditor = deps.getPreviewEditor;
  _showToast = deps.showToast;
  _focusPanel = deps.focusPanel;
  _currentPath = deps.getCurrentPath;
}

// --- Prompt functions (used by paste/scan operations) ---

export function promptConflict(fileName: string): Promise<'overwrite' | 'skip' | 'abort'> {
  return new Promise(resolve => {
    confirmFileName = fileName;
    showConfirmModal = true;
    pasteResolve = resolve;
  });
}

export function promptConflictStream(fileName: string): Promise<'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort'> {
  return new Promise(resolve => {
    streamConflictName = fileName;
    streamConflictVisible = true;
    streamConflictResolve = resolve;
  });
}

export function resolveStreamConflict(choice: 'overwrite' | 'skip' | 'overwrite-all' | 'skip-all' | 'abort') {
  streamConflictVisible = false;
  streamConflictResolve?.(choice);
  streamConflictResolve = null;
}

// --- Confirm handlers (paste conflict) ---

export function handleConfirmOverwrite() {
  showConfirmModal = false;
  pasteResolve?.('overwrite');
  pasteResolve = null;
}

export function handleConfirmSkip() {
  showConfirmModal = false;
  pasteResolve?.('skip');
  pasteResolve = null;
}

export function handleConfirmAbort() {
  showConfirmModal = false;
  pasteResolve?.('abort');
  pasteResolve = null;
}

// --- Unsaved changes handlers ---

export async function handleUnsavedSave() {
  showUnsavedConfirm = false;
  const targetPath = pendingActionPath;
  const action = pendingAction;
  pendingActionPath = '';
  const filePath = _selectedFile?.();
  const content = _previewEditor?.()?.getContent();
  if (filePath && content !== undefined) {
    try {
      await invoke('write_file', { path: filePath, content });
      _previewEditor?.()?.setContent(content);
    } catch (e) {
      _showToast?.(`Failed to save: ${e}`);
      return;
    }
  }
  dispatchPendingAction(targetPath, action);
}

export function handleUnsavedDiscard() {
  showUnsavedConfirm = false;
  const targetPath = pendingActionPath;
  const action = pendingAction;
  pendingActionPath = '';
  dispatchPendingAction(targetPath, action);
}

export function handleUnsavedCancel() {
  showUnsavedConfirm = false;
  pendingActionPath = '';
  _focusPanel?.('preview');
}

function dispatchPendingAction(path: string, action: 'navigate' | 'activate') {
  if (action === 'navigate') {
    layout.setCurrentPath(path);
  } else {
    layout.setSelectedFile(path);
    if (!get(layout).previewExpanded) {
      layout.expandPreview();
    }
    _focusPanel?.('preview');
  }
}

// --- Compress dialog handlers ---

export async function handleCompressDialogConfirm(value: string, currentDirectoryPanel: any) {
  compressDialogVisible = false;
  if (!value || compressMarkPaths.length === 0) {
    layout.clearMark();
    return;
  }
  const currentPathVal = _currentPath?.() || '';
  const destPath = currentPathVal.replace(/[\\/]+$/, '') + '\\' + value;
  try {
    await invoke('compress_files', { sources: compressMarkPaths, destPath });
    _showToast?.(`Archive created: ${value}`);
    layout.clearMark();
    currentDirectoryPanel?.refresh();
  } catch (e) {
    _showToast?.(`Compress failed: ${e}`);
  }
}

export function handleCompressDialogCancel() {
  compressDialogVisible = false;
  layout.clearMark();
}