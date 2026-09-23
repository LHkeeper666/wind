import { get } from 'svelte/store';
import { layout } from '$lib/stores/layout';
import { tick } from 'svelte';
import type { TabState } from '$lib/stores/tabs';

// --- Prefix state ---
let waitingForWindowKey: boolean = $state(false);
let windowKeyTimeout: ReturnType<typeof setTimeout> | null = null;
let waitingForGKey: boolean = $state(false);
let gKeyTimeout: ReturnType<typeof setTimeout> | null = null;

// --- Alt+Tab switcher state ---
let altHeld: boolean = $state(false);
let switcherActive: boolean = $state(false);
let switcherSelectionId: number = $state(-1);
let switcherOriginTabId: number = $state(-1);
let switcherMruIds: number[] = $state([]);
let switcherPhysicalIds: number[] = $state([]);

// --- Getters for template binding ---
export function getSwitcherActive() { return switcherActive; }
export function getSwitcherSelectionId() { return switcherSelectionId; }

// --- Dependencies ---
interface KeyboardDeps {
  getPreviewEditor: () => any;
  getFullscreenEditor: () => any;
  getRecycleBinPanel: () => any;
  getCurrentDirectoryPanel: () => any;

  getCommandPaletteVisible: () => boolean;
  setCommandPaletteVisible: (v: boolean) => void;
  getShowFileSearch: () => boolean;
  setShowHelp: (v: boolean) => void;
  toggleShowTransfer: () => void;
  getZoomLevel: () => number;
  applyZoom: (level: number) => void;

  focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void;
  handleTabNew: () => void;
  handleTabClose: () => void;
  handleTabSwitchByIndex: (index: number) => void;
  handlePaste: (force?: boolean) => void;
  showToast: (msg: string) => void;

  // Tab switcher support
  saveCurrentTabState: () => void;
  getTabActiveId: () => number;
  getTabPhysicalIds: () => number[];
  getTabsMruOrder: () => number[];
  restoreTabContent: (tab: TabState) => void;
  getTabById: (id: number) => TabState | undefined;
  commitTabSwitch: (tabId: number) => void;
  swapTab: (delta: number) => void;
  renameActiveTab: (path: string) => void;
  openFileSearch: () => void;
  synchronizeProjectTreeWatcher: () => Promise<void>;
}

let _deps: KeyboardDeps | null = null;

// --- Lifecycle ---

let _boundKeydown: ((e: KeyboardEvent) => void) | null = null;
let _boundKeyup: ((e: KeyboardEvent) => void) | null = null;
let _boundWheel: ((e: WheelEvent) => void) | null = null;

export function initKeyboardDeps(deps: KeyboardDeps) {
  _deps = deps;
}

export function setup() {
  _boundKeydown = handleGlobalKeydown;
  _boundKeyup = handleGlobalKeyup;
  _boundWheel = handleGlobalWheel;
  window.addEventListener('keydown', _boundKeydown, true);
  window.addEventListener('keyup', _boundKeyup, true);
  window.addEventListener('wheel', _boundWheel, { passive: false, capture: true });
}

export function teardown() {
  if (_boundKeydown) { window.removeEventListener('keydown', _boundKeydown, true); _boundKeydown = null; }
  if (_boundKeyup) { window.removeEventListener('keyup', _boundKeyup, true); _boundKeyup = null; }
  if (_boundWheel) { window.removeEventListener('wheel', _boundWheel, { capture: true } as any); _boundWheel = null; }
  if (windowKeyTimeout) { clearTimeout(windowKeyTimeout); windowKeyTimeout = null; }
  if (gKeyTimeout) { clearTimeout(gKeyTimeout); gKeyTimeout = null; }
}

// --- Helpers ---

const supportedAltTabCodes = new Set([
  'KeyN', 'KeyM', 'KeyU', 'KeyR', 'KeyH', 'KeyL', 'Comma', 'Period', 'KeyD',
  'Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5', 'Digit6', 'Digit7', 'Digit8', 'Digit9',
]);

function isSupportedAltTabCode(code: string): boolean {
  return supportedAltTabCodes.has(code);
}

function isTerminalInputTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('.terminal-containers') !== null;
}

// --- Switcher operations ---

function startSwitcher(mode: 'mru' | 'physical', direction: 1 | -1 = 1) {
  const deps = _deps!;
  const tabStore = deps.saveCurrentTabState();
  switcherOriginTabId = deps.getTabActiveId();
  switcherMruIds = deps.getTabsMruOrder();
  switcherPhysicalIds = deps.getTabPhysicalIds();
  switcherSelectionId = deps.getTabActiveId();
  switcherActive = true;
  if (mode === 'physical') {
    moveSwitcherIn(switcherPhysicalIds, direction);
  } else {
    moveSwitcherIn(switcherMruIds, 1);
  }
}

function moveSwitcherIn(ids: number[], direction: 1 | -1) {
  const deps = _deps!;
  const idx = ids.indexOf(switcherSelectionId);
  if (idx === -1) return;
  const newIdx = (idx + direction + ids.length) % ids.length;
  switcherSelectionId = ids[newIdx];
  const tab = deps.getTabById(switcherSelectionId);
  if (tab) deps.restoreTabContent(tab);
}

export function commitSwitcher() {
  if (!switcherActive) return;
  const deps = _deps!;
  const selectionId = switcherSelectionId;
  const originId = switcherOriginTabId;
  if (selectionId !== originId && selectionId >= 0) {
    deps.commitTabSwitch(selectionId);
  }
  switcherActive = false;
  switcherSelectionId = -1;
  switcherOriginTabId = -1;
  switcherMruIds = [];
  switcherPhysicalIds = [];
}

// --- Mode checkers ---

function getPreviewMode(): string {
  return _deps?.getPreviewEditor()?.getMode?.() || 'global-normal';
}

function canOpenCommandPalette(): boolean {
  const l = get(layout);
  const previewMode = getPreviewMode();
  return !l.fullscreenEditorOpen && !l.fullscreenImageViewerOpen
    && !l.fullscreenPdfViewerOpen && !l.fullscreenVideoPlayerOpen
    && !l.fullscreenTerminalOpen
    && (l.activeColumn !== 'preview' || previewMode === 'global-normal');
}

function canUseTabShortcuts(): boolean {
  const l = get(layout);
  const deps = _deps!;
  const previewMode = getPreviewMode();
  return !deps.getCommandPaletteVisible() && !deps.getShowFileSearch()
    && !l.fullscreenEditorOpen && !l.fullscreenImageViewerOpen
    && !l.fullscreenPdfViewerOpen && !l.fullscreenVideoPlayerOpen
    && !(l.activeColumn === 'preview' && previewMode === 'editor-insert');
}

function canUseGlobalFileOperations(): boolean {
  const l = get(layout);
  const deps = _deps!;
  const previewMode = getPreviewMode();
  return !deps.getCommandPaletteVisible() && !deps.getShowFileSearch()
    && !l.fullscreenEditorOpen && !l.fullscreenImageViewerOpen
    && !l.fullscreenPdfViewerOpen && !l.fullscreenVideoPlayerOpen
    && !(l.activeColumn === 'preview' && previewMode !== 'global-normal');
}

// --- Internal helpers ---

function togglePreviewLayout() {
  const l = get(layout);
  if (l.previewExpanded) {
    layout.collapsePreview();
  } else {
    layout.expandPreview();
  }
}

function handleSwitchPanel(direction: 'left' | 'right') {
  const deps = _deps!;
  const l = get(layout);
  const current = l.activeColumn;
  if (direction === 'left') {
    if (current === 'preview') {
      deps.focusPanel('current');
    } else if (current === 'current') {
      deps.focusPanel('parent');
    }
  } else if (direction === 'right') {
    if (current === 'parent') {
      deps.focusPanel('current');
    } else if (current === 'current') {
      deps.focusPanel('preview');
    } else if (current === 'preview') {
      if (l.previewExpanded && deps.getPreviewEditor()?.isTocVisible() && !deps.getPreviewEditor()?.isTocFocused()) {
        deps.getPreviewEditor().focusToc();
      }
    }
  }
}

// --- Main keydown handler ---

async function handleGlobalKeydown(event: KeyboardEvent) {
  if (!_deps) return;
  const deps = _deps;

  if ((event.code === 'AltLeft' || event.code === 'AltRight') && !event.repeat) {
    altHeld = true;
  }

  const target = event.target as HTMLElement | null;
  const isTerminalAltTabChord = event.altKey && !event.ctrlKey && !event.metaKey
    && !event.shiftKey && isSupportedAltTabCode(event.code) && isTerminalInputTarget(event.target);
  const l0 = get(layout);
  const isPreviewCodeMirrorTab = event.key === 'Tab'
    && l0.activeColumn === 'preview'
    && !l0.fullscreenEditorOpen
    && target instanceof Element
    && target.closest('.preview-panel .cm-editor') !== null;
  const isPdfPreview = l0.activeColumn === 'preview'
    && (deps.getPreviewEditor()?.getFile?.() || '').toLowerCase().endsWith('.pdf');
  if (isPdfPreview && event.ctrlKey
    && (event.key === '=' || event.key === '+' || event.key === '-')) {
    return;
  }
  if ((target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable)
    && !isTerminalAltTabChord
    && !isPreviewCodeMirrorTab
    && !waitingForWindowKey
    && !waitingForGKey) {
    if (event.key !== 'Escape' && !event.ctrlKey) return;
  }

  if (target instanceof Element && target.closest('.pdf-preview-panel')) {
    if (!event.ctrlKey && !event.altKey && !event.metaKey) return;
    if (event.ctrlKey && (event.key === '=' || event.key === '+' || event.key === '-')) return;
  }

  // Tab / Shift+Tab: prevent native focus switching
  if (event.key === 'Tab' && !deps.getCommandPaletteVisible()) {
    event.preventDefault();
    const l = get(layout);
    const isPreviewEditorTab = l.activeColumn === 'preview'
      && !l.fullscreenEditorOpen
      && target instanceof Element
      && target.closest('.preview-panel .cm-editor') !== null;
    if (isPreviewEditorTab) {
      if (event.shiftKey) {
        deps.getPreviewEditor()?.pressShiftTab();
      } else {
        deps.getPreviewEditor()?.pressTab();
      }
      return;
    }
    if (l.fullscreenEditorOpen) {
      if (event.shiftKey) {
        deps.getFullscreenEditor()?.pressShiftTab();
      } else {
        deps.getFullscreenEditor()?.pressTab();
      }
      return;
    }
  }

  // Ctrl+= / Ctrl+- / Ctrl+0 for zoom
  if (event.ctrlKey && (event.key === '=' || event.key === '+')) {
    event.preventDefault();
    deps.applyZoom(deps.getZoomLevel() + 0.1);
    return;
  }
  if (event.ctrlKey && event.key === '-') {
    event.preventDefault();
    deps.applyZoom(deps.getZoomLevel() - 0.1);
    return;
  }
  if (event.ctrlKey && event.key === '0') {
    event.preventDefault();
    deps.applyZoom(1);
    return;
  }

  // F1 to show help overlay
  if (event.key === 'F1') {
    event.preventDefault();
    deps.setShowHelp(true);
    return;
  }

  // Ctrl+T to toggle Transfer Manager
  if (event.ctrlKey && !event.altKey && event.key === 't' && !waitingForWindowKey) {
    const l = get(layout);
    const canToggle = !deps.getCommandPaletteVisible() && !deps.getShowFileSearch()
      && !l.fullscreenEditorOpen && !l.fullscreenImageViewerOpen
      && !l.fullscreenTerminalOpen;
    if (canToggle) {
      event.preventDefault();
      deps.toggleShowTransfer();
      return;
    }
  }

  // Ctrl+Shift+E to toggle project tree mode
  if (event.ctrlKey && event.shiftKey && event.code === 'KeyE') {
    const l = get(layout);
    if (l.activeColumn === 'current' && !deps.getCommandPaletteVisible() && !deps.getShowFileSearch()) {
      event.preventDefault();
      event.stopPropagation();
      const panel = deps.getCurrentDirectoryPanel();
      const state = panel?.getProjectTreeState();
      await panel?.setProjectMode(!state?.enabled);
      const tree = panel?.getProjectTreeState();
      if (tree?.enabled && tree.rootPath) {
        deps.renameActiveTab(tree.rootPath);
      }
      await deps.synchronizeProjectTreeWatcher();
      return;
    }
  }

  // Ctrl+L to manually restore focus
  if (event.ctrlKey && event.key === 'l' && !waitingForWindowKey) {
    const l = get(layout);
    const canRestore = !deps.getCommandPaletteVisible() && !deps.getShowFileSearch()
      && !l.fullscreenEditorOpen && !l.fullscreenImageViewerOpen
      && !l.fullscreenPdfViewerOpen && !l.fullscreenVideoPlayerOpen;
    if (canRestore) {
      event.preventDefault();
      deps.focusPanel(l.activeColumn);
      deps.showToast(`Focus: ${l.activeColumn.toUpperCase()}`);
    }
    return;
  }

  // Ctrl+Shift+` to toggle fullscreen terminal
  if (event.ctrlKey && event.shiftKey && (event.key === '`' || event.key === '~')) {
    event.preventDefault();
    const l = get(layout);
    if (l.fullscreenTerminalOpen) {
      layout.closeFullscreenTerminal();
    } else if (l.terminalVisible) {
      layout.openFullscreenTerminal();
    } else {
      layout.showTerminal();
      layout.openFullscreenTerminal();
      deps.focusPanel('terminal');
    }
    return;
  }

  // Ctrl+` to toggle terminal
  if (event.ctrlKey && event.key === '`' && !event.shiftKey) {
    event.preventDefault();
    const l = get(layout);
    if (l.fullscreenTerminalOpen) {
      layout.closeFullscreenTerminal();
      layout.hideTerminal();
      deps.focusPanel(l.activeColumn);
    } else if (l.terminalVisible) {
      layout.hideTerminal();
      deps.focusPanel(l.activeColumn);
    } else {
      layout.showTerminal();
      deps.focusPanel('terminal');
    }
    return;
  }

  // g prefix for recycle bin (gr)
  const l1 = get(layout);
  if (event.key === 'g' && !event.ctrlKey && !event.altKey && !event.metaKey
    && !waitingForWindowKey && !l1.fullscreenEditorOpen
    && !l1.fullscreenImageViewerOpen && !l1.fullscreenPdfViewerOpen
    && !l1.fullscreenVideoPlayerOpen) {
    waitingForGKey = true;
    layout.setKeyPrefix('g');
    if (gKeyTimeout) clearTimeout(gKeyTimeout);
    gKeyTimeout = setTimeout(() => { waitingForGKey = false; layout.clearKeyPrefix(); }, 500);
  }

  if (waitingForGKey && event.key === 'r' && !event.ctrlKey && !event.altKey && !event.metaKey) {
    waitingForGKey = false;
    layout.clearKeyPrefix();
    if (gKeyTimeout) { clearTimeout(gKeyTimeout); gKeyTimeout = null; }
    event.preventDefault();
    event.stopPropagation();
    const l = get(layout);
    if (!l.recycleBinMode) {
      layout.recycleBinEnter();
      deps.focusPanel('current');
      tick().then(() => deps.getRecycleBinPanel()?.reload());
      deps.showToast('Recycle Bin');
    } else {
      layout.recycleBinExit();
      deps.focusPanel('current');
      deps.showToast('Exited Recycle Bin');
    }
    return;
  }

  if (waitingForGKey && event.key !== 'g' && event.key !== 'r') {
    waitingForGKey = false;
    layout.clearKeyPrefix();
    if (gKeyTimeout) { clearTimeout(gKeyTimeout); gKeyTimeout = null; }
  }

  // Ctrl+W prefix for vim-style window navigation
  const l2 = get(layout);
  if (event.ctrlKey && event.key === 'w' && !l2.fullscreenTerminalOpen) {
    event.preventDefault();
    if (l2.activeColumn === 'terminal') {
      event.stopPropagation();
    }
    waitingForWindowKey = true;
    layout.setKeyPrefix('^W');
    if (windowKeyTimeout) clearTimeout(windowKeyTimeout);
    windowKeyTimeout = setTimeout(() => { waitingForWindowKey = false; layout.clearKeyPrefix(); }, 1000);
    return;
  }

  if (waitingForWindowKey) {
    waitingForWindowKey = false;
    layout.clearKeyPrefix();
    if (windowKeyTimeout) { clearTimeout(windowKeyTimeout); windowKeyTimeout = null; }

    const code = event.code;
    if (deps.getPreviewEditor()?.isTocFocused?.() && code === 'KeyH') {
      event.preventDefault();
      event.stopPropagation();
      deps.getPreviewEditor().focusContent();
      return;
    }
    if (code === 'KeyH') {
      event.preventDefault();
      event.stopPropagation();
      handleSwitchPanel('left');
      return;
    } else if (code === 'KeyL') {
      event.preventDefault();
      event.stopPropagation();
      handleSwitchPanel('right');
      return;
    } else if (code === 'KeyJ') {
      event.preventDefault();
      event.stopPropagation();
      const l = get(layout);
      if (l.terminalVisible && l.activeColumn !== 'terminal') {
        layout.setPreTerminalColumn(l.activeColumn as 'parent' | 'current' | 'preview');
        deps.focusPanel('terminal');
      }
      return;
    } else if (code === 'KeyK') {
      event.preventDefault();
      event.stopPropagation();
      const l = get(layout);
      if (l.activeColumn === 'terminal') {
        deps.focusPanel(l.preTerminalColumn === 'terminal' ? 'current' : l.preTerminalColumn);
      }
      return;
    } else if (code === 'KeyM') {
      event.preventDefault();
      event.stopPropagation();
      togglePreviewLayout();
      return;
    }
  }

  // Alt+Tab switcher
  if (switcherActive && altHeld) {
    event.preventDefault();
    event.stopPropagation();
    if (event.code === 'KeyM') {
      moveSwitcherIn(switcherMruIds, 1);
    } else if (event.code === 'KeyL') {
      moveSwitcherIn(switcherPhysicalIds, 1);
    } else if (event.code === 'KeyH') {
      moveSwitcherIn(switcherPhysicalIds, -1);
    }
    return;
  }

  const isAltTabShortcut = event.altKey && !event.ctrlKey && !event.metaKey
    && !event.shiftKey && canUseTabShortcuts() && isSupportedAltTabCode(event.code);
  if (isAltTabShortcut) {
    if (event.repeat) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    let handled = true;
    switch (event.code) {
      case 'KeyN':
        deps.handleTabNew();
        break;
      case 'KeyU':
        deps.handleTabClose();
        break;
      case 'KeyR':
        deps.showToast('Double-click tab name to rename');
        break;
      case 'KeyM':
        startSwitcher('mru');
        break;
      case 'KeyL':
        startSwitcher('physical');
        break;
      case 'KeyH':
        startSwitcher('physical', -1);
        break;
      case 'Comma':
        deps.swapTab(-1);
        break;
      case 'Period':
        deps.swapTab(1);
        break;
      case 'KeyD':
        layout.toggleDetach();
        deps.showToast(get(layout).leftMode === 'manual' ? 'Panel detached' : 'Panel attached');
        break;
      default:
        if (/^Digit[1-9]$/.test(event.code)) {
          deps.handleTabSwitchByIndex(parseInt(event.code.substring(5)) - 1);
        } else {
          handled = false;
        }
    }

    if (!handled) return;
    event.preventDefault();
    event.stopPropagation();
    return;
  }

  // P key for force paste
  if (event.key === 'P' && event.shiftKey && !event.ctrlKey && !event.altKey && canUseGlobalFileOperations()) {
    event.preventDefault();
    deps.handlePaste(true);
    return;
  }

  // p key for paste
  if (event.code === 'KeyP' && !event.ctrlKey && !event.altKey && !event.shiftKey && canUseGlobalFileOperations()) {
    event.preventDefault();
    deps.handlePaste();
    return;
  }

  // : to open command palette
  if (event.key === ':' && !deps.getCommandPaletteVisible() && !deps.getShowFileSearch() && canOpenCommandPalette()) {
    event.preventDefault();
    deps.setCommandPaletteVisible(true);
  }

  // Ctrl+P for file search
  if (event.ctrlKey && event.key === 'p') {
    event.preventDefault();
    deps.openFileSearch();
  }
}

// --- Keyup handler ---

function handleGlobalKeyup(event: KeyboardEvent) {
  if (event.code === 'AltLeft' || event.code === 'AltRight') {
    altHeld = event.getModifierState('Alt');
    if (!altHeld && switcherActive) commitSwitcher();
  }
}

// --- Wheel handler ---

function handleGlobalWheel(event: WheelEvent) {
  if (!_deps) return;
  if (!event.ctrlKey) return;
  const target = event.target as HTMLElement | null;
  if (target instanceof Element && target.closest('.pdf-preview-panel')) return;
  event.preventDefault();
  _deps.applyZoom(_deps.getZoomLevel() + (event.deltaY < 0 ? 0.1 : -0.1));
}

// --- Exports for PanelLayout ---

export function notifyWindowFocusLost() {
  if (switcherActive) commitSwitcher();
  altHeld = false;
}
