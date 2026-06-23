import { writable, derived, get } from 'svelte/store';
import { layout } from './layout';

export interface TabState {
  id: number;
  name: string;
  parentPath: string;
  currentPath: string;
  selectedFile: string | null;
  cursorIndex: number;
  scrollOffset: number;
  terminalVisible: boolean;
  terminalMode: 'insert' | 'normal' | null;
  terminalHeight: number;
  shellType: string;
}

interface TabsState {
  tabs: TabState[];
  activeTabId: number;
  nextId: number;
}

function getDefaultTab(id: number): TabState {
  return {
    id,
    name: 'home',
    parentPath: '',
    currentPath: '',
    selectedFile: null,
    cursorIndex: 0,
    scrollOffset: 0,
    terminalVisible: false,
    terminalMode: null,
    terminalHeight: 300,
    shellType: 'git-bash',
  };
}

function getDirName(path: string): string {
  if (!path || path === '/' || path === '\\') return 'root';
  const normalized = path.replace(/\//g, '\\');
  const parts = normalized.split('\\').filter(Boolean);
  if (parts.length === 1 && /^[A-Za-z]:$/.test(parts[0])) return parts[0];
  return parts[parts.length - 1] || 'home';
}

function createTabsStore() {
  const initialTab = getDefaultTab(1);
  const { subscribe, set, update } = writable<TabsState>({
    tabs: [initialTab],
    activeTabId: 1,
    nextId: 2,
  });

  return {
    subscribe,

    createTab(inheritPath?: string) {
      update(state => {
        const newTab = getDefaultTab(state.nextId);
        if (inheritPath) {
          newTab.currentPath = inheritPath;
          newTab.name = getDirName(inheritPath);
          const normalized = inheritPath.replace(/\//g, '\\');
          if (/^[A-Za-z]:\\$/.test(normalized) || normalized === '\\') {
            newTab.parentPath = '\\';
          } else {
            const lastSlash = normalized.lastIndexOf('\\');
            newTab.parentPath = lastSlash > 0 ? normalized.substring(0, lastSlash) : '\\';
          }
        } else {
          // Inherit from current active tab
          const currentTab = state.tabs.find(t => t.id === state.activeTabId);
          if (currentTab) {
            newTab.currentPath = currentTab.currentPath;
            newTab.parentPath = currentTab.parentPath;
            newTab.name = getDirName(currentTab.currentPath);
          }
        }
        return {
          ...state,
          tabs: [...state.tabs, newTab],
          activeTabId: newTab.id,
          nextId: state.nextId + 1,
        };
      });
    },

    closeTab(tabId: number) {
      update(state => {
        if (state.tabs.length <= 1) return state;
        const idx = state.tabs.findIndex(t => t.id === tabId);
        if (idx === -1) return state;
        const newTabs = state.tabs.filter(t => t.id !== tabId);
        let newActiveId = state.activeTabId;
        if (state.activeTabId === tabId) {
          // Switch to nearest tab
          const newIdx = Math.min(idx, newTabs.length - 1);
          newActiveId = newTabs[newIdx].id;
        }
        return { ...state, tabs: newTabs, activeTabId: newActiveId };
      });
    },

    switchTab(tabId: number) {
      update(state => {
        if (!state.tabs.find(t => t.id === tabId)) return state;
        if (state.activeTabId === tabId) return state;
        return { ...state, activeTabId: tabId };
      });
    },

    switchTabRelative(delta: number) {
      update(state => {
        const idx = state.tabs.findIndex(t => t.id === state.activeTabId);
        if (idx === -1) return state;
        const newIdx = (idx + delta + state.tabs.length) % state.tabs.length;
        return { ...state, activeTabId: state.tabs[newIdx].id };
      });
    },

    switchTabByIndex(index: number) {
      update(state => {
        if (index < 0 || index >= state.tabs.length) return state;
        return { ...state, activeTabId: state.tabs[index].id };
      });
    },

    renameTab(tabId: number, name: string) {
      update(state => ({
        ...state,
        tabs: state.tabs.map(t => t.id === tabId ? { ...t, name } : t),
      }));
    },

    swapTab(delta: number) {
      update(state => {
        const idx = state.tabs.findIndex(t => t.id === state.activeTabId);
        if (idx === -1) return state;
        const targetIdx = idx + delta;
        if (targetIdx < 0 || targetIdx >= state.tabs.length) return state;
        const newTabs = [...state.tabs];
        [newTabs[idx], newTabs[targetIdx]] = [newTabs[targetIdx], newTabs[idx]];
        return { ...state, tabs: newTabs };
      });
    },

    saveActiveTabState() {
      update(state => {
        const idx = state.tabs.findIndex(t => t.id === state.activeTabId);
        if (idx === -1) return state;
        const layoutState = get(layout);
        const newTabs = [...state.tabs];
        newTabs[idx] = {
          ...newTabs[idx],
          parentPath: layoutState.parentPath,
          currentPath: layoutState.currentPath,
          selectedFile: layoutState.selectedFile,
          terminalVisible: layoutState.terminalVisible,
          terminalMode: layoutState.terminalMode,
          terminalHeight: layoutState.terminalHeight,
        };
        // Update tab name from current path
        if (layoutState.currentPath) {
          newTabs[idx].name = getDirName(layoutState.currentPath);
        }
        return { ...state, tabs: newTabs };
      });
    },

    restoreActiveTabState() {
      const state = get({ subscribe });
      const tab = state.tabs.find(t => t.id === state.activeTabId);
      if (!tab) return;
      if (tab.currentPath) {
        layout.setCurrentPath(tab.currentPath);
      }
      layout.setActiveColumn('current');
    },
  };
}

export const tabs = createTabsStore();

export const activeTab = derived(tabs, ($tabs) => $tabs.tabs.find(t => t.id === $tabs.activeTabId) || $tabs.tabs[0]);
export const activeTabIndex = derived(tabs, ($tabs) => $tabs.tabs.findIndex(t => t.id === $tabs.activeTabId));
export const tabCount = derived(tabs, ($tabs) => $tabs.tabs.length);
