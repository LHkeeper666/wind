import { isImageFile, isPdfFile, isVideoFile } from '$lib/utils/file-types';
import { layout } from '$lib/stores/layout';

// Fullscreen image viewer state
let fullscreenImageList: { name: string; path: string }[] = $state([]);
let fullscreenImageIndex: number = $state(0);

// Fullscreen PDF viewer state
let fullscreenPdfPath: string = $state('');
let fullscreenPdfPage: number = $state(0);
let fullscreenPdfPageCount: number = $state(0);
let fullscreenPdfFileSize: number = $state(0);

// Fullscreen video player state
let fullscreenVideoPlayerPath: string = $state('');
let fullscreenVideoPlayerFileSize: number = $state(0);

// Track active column before fullscreen for restoration
let preFullscreenColumn: 'parent' | 'current' | 'preview' | 'terminal' | null = $state(null);

// Editor initial line for fullscreen editor
let editorInitialLine: number = $state(0);

// Getters
export function getFullscreenImageList() { return fullscreenImageList; }
export function getFullscreenImageIndex() { return fullscreenImageIndex; }
export function getFullscreenPdfPath() { return fullscreenPdfPath; }
export function getFullscreenPdfPage() { return fullscreenPdfPage; }
export function getFullscreenPdfPageCount() { return fullscreenPdfPageCount; }
export function getFullscreenPdfFileSize() { return fullscreenPdfFileSize; }
export function getFullscreenVideoPlayerPath() { return fullscreenVideoPlayerPath; }
export function getFullscreenVideoPlayerFileSize() { return fullscreenVideoPlayerFileSize; }
export function getPreFullscreenColumn() { return preFullscreenColumn; }
export function getEditorInitialLine() { return editorInitialLine; }

// Setters
export function setFullscreenImageList(v: { name: string; path: string }[]) { fullscreenImageList = v; }
export function setFullscreenImageIndex(v: number) { fullscreenImageIndex = v; }
export function setFullscreenPdfPath(v: string) { fullscreenPdfPath = v; }
export function setFullscreenPdfPage(v: number) { fullscreenPdfPage = v; }
export function setFullscreenPdfPageCount(v: number) { fullscreenPdfPageCount = v; }
export function setFullscreenPdfFileSize(v: number) { fullscreenPdfFileSize = v; }
export function setFullscreenVideoPlayerPath(v: string) { fullscreenVideoPlayerPath = v; }
export function setFullscreenVideoPlayerFileSize(v: number) { fullscreenVideoPlayerFileSize = v; }
export function setEditorInitialLine(v: number) { editorInitialLine = v; }

export function handleFullscreenEditor(
  selectedFile: string | null,
  currentDirectoryPanel: any,
  previewEditor: any,
) {
  if (selectedFile && isImageFile(selectedFile)) {
    const imageFiles = currentDirectoryPanel?.getImageFiles() || [];
    const idx = imageFiles.findIndex((f: any) => f.path === selectedFile);
    fullscreenImageList = imageFiles;
    fullscreenImageIndex = idx >= 0 ? idx : 0;
    const unsub = layout.subscribe(v => { preFullscreenColumn = v.activeColumn; })();
    layout.setFullscreenViewer('image');
  } else if (selectedFile && isPdfFile(selectedFile)) {
    const pdfInfo = previewEditor?.getPdfInfo();
    fullscreenPdfPath = selectedFile;
    fullscreenPdfPage = pdfInfo?.currentPage ?? 0;
    fullscreenPdfPageCount = pdfInfo?.pageCount ?? 0;
    fullscreenPdfFileSize = 0;
    const unsub = layout.subscribe(v => { preFullscreenColumn = v.activeColumn; })();
    layout.setFullscreenViewer('pdf');
  } else if (selectedFile && isVideoFile(selectedFile)) {
    fullscreenVideoPlayerPath = selectedFile;
    fullscreenVideoPlayerFileSize = currentDirectoryPanel?.getSelectedFileSize() ?? 0;
    const unsub = layout.subscribe(v => { preFullscreenColumn = v.activeColumn; })();
    layout.setFullscreenViewer('video');
  } else {
    const unsub = layout.subscribe(v => { preFullscreenColumn = v.activeColumn; })();
    editorInitialLine = previewEditor?.getVisibleLine() ?? 0;
    layout.setFullscreenViewer('editor');
  }
}

export function handleCloseFullscreen(focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void) {
  layout.closeFullscreenViewer();
  const restoreTo = preFullscreenColumn || 'current';
  preFullscreenColumn = null;
  focusPanel(restoreTo);
}

export function handleSaveFullscreen(content: string, previewEditor: any) {
  if (previewEditor) {
    previewEditor.setContent(content);
  }
}

export function handleCloseImageViewer(focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void) {
  layout.closeFullscreenViewer();
  const restoreTo = preFullscreenColumn || 'current';
  preFullscreenColumn = null;
  focusPanel(restoreTo);
}

export function handleClosePdfViewer(focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void) {
  layout.closeFullscreenViewer();
  const restoreTo = preFullscreenColumn || 'current';
  preFullscreenColumn = null;
  focusPanel(restoreTo);
}

export function handleCloseVideoPlayer(focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void) {
  layout.closeFullscreenViewer();
  fullscreenVideoPlayerPath = '';
  const restoreTo = preFullscreenColumn || 'current';
  preFullscreenColumn = null;
  focusPanel(restoreTo);
}

export function handleImageViewerNavigate(index: number) {
  fullscreenImageIndex = index;
}