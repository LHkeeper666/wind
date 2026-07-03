## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: t prefix tab operations work globally
The `t` prefix for tab operations (t n, t p, t c, t t, etc.) SHALL work from any panel focus state, including terminal normal mode, as long as no modal overlay (command palette, file search, fullscreen editor) is open and the active input is not in insert mode.

#### Scenario: t n in terminal normal mode
- **WHEN** user presses `t` then `n` while terminal is in normal mode
- **THEN** system switches to the next tab
- **AND** key prefix display shows `t` briefly after pressing `t`

#### Scenario: t p in terminal normal mode
- **WHEN** user presses `t` then `p` while terminal is in normal mode
- **THEN** system switches to the previous tab

#### Scenario: t prefix in terminal insert mode
- **WHEN** user presses `t` while terminal is in insert mode
- **THEN** `t` is sent to the shell (not intercepted)

#### Scenario: t prefix when command palette is open
- **WHEN** user presses `t` while command palette is open
- **THEN** `t` is typed into the command input (not intercepted)

#### Scenario: t prefix timeout
- **WHEN** user presses `t` but does not press a second key within 1 second
- **THEN** the prefix state resets and no tab operation occurs

#### Scenario: t prefix in editor insert mode
- **WHEN** user presses `t` while preview editor is in insert mode
- **THEN** `t` is typed into the editor (not intercepted)
