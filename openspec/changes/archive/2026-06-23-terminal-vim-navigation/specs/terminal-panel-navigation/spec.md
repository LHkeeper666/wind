## ADDED Requirements

### Requirement: Ctrl+W prefix for panel switching
The system SHALL use `Ctrl+W` followed by `h/j/k/l` as the panel switching shortcut, replacing bare `h/l`.

#### Scenario: Switch left with Ctrl+W h
- **WHEN** user presses `Ctrl+W` then `h` while any panel is focused
- **THEN** focus moves to the left panel (preview → current → parent)

#### Scenario: Switch right with Ctrl+W l
- **WHEN** user presses `Ctrl+W` then `l` while any panel is focused
- **THEN** focus moves to the right panel (parent → current → preview)

#### Scenario: Switch to terminal with Ctrl+W j
- **WHEN** user presses `Ctrl+W` then `j` while a panel is focused and terminal is visible
- **THEN** focus moves to the terminal and terminal enters insert mode

#### Scenario: Switch from terminal to current with Ctrl+W k
- **WHEN** user presses `Ctrl+W` then `k` while terminal is in normal mode
- **THEN** focus moves to the current directory panel, terminal remains visible

#### Scenario: Ctrl+W timeout
- **WHEN** user presses `Ctrl+W` but does not press a direction key within 1 second
- **THEN** the prefix state resets and no panel switch occurs

### Requirement: Current panel h navigates to parent directory
In the current directory panel, pressing `h` SHALL navigate to the parent directory (equivalent to cd ..).

#### Scenario: h in current panel
- **WHEN** user presses `h` while the current directory panel is focused
- **THEN** the current directory navigates to its parent directory

#### Scenario: h in parent panel has no effect
- **WHEN** user presses `h` while the parent directory panel is focused
- **THEN** no action occurs (h is not bound in parent panel)

### Requirement: Current panel l enters selected item
In the current directory panel, pressing `l` SHALL enter the selected item (equivalent to pressing Enter).

#### Scenario: l in current panel on directory
- **WHEN** user presses `l` while a directory is selected in the current panel
- **THEN** the directory is entered

#### Scenario: l in current panel on file
- **WHEN** user presses `l` while a file is selected in the current panel
- **THEN** the file is opened (same as Enter behavior)

### Requirement: Status bar shows terminal mode
The status bar SHALL display the terminal mode when the terminal is the active focus target.

#### Scenario: Status bar when terminal is focused in insert mode
- **WHEN** terminal is focused and in insert mode
- **THEN** status bar shows `TERMINAL-INSERT`

#### Scenario: Status bar when terminal is focused in normal mode
- **WHEN** terminal is focused and in normal mode
- **THEN** status bar shows `TERMINAL-NORMAL`

#### Scenario: Status bar when panel is focused
- **WHEN** a panel (parent/current/preview) is focused
- **THEN** status bar shows the panel name in uppercase (existing behavior)

### Requirement: Terminal mode synced to store
The terminal's mode state (insert/normal) SHALL be synchronized to the layout store.

#### Scenario: Terminal enters normal mode
- **WHEN** user presses Escape in terminal insert mode
- **THEN** layout store's `terminalMode` is set to `'normal'`

#### Scenario: Terminal enters insert mode
- **WHEN** user presses `i` in terminal normal mode
- **THEN** layout store's `terminalMode` is set to `'insert'`

#### Scenario: Terminal is hidden
- **WHEN** terminal is toggled off via Ctrl+`
- **THEN** layout store's `terminalMode` is set to `null`
