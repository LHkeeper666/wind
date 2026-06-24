import { writable, derived } from 'svelte/store';

export interface LayoutState {
  // Column ratios [parent, current, preview]
  columnRatios: [number, number, number];

  // Current navigation paths
  parentPath: string;
  currentPath: string;

  // Selected file in current directory
  selectedFile: string | null;

  // Active column for focus
  activeColumn: 'parent' | 'current' | 'preview' | 'terminal';

  // Terminal mode (null when terminal is hidden)
  terminalMode: 'insert' | 'normal' | null;

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
}

const initialState: LayoutState = {
  columnRatios: [1, 2, 2],
  parentPath: '',
  currentPath: '',
  selectedFile: null,
  activeColumn: 'current',
  terminalMode: null,
  previewMode: 'global-normal',
  fullscreenEditorOpen: false,
  fullscreenImageViewerOpen: false,
  fullscreenPdfViewerOpen: false,
  fullscreenVideoPlayerOpen: false,
  fullscreenTerminalOpen: false,
  terminalVisible: false,
  terminalHeight: 300,
};

function createLayoutStore() {
  const { subscribe, set, update } = writable<LayoutState>(initialState);

  return {
    subscribe,

    // Set column ratios
    setRatios(ratios: [number, number, number]) {
      update(state => ({ ...state, columnRatios: ratios }));
    },

    // Update current path and auto-update parent path
    setCurrentPath(path: string) {
      update(state => {
        // 规范化路径分隔符
        let normalized = path.replace(/\//g, '\\');
        // 确保驱动器根目录格式为 X:\（不是 X:）
        if (/^[A-Za-z]:$/.test(normalized)) {
          normalized = normalized + '\\';
        }

        let parentPath: string;
        if (normalized === '\\') {
          parentPath = '\\';
        } else if (/^[A-Za-z]:\\$/.test(normalized)) {
          parentPath = '\\';
        } else {
          const lastSlash = normalized.lastIndexOf('\\');
          parentPath = lastSlash > 0 ? normalized.substring(0, lastSlash) : '\\';
        }

        return {
          ...state,
          currentPath: normalized,
          parentPath: parentPath,
          selectedFile: null,
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

    // Set terminal mode
    setTerminalMode(mode: 'insert' | 'normal' | null) {
      update(state => ({ ...state, terminalMode: mode }));
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
      update(state => ({ ...state, fullscreenTerminalOpen: true }));
    },

    // Close fullscreen terminal
    closeFullscreenTerminal() {
      update(state => ({ ...state, fullscreenTerminalOpen: false }));
    },

    // Toggle fullscreen terminal
    toggleFullscreenTerminal() {
      update(state => ({ ...state, fullscreenTerminalOpen: !state.fullscreenTerminalOpen }));
    },

    // Toggle terminal visibility
    toggleTerminal() {
      update(state => ({
        ...state,
        terminalVisible: !state.terminalVisible,
        terminalMode: !state.terminalVisible ? 'insert' : null,
        activeColumn: !state.terminalVisible ? 'terminal' : 'current',
      }));
    },

    // Show terminal
    showTerminal() {
      update(state => ({ ...state, terminalVisible: true, terminalMode: 'insert', activeColumn: 'terminal' }));
    },

    // Hide terminal
    hideTerminal() {
      update(state => ({ ...state, terminalVisible: false, terminalMode: null, activeColumn: 'current' }));
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
