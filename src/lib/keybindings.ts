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
      { key: 'Ctrl+Shift+E', context: 'current directory', description: 'Toggle project tree mode' },
      { key: 'Ctrl+=/-/0', context: 'global', description: 'Zoom in/out/reset' },
      { key: 'Ctrl+W h/l', context: 'global', description: 'Switch panel left/right' },
      { key: 'Ctrl+W j/k', context: 'global', description: 'Focus terminal/preview' },
      { key: 'Ctrl+W m', context: 'global', description: 'Toggle 2-column/3-column layout' },
      { key: 'F1', context: 'global', description: 'Show this help' },
    ],
  },
  {
    title: 'Directory Panel',
    items: [
      { key: 'j/k', context: 'directory', description: 'Navigate up/down' },
      { key: 'gg / G', context: 'directory', description: 'Jump to top/bottom' },
      { key: 'Enter / l', context: 'directory', description: 'Open file (expanded preview)' },
      { key: 'R', context: 'directory', description: 'Refresh' },
      { key: 'h', context: 'directory', description: 'Go to parent directory' },
      { key: '/', context: 'directory', description: 'Search in current dir' },
      { key: 'g/', context: 'directory', description: 'Search recursively' },
      { key: 'Space', context: 'directory', description: 'Toggle file selection' },
      { key: 'v', context: 'directory', description: 'Select all / Deselect all' },
      { key: 'y', context: 'directory', description: 'Yank selected files (copy)' },
      { key: 'x', context: 'directory', description: 'Cut selected files (move)' },
      { key: 'p', context: 'directory', description: 'Paste from clipboard' },
      { key: 'P', context: 'directory', description: 'Force paste (overwrite all)' },
      { key: 'd', context: 'directory', description: 'Move to trash' },
      { key: 'D', context: 'directory', description: 'Permanent delete' },
      { key: 'r', context: 'directory', description: 'Rename / Batch rename' },
      { key: 'a', context: 'directory', description: 'Create new file' },
      { key: 'a/', context: 'directory', description: 'Create new directory' },
      { key: '.', context: 'directory', description: 'Toggle hidden files' },
      { key: 'i', context: 'directory', description: 'File info' },
      { key: 'o', context: 'directory', description: 'Open with default program' },
      { key: 'O', context: 'directory', description: 'Open with (choose program)' },
      { key: 'f', context: 'directory', description: 'Filter files by pattern' },
      { key: 'sn/sN', context: 'directory', description: 'Sort by name / reversed' },
      { key: 'ss/sS', context: 'directory', description: 'Sort by size / reversed' },
      { key: 'se/sE', context: 'directory', description: 'Sort by extension / reversed' },
      { key: 'sm/sM', context: 'directory', description: 'Sort by modified time / reversed' },
      { key: 'sc/sC', context: 'directory', description: 'Sort by created time / reversed' },
      { key: 'st', context: 'directory', description: 'Toggle directory first' },
      { key: 'c', context: 'directory', description: 'Compress selected files to .zip' },
      { key: 'C', context: 'directory', description: 'Mark files for compression (then p to compress)' },
      { key: 'e', context: 'directory (on archive)', description: 'Extract archive to current directory' },
      { key: 'E', context: 'directory (on archive)', description: 'Mark archive for extraction (then p to extract)' },
      { key: 'l', context: 'directory (on archive)', description: 'Enter archive as virtual directory' },
      { key: 'l / L', context: 'project tree', description: 'Expand current / recursively expand subtree' },
      { key: 'h / H', context: 'project tree', description: 'Collapse current / deepest expanded level' },
      { key: 'K', context: 'project tree', description: 'Select parent directory' },
    ],
  },
  {
    title: 'Archive Browsing',
    items: [
      { key: 'j/k', context: 'archive', description: 'Navigate up/down' },
      { key: 'l', context: 'archive', description: 'Enter directory / Preview file' },
      { key: 'h', context: 'archive', description: 'Go up (exit archive at root)' },
      { key: 'x', context: 'archive', description: 'Extract selected files to parent dir' },
      { key: 'd', context: 'archive (ZIP only)', description: 'Delete entry from archive' },
      { key: 'r', context: 'archive (ZIP only)', description: 'Rename entry in archive' },
      { key: 'R', context: 'archive', description: 'Refresh archive listing' },
    ],
  },
  {
    title: 'Expanded Preview',
    items: [
      { key: 'Ctrl+W m', context: 'preview', description: 'Toggle 2-column/3-column layout' },
      { key: 'Ctrl+W l', context: 'preview', description: 'Focus TOC sidebar (markdown)' },
      { key: 'Ctrl+W h', context: 'toc', description: 'Back to preview content' },
    ],
  },
  {
    title: 'TOC Sidebar',
    items: [
      { key: 'j / k', context: 'toc', description: 'Navigate up/down' },
      { key: 'gg / G', context: 'toc', description: 'Jump to first/last' },
      { key: 'Enter', context: 'toc', description: 'Jump to heading' },
      { key: 'h / l', context: 'toc', description: 'Collapse/expand current' },
      { key: 'H / L', context: 'toc', description: 'Collapse/expand all' },
      { key: '/', context: 'toc', description: 'Search headings' },
      { key: 'Ctrl+W h', context: 'toc', description: 'Back to preview content' },
    ],
  },
  {
    title: 'Tab (Alt shortcuts)',
    items: [
      { key: 'Alt+N', context: 'global / terminal', description: 'New tab' },
      { key: 'Alt+U', context: 'global / terminal', description: 'Close tab' },
      { key: 'Alt+R', context: 'global / terminal', description: 'Show tab rename hint' },
      { key: 'Alt+M', context: 'global / terminal', description: 'Switch tabs (most recently used)' },
      { key: 'Alt+H / Alt+L', context: 'global / terminal', description: 'Previous / next tab (display order)' },
      { key: 'Alt+,', context: 'global / terminal', description: 'Swap tab backward' },
      { key: 'Alt+.', context: 'global / terminal', description: 'Swap tab forward' },
      { key: 'Alt+D', context: 'global / terminal', description: 'Toggle left panel detach' },
      { key: 'Alt+1-9', context: 'global / terminal', description: 'Switch to tab N' },
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
    title: 'Recycle Bin',
    items: [
      { key: 'g r', context: 'global', description: 'Toggle recycle bin view' },
      { key: 'j/k', context: 'recycle bin', description: 'Navigate up/down' },
      { key: 'gg / G', context: 'recycle bin', description: 'Jump to top/bottom' },
      { key: 'Enter', context: 'recycle bin', description: 'Preview file' },
      { key: 'Space', context: 'recycle bin', description: 'Toggle multi-select' },
      { key: 'r', context: 'recycle bin', description: 'Restore selected items' },
      { key: 'd', context: 'recycle bin', description: 'Permanently delete selected items' },
      { key: 'g d', context: 'recycle bin', description: 'Empty recycle bin' },
      { key: 'R', context: 'recycle bin', description: 'Refresh list' },
      { key: 'i', context: 'recycle bin', description: 'File info' },
      { key: 'h', context: 'recycle bin', description: 'Exit recycle bin view' },
    ],
  },
  {
    title: 'Commands',
    items: [
      { key: 'ratio X:Y:Z', context: 'global', description: 'Set column ratios' },
      { key: 'cd <path>', context: 'global', description: 'Change directory' },
      { key: 'e <path>', context: 'global', description: 'Open file/directory' },
      { key: 'help', context: 'global', description: 'Show keybindings' },
      { key: 'clip', context: 'global', description: 'Show clipboard contents' },
      { key: 'clear', context: 'global', description: 'Clear clipboard' },
    ],
  },
];
