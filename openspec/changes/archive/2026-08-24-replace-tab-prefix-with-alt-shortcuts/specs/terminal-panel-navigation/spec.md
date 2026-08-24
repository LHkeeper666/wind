## ADDED Requirements

### Requirement: Alt-based tab operations work globally
The system SHALL provide tab operations through the direct shortcuts `Alt+T`, `Alt+C`, `Alt+R`, `Alt+N`, `Alt+H`, `Alt+L`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9` whenever the active context is not editable and no modal or excluded fullscreen overlay is open.

#### Scenario: Direct tab actions
- **WHEN** the user presses `Alt+T`, `Alt+C`, `Alt+R`, `Alt+,`, `Alt+.`, `Alt+D`, or `Alt+1` through `Alt+9` in a directory, preview, editor Normal mode, or terminal Normal mode
- **THEN** the system performs the corresponding new, close, rename-hint, swap-backward, swap-forward, detach-toggle, or indexed-tab action

#### Scenario: MRU tab switcher commits on Alt release
- **WHEN** the user presses `Alt+N` with more than one tab open, optionally presses `N` again while still holding Alt, and then releases Alt
- **THEN** the system previews each selected MRU tab while cycling and commits the final previewed tab when Alt is released

#### Scenario: Display-order tab switcher commits on Alt release
- **WHEN** the user presses `Alt+L` or `Alt+H` with more than one tab open, optionally repeats `L` or `H` while still holding Alt, and then releases Alt
- **THEN** the system previews each selected tab in forward or backward display order and commits the final previewed tab when Alt is released

#### Scenario: Alt tab operations in fullscreen terminal
- **WHEN** the user presses a supported Alt tab shortcut while the fullscreen terminal is focused
- **THEN** the system handles the shortcut according to its tab operation

#### Scenario: Editable and input contexts retain Alt input
- **WHEN** the user presses an Alt combination while the editor is in insert mode, or while a command, search, dialog, or other text input is focused
- **THEN** the system SHALL NOT treat it as a tab operation

#### Scenario: Unrelated Alt and AltGr shortcuts remain available
- **WHEN** the user presses bare Alt, an unsupported Alt shortcut, or a `Ctrl+Alt` keyboard-layout shortcut
- **THEN** the system SHALL NOT intercept it as a tab operation

## REMOVED Requirements

### Requirement: t prefix tab operations work globally
**Reason**: The timed `t` prefix conflicts with Vim's Normal-mode `t` motion and requires duplicate state machines.

**Migration**: Replace every former `t` sequence with the corresponding direct Alt shortcut: `t t` → `Alt+T`, `t c` → `Alt+C`, `t r` → `Alt+R`, `t n` → `Alt+N`, `t p` → `Alt+L`, `t ,` → `Alt+,`, `t .` → `Alt+.`, `t d` → `Alt+D`, and `t 1`–`9` → `Alt+1`–`9`.
