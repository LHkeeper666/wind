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
  activeColumn: 'parent' | 'current' | 'preview';

  // Preview/Editor mode
  previewMode: 'global-normal' | 'editor-normal' | 'editor-insert';

  // Fullscreen editor state
  fullscreenEditorOpen: boolean;

  // Fullscreen image viewer state
  fullscreenImageViewerOpen: boolean;

  // Terminal state
  terminalVisible: boolean;
}

const initialState: LayoutState = {
  columnRatios: [1, 2, 2],
  parentPath: '',
  currentPath: '',
  selectedFile: null,
  activeColumn: 'current',
  previewMode: 'global-normal',
  fullscreenEditorOpen: false,
  fullscreenImageViewerOpen: false,
  terminalVisible: false,
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
    setActiveColumn(column: 'parent' | 'current' | 'preview') {
      update(state => ({ ...state, activeColumn: column }));
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

    // Toggle terminal visibility
    toggleTerminal() {
      update(state => ({ ...state, terminalVisible: !state.terminalVisible }));
    },

    // Show terminal
    showTerminal() {
      update(state => ({ ...state, terminalVisible: true }));
    },

    // Hide terminal
    hideTerminal() {
      update(state => ({ ...state, terminalVisible: false }));
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
