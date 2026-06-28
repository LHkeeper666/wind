export interface Keybinding {
  key: string;
  context: string;
  description: string;
}

export interface KeybindingGroup {
  title: string;
  items: Keybinding[];
}

export const keybindingGroups: KeybindingGroup[] = [
  {
    title: 'Global',
    items: [
      { key: ':', context: 'global', description: 'Open command palette' },
      { key: 'Ctrl+P', context: 'global', description: 'File search' },
      { key: 'Ctrl+`', context: 'global', description: 'Toggle terminal' },
      { key: 'Ctrl+Shift+`', context: 'global', description: 'Fullscreen terminal' },
      { key: 'Ctrl+L', context: 'global', description: 'Restore focus' },
      { key: 'Ctrl+=/-/0', context: 'global', description: 'Zoom in/out/reset' },
      { key: 'Ctrl+W h/l', context: 'global', description: 'Switch panel left/right' },
      { key: 'Ctrl+W j/k', context: 'global', description: 'Focus terminal/preview' },
      { key: 'F1', context: 'global', description: 'Show this help' },
    ],
  },
  {
    title: 'Directory Panel',
    items: [
      { key: 'j/k', context: 'directory', description: 'Navigate up/down' },
      { key: 'gg / G', context: 'directory', description: 'Jump to top/bottom' },
      { key: 'Enter', context: 'directory', description: 'Open file or directory' },
      { key: 'R', context: 'directory', description: 'Refresh' },
      { key: 'h', context: 'directory', description: 'Go to parent directory' },
      { key: 'l', context: 'directory', description: 'Open selected item' },
      { key: '/', context: 'directory', description: 'Search in current dir' },
      { key: 'g/', context: 'directory', description: 'Search recursively' },
    ],
  },
  {
    title: 'Tab (t prefix)',
    items: [
      { key: 't t', context: 'directory', description: 'New tab' },
      { key: 't c', context: 'directory', description: 'Close tab' },
      { key: 't r', context: 'directory', description: 'Rename tab' },
      { key: 't n / t ]', context: 'directory', description: 'Next tab' },
      { key: 't p / t [', context: 'directory', description: 'Previous tab' },
      { key: 't ,', context: 'directory', description: 'Swap tab backward' },
      { key: 't .', context: 'directory', description: 'Swap tab forward' },
      { key: 't 1-9', context: 'directory', description: 'Switch to tab N' },
    ],
  },
  {
    title: 'Terminal',
    items: [
      { key: 'Escape', context: 'terminal', description: 'Switch to normal mode' },
      { key: 'i', context: 'terminal', description: 'Switch to insert mode' },
    ],
  },
  {
    title: 'Preview / Editor',
    items: [
      { key: 'Escape', context: 'editor', description: 'Exit to normal mode' },
      { key: 'Ctrl+[', context: 'editor', description: 'Exit to normal mode' },
      { key: 'Enter', context: 'editor', description: 'Focus editor (from overlay)' },
    ],
  },
  {
    title: 'Commands',
    items: [
      { key: 'ratio X:Y:Z', context: 'global', description: 'Set column ratios' },
      { key: 'cd <path>', context: 'global', description: 'Change directory' },
      { key: 'e <path>', context: 'global', description: 'Open file/directory' },
      { key: 'help', context: 'global', description: 'Show keybindings' },
    ],
  },
];
