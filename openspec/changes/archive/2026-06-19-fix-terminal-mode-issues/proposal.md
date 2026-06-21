## Why

Terminal mode system has two bugs that break the vim-like workflow:

1. **Startup mode mismatch**: `TerminalPanel` initializes with `mode = 'insert'` and `startShell()` calls `terminal.focus()`, stealing focus to xterm.js. But `activePanel` is `'file-tree'`, so the status bar shows 'NORMAL'. Since xterm has focus in insert mode, the user can type but Ctrl+W panel switching doesn't work (xterm captures keyboard events before they reach the app-layout handler). Only after pressing ESC (which triggers the capturing-phase handler to switch to normal mode and blur xterm) does panel switching work.

2. **Auto-insert on panel switch**: `switchPanel()` in `PanelLayout.svelte` always calls `terminalPanel.setMode('insert')` when entering the terminal panel via Ctrl+W j. This forces insert mode every time, breaking the expected vim-like behavior where the terminal should stay in normal mode until the user explicitly presses 'i'.

## What Changes

- Change terminal initial mode from `'insert'` to `'normal'` so the terminal starts in a state where panel switching works immediately
- Remove unconditional `terminal.focus()` from `startShell()` to prevent xterm from stealing focus at startup
- Remove auto-insert behavior from `switchPanel()` — entering terminal via Ctrl+W j should keep normal mode
- Remove auto-insert behavior from the terminal panel click handler — clicking should focus the panel but not change mode

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `terminal-mode`: Fix terminal mode initialization and panel-switch behavior to respect vim-like normal/insert mode contract

## Impact

- `src/lib/components/TerminalPanel.svelte` — initial mode, `startShell()` focus behavior
- `src/lib/components/PanelLayout.svelte` — `switchPanel()` and terminal click handler
