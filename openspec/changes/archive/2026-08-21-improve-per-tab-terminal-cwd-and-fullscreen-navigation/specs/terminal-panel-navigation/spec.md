## MODIFIED Requirements

### Requirement: t prefix tab operations work globally
The `t` prefix for tab operations (t n, t p, t c, t t, etc.) SHALL work from any panel focus state, including fullscreen terminal normal mode, as long as no modal overlay (command palette, file search, fullscreen editor) is open and the active input is not in insert mode.

#### Scenario: t n in fullscreen terminal normal mode
- **WHEN** user presses `t` then `n` while terminal is in fullscreen normal mode
- **THEN** system switches to the next tab
- **AND** key prefix display shows `t` briefly after pressing `t`

#### Scenario: t p in fullscreen terminal normal mode
- **WHEN** user presses `t` then `p` while terminal is in fullscreen normal mode
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

### Requirement: Ctrl+L restores focus to fullscreen terminal
系统 SHALL allow Ctrl+L to restore focus to the active fullscreen terminal when no modal overlay is open.

#### Scenario: Ctrl+L in fullscreen terminal
- **WHEN** terminal 处于 fullscreen normal mode 且用户按下 Ctrl+L
- **THEN** 焦点回到当前 tab 的 terminal
- **AND** terminal 可以继续接收 normal mode 快捷键

### Requirement: Panel switching disabled in fullscreen terminal
系统 SHALL disable Ctrl+W panel switching when the terminal is in fullscreen mode.

#### Scenario: Panel switch attempt in fullscreen
- **WHEN** the user presses Ctrl+W followed by h/l/j/k while in fullscreen terminal
- **THEN** no panel switching operation is executed
