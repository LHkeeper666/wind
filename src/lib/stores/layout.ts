import { writable, derived } from 'svelte/store';

export type ArchiveFormat = 'zip' | 'tar' | 'tar.gz' | '7z';

export interface ArchiveState {
  archivePath: string;
  internalPath: string;
  format: ArchiveFormat;
}

export type MarkType = 'copy' | 'cut' | 'extract' | 'compress' | null;

export interface LayoutState {
  // Column ratios [parent, current, preview]
  columnRatios: [number, number, number];

  // Current navigation paths
  parentPath: string;
  currentPath: string;

  // Left panel detach mode
  leftMode: 'auto' | 'manual';
  leftPath: string;

  // Selected file in current directory
  selectedFile: string | null;

  // Active column for focus
  activeColumn: 'parent' | 'current' | 'preview' | 'terminal';

  // Preview/Editor mode
  previewMode: 'global-normal' | 'editor-normal' | 'editor-insert';

  // Fullscreen editor state
  fullscreenEditorOpen: boolean;

  // Fullscreen image viewer state
  fullscreenImageViewerOpen: boolean;

  // Fullscreen PDF viewer state
  fullscreenPdfViewerOpen: boolean;

  // Fullscreen video player state
  fullscreenVideoPlayerOpen: boolean;

  // Fullscreen terminal state
  fullscreenTerminalOpen: boolean;

  // Terminal state
  terminalVisible: boolean;
  terminalHeight: number;

  // Key prefix display (e.g. 't', '^W')
  keyPrefix: string | null;

  // Expanded preview mode (0:1:4 layout, hides parent)
  previewExpanded: boolean;
  originalRatios: [number, number, number];

  // Pre-terminal active column for focus restore on close
  preTerminalColumn: 'parent' | 'current' | 'preview' | 'terminal';

  // Recycle bin mode
  recycleBinMode: boolean;
  recycleBinOriginalRatios: [number, number, number];

  // Archive browsing state
  archiveState: ArchiveState | null;

  // Unified mark state for E/y/x mutual exclusion
  markType: MarkType;
  markPaths: string[];
}

const initialState: LayoutState = {
  columnRatios: [1, 1, 3],
  parentPath: '',
  currentPath: '',
  leftMode: 'auto',
  leftPath: '',
  selectedFile: null,
  activeColumn: 'current',
  previewMode: 'global-normal',
  fullscreenEditorOpen: false,
  fullscreenImageViewerOpen: false,
  fullscreenPdfViewerOpen: false,
  fullscreenVideoPlayerOpen: false,
  fullscreenTerminalOpen: false,
  terminalVisible: false,
  terminalHeight: 300,
  keyPrefix: null,
  previewExpanded: false,
  originalRatios: [1, 1, 3],
  preTerminalColumn: 'current',
  recycleBinMode: false,
  recycleBinOriginalRatios: [1, 1, 3],
  archiveState: null,
  markType: null,
  markPaths: [],
};

function createLayoutStore() {
  const { subscribe, set, update } = writable<LayoutState>(initialState);

  return {
    subscribe,

    // Apply all tab state fields in a single store update to avoid
    // cascading reactive triggers from multiple individual set* calls.
    restoreTabState(partial: {
      columnRatios: [number, number, number];
      previewExpanded: boolean;
      originalRatios: [number, number, number];
      currentPath: string;
      selectedFile: string | null;
      terminalVisible: boolean;
      terminalHeight: number;
      fullscreenTerminalOpen: boolean;
      leftMode?: 'auto' | 'manual';
      leftPath?: string;
      archiveState?: ArchiveState | null;
    }) {
      update(state => {
        let normalized: string;
        let parentPath: string;

        if (partial.currentPath.startsWith('ftp://')) {
          normalized = partial.currentPath;
          const lastSlash = partial.currentPath.lastIndexOf('/');
          parentPath = lastSlash > 6 ? partial.currentPath.substring(0, lastSlash) : '\\';
        } else {
          normalized = partial.currentPath.replace(/\//g, '\\');
          if (/^[A-Za-z]:$/.test(normalized)) normalized += '\\';
          if (normalized === '\\' || /^[A-Za-z]:\\$/.test(normalized)) {
            parentPath = '\\';
          } else {
            const lastSlash = normalized.lastIndexOf('\\');
            parentPath = lastSlash > 0 ? normalized.substring(0, lastSlash) : '\\';
          }
        }
        return {
          ...state,
          columnRatios: [...partial.columnRatios] as [number, number, number],
          previewExpanded: partial.previewExpanded,
          originalRatios: [...partial.originalRatios] as [number, number, number],
          currentPath: normalized,
          parentPath,
          selectedFile: partial.selectedFile,
          terminalVisible: partial.terminalVisible,
          terminalHeight: partial.terminalHeight,
          fullscreenTerminalOpen: partial.fullscreenTerminalOpen,
          leftMode: partial.leftMode ?? state.leftMode,
          leftPath: partial.leftPath ?? state.leftPath,
          archiveState: partial.archiveState ?? state.archiveState,
        };
      });
    },

    // Set column ratios
    setRatios(ratios: [number, number, number]) {
      update(state => ({ ...state, columnRatios: ratios }));
    },

    // Update current path and auto-update parent path
    setCurrentPath(path: string, resetSelectedFile: boolean = true) {
      update(state => {
        let normalized: string;
        let parentPath: string;

        if (path.startsWith('ftp://')) {
          // FTP paths: keep forward slashes
          normalized = path;
          const lastSlash = path.lastIndexOf('/');
          if (lastSlash <= 6) {
            // ftp://name/ — root of connection
            parentPath = '\\';
          } else {
            parentPath = path.substring(0, lastSlash);
          }
        } else {
          // 规范化路径分隔符
          normalized = path.replace(/\//g, '\\');
          // 确保驱动器根目录格式为 X:\（不是 X:）
          if (/^[A-Za-z]:$/.test(normalized)) {
            normalized = normalized + '\\';
          }

          if (normalized === '\\') {
            parentPath = '\\';
          } else if (/^[A-Za-z]:\\$/.test(normalized)) {
            parentPath = '\\';
          } else {
            const lastSlash = normalized.lastIndexOf('\\');
            parentPath = lastSlash > 0 ? normalized.substring(0, lastSlash) : '\\';
          }
        }

        return {
          ...state,
          currentPath: normalized,
          parentPath: parentPath,
          selectedFile: resetSelectedFile ? null : state.selectedFile,
        };
      });
    },

    // Set selected file
    setSelectedFile(filePath: string | null) {
      update(state => ({ ...state, selectedFile: filePath }));
    },

    // Set active column
    setActiveColumn(column: 'parent' | 'current' | 'preview' | 'terminal') {
      update(state => ({ ...state, activeColumn: column }));
    },

    // Set terminal height
    setTerminalHeight(height: number) {
      update(state => ({ ...state, terminalHeight: height }));
    },

    // Set preview mode
    setPreviewMode(mode: 'global-normal' | 'editor-normal' | 'editor-insert') {
      update(state => ({ ...state, previewMode: mode }));
    },

    // Toggle fullscreen editor
    toggleFullscreenEditor() {
      update(state => ({ ...state, fullscreenEditorOpen: !state.fullscreenEditorOpen }));
    },

    // Open fullscreen editor
    openFullscreenEditor() {
      update(state => ({ ...state, fullscreenEditorOpen: true }));
    },

    // Close fullscreen editor
    closeFullscreenEditor() {
      update(state => ({ ...state, fullscreenEditorOpen: false }));
    },

    // Open fullscreen image viewer
    openFullscreenImageViewer() {
      update(state => ({ ...state, fullscreenImageViewerOpen: true }));
    },

    // Close fullscreen image viewer
    closeFullscreenImageViewer() {
      update(state => ({ ...state, fullscreenImageViewerOpen: false }));
    },

    // Open fullscreen PDF viewer
    openFullscreenPdfViewer() {
      update(state => ({ ...state, fullscreenPdfViewerOpen: true }));
    },

    // Close fullscreen PDF viewer
    closeFullscreenPdfViewer() {
      update(state => ({ ...state, fullscreenPdfViewerOpen: false }));
    },

    // Open fullscreen video player
    openFullscreenVideoPlayer() {
      update(state => ({ ...state, fullscreenVideoPlayerOpen: true }));
    },

    // Close fullscreen video player
    closeFullscreenVideoPlayer() {
      update(state => ({ ...state, fullscreenVideoPlayerOpen: false }));
    },

    // Open fullscreen terminal
    openFullscreenTerminal() {
      update(state => ({ ...state, fullscreenTerminalOpen: true, activeColumn: 'terminal' }));
    },

    // Close fullscreen terminal
    closeFullscreenTerminal() {
      update(state => ({ ...state, fullscreenTerminalOpen: false }));
    },

    // Toggle fullscreen terminal
    toggleFullscreenTerminal() {
      update(state => ({
        ...state,
        fullscreenTerminalOpen: !state.fullscreenTerminalOpen,
        activeColumn: !state.fullscreenTerminalOpen ? 'terminal' : state.activeColumn,
      }));
    },

    // Enter expanded preview mode
    expandPreview() {
      update(state => ({
        ...state,
        originalRatios: [...state.columnRatios] as [number, number, number],
        columnRatios: [0, 1, 4],
        previewExpanded: true,
      }));
    },

    // Exit expanded preview mode
    collapsePreview() {
      update(state => ({
        ...state,
        columnRatios: [...state.originalRatios] as [number, number, number],
        previewExpanded: false,
      }));
    },

    // Set key prefix for status bar display
    setKeyPrefix(prefix: string) {
      update(state => ({ ...state, keyPrefix: prefix }));
    },

    // Clear key prefix
    clearKeyPrefix() {
      update(state => ({ ...state, keyPrefix: null }));
    },

    // Toggle terminal visibility
    toggleTerminal() {
      update(state => ({
        ...state,
        terminalVisible: !state.terminalVisible,
        activeColumn: !state.terminalVisible ? 'terminal'
          : state.activeColumn === 'terminal' ? state.preTerminalColumn : state.activeColumn,
        preTerminalColumn: !state.terminalVisible ? state.activeColumn : state.preTerminalColumn,
      }));
    },

    // Show terminal
    showTerminal() {
      update(state => ({
        ...state,
        terminalVisible: true,
        activeColumn: 'terminal',
        preTerminalColumn: state.activeColumn === 'terminal' ? state.preTerminalColumn : state.activeColumn,
      }));
    },

    // Remember which column was active before switching to terminal
    setPreTerminalColumn(column: 'parent' | 'current' | 'preview') {
      update(state => ({ ...state, preTerminalColumn: column }));
    },

    // Hide terminal — restore pre-terminal focus
    hideTerminal() {
      update(state => ({
        ...state,
        terminalVisible: false,
        activeColumn: state.activeColumn === 'terminal' ? state.preTerminalColumn : state.activeColumn,
      }));
    },

    // Detach left panel — freeze current path and enter manual mode
    detach() {
      update(state => ({
        ...state,
        leftMode: 'manual',
        leftPath: state.parentPath,
      }));
    },

    // Attach left panel — restore auto mode, follow center panel's parent
    attach() {
      update(state => ({
        ...state,
        leftMode: 'auto',
      }));
    },

    // Toggle detach/attach
    toggleDetach() {
      update(state => {
        if (state.leftMode === 'auto') {
          return { ...state, leftMode: 'manual', leftPath: state.parentPath };
        } else {
          return { ...state, leftMode: 'auto' };
        }
      });
    },

    // Set left panel path (manual mode)
    setLeftPath(path: string) {
      update(state => ({
        ...state,
        leftPath: path,
      }));
    },

    // Enter recycle bin mode with ratio adjustment
    recycleBinEnter() {
      update(state => ({
        ...state,
        recycleBinMode: true,
        recycleBinOriginalRatios: [...state.columnRatios] as [number, number, number],
        columnRatios: [1, 2, 2],
      }));
    },

    // Exit recycle bin mode and restore ratios
    recycleBinExit() {
      update(state => ({
        ...state,
        recycleBinMode: false,
        columnRatios: [...state.recycleBinOriginalRatios] as [number, number, number],
      }));
    },

    // Set archive browsing state (enter archive)
    setArchiveState(state: ArchiveState) {
      update(s => ({ ...s, archiveState: state }));
    },

    // Update internal path within archive
    setArchiveInternalPath(internalPath: string) {
      update(s => {
        if (!s.archiveState) return s;
        return { ...s, archiveState: { ...s.archiveState, internalPath } };
      });
    },

    // Clear archive state (exit archive)
    clearArchiveState() {
      update(s => ({ ...s, archiveState: null }));
    },

    // Set mark (E/y/x mutual exclusion — clears previous mark)
    setMark(type: MarkType, paths: string[]) {
      update(s => ({ ...s, markType: type, markPaths: paths }));
    },

    // Clear mark
    clearMark() {
      update(s => ({ ...s, markType: null, markPaths: [] }));
    },

    // Reset to initial state
    reset() {
      set(initialState);
    },
  };
}

export const layout = createLayoutStore();

// Derived stores for convenience
export const columnWidths = derived(layout, ($layout) => {
  const total = $layout.columnRatios[0] + $layout.columnRatios[1] + $layout.columnRatios[2];
  return {
    parent: ($layout.columnRatios[0] / total) * 100,
    current: ($layout.columnRatios[1] / total) * 100,
    preview: ($layout.columnRatios[2] / total) * 100,
  };
});

export const isEditing = derived(layout, ($layout) => $layout.previewMode !== 'global-normal');
export const isFullscreenEditor = derived(layout, ($layout) => $layout.fullscreenEditorOpen);
export const isTerminalVisible = derived(layout, ($layout) => $layout.terminalVisible);
