## Context

The Wind terminal uses a vim-like modal system with Normal and Insert modes. Currently there are two bugs:

1. `TerminalPanel` initializes with `mode = 'insert'` and `startShell()` calls `terminal.focus()`, which puts keyboard focus on the xterm canvas. Since xterm captures all keyboard input in insert mode, Ctrl+W panel switching is blocked. The status bar shows 'NORMAL' (because `activePanel` is `'file-tree'`), creating a confusing state where the UI says NORMAL but the terminal behaves as INSERT.

2. `switchPanel()` always forces insert mode when entering the terminal, overriding any previous mode state.

## Goals / Non-Goals

**Goals:**
- Terminal starts in normal mode, consistent with the status bar display
- Panel switching (Ctrl+W h/j/k/l) works immediately on startup without pressing ESC first
- Entering the terminal panel (via keyboard or mouse) keeps normal mode; user explicitly presses 'i' to enter insert
- Mode transitions are predictable: normal → 'i' → insert → ESC → normal

**Non-Goals:**
- Changing the editor panel's mode system (it has its own independent isEditing state)
- Adding a global mode store (keeping mode local to TerminalPanel is fine for now)
- Adding timeout to the Ctrl+W chord (existing behavior is acceptable)

## Decisions

### 1. Initialize terminal mode as `'normal'`

Change `let mode = $state('insert')` to `let mode = $state('normal')` and set `disableStdin: true` in the Terminal constructor.

**Why**: The terminal should start in a state consistent with the UI and where panel switching works. Normal mode is the "idle" state for a vim-like terminal.

**Alternative considered**: Keep insert mode but add special handling for Ctrl+W in xterm's onData handler. Rejected because it adds complexity and couples xterm input handling to panel navigation.

### 2. Remove unconditional `terminal.focus()` from `startShell()`

The shell still spawns and receives output, but the xterm canvas doesn't steal keyboard focus on startup.

**Why**: Focus should be managed by the panel system (`PanelLayout`), not by the terminal initialization code. The file tree panel should retain focus on startup since `activePanel = 'file-tree'`.

### 3. Remove auto-insert from `switchPanel()` and click handler

When entering the terminal panel via Ctrl+W j or clicking, the terminal stays in its current mode (normal). The user presses 'i' to enter insert mode.

**Why**: This matches vim semantics — switching windows doesn't change mode. The previous auto-insert behavior was a convenience that broke the modal contract.

### 4. Keep `setMode('normal')` when leaving terminal in `switchPanel()`

When switching away from terminal, force normal mode to disable stdin and blur xterm. This prevents the terminal from consuming keyboard input when it's not the active panel.

**Why**: Without this, xterm would still capture keyboard events even when the user is in another panel.

## Risks / Trade-offs

- **[Risk] Users who expect terminal to accept input immediately after Ctrl+W j** → Mitigation: This is a vim-driven workstation; normal mode first is the expected behavior. Users press 'i' to type.
- **[Risk] Shell might output prompts before user can interact** → No issue: output still displays in xterm regardless of mode. Only stdin (typing) is disabled in normal mode.
