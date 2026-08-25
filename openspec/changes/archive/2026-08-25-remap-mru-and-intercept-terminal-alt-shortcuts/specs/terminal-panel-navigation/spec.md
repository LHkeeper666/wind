## REMOVED Requirements

### Requirement: Terminal mode synced to store
**Reason**: The terminal is no longer modeled as separate application Insert and Normal modes. Shell input owns ordinary terminal keys, while Wind owns the explicit supported Alt tab chords regardless of shell state.
**Migration**: Treat terminal and fullscreen-terminal focus as one context. Do not expose or persist a terminal mode value for tab shortcut routing.

## MODIFIED Requirements

### Requirement: Alt-based tab operations work globally
The system SHALL provide tab operations through the direct shortcuts `Alt+N`, `Alt+U`, `Alt+R`, `Alt+M`, `Alt+H`, `Alt+L`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9` in supported non-modal application contexts. Generic editable inputs and dialogs SHALL retain Alt input, but terminal xterm input SHALL be an explicit exception for the supported tab chords. The same rules SHALL apply when the terminal is fullscreen.

#### Scenario: Direct tab actions in application panels
- **WHEN** the user presses `Alt+N`, `Alt+U`, `Alt+R`, `Alt+,`, `Alt+.`, `Alt+D`, or `Alt+1` through `Alt+9` in a directory, preview, or editor Normal context
- **THEN** the system performs the corresponding new, close, rename-hint, swap-backward, swap-forward, detach-toggle, or indexed-tab action

#### Scenario: MRU tab switcher commits on Alt release
- **WHEN** the user presses `Alt+M` with more than one tab open, optionally presses `M` again while still holding Alt, and then releases Alt
- **THEN** the system previews each selected MRU tab while cycling and commits the final previewed tab when Alt is released

#### Scenario: Display-order tab switcher commits on Alt release
- **WHEN** the user presses `Alt+L` or `Alt+H` with more than one tab open, optionally repeats `L` or `H` while still holding Alt, and then releases Alt
- **THEN** the system previews each selected tab in forward or backward display order and commits the final previewed tab when Alt is released

#### Scenario: Supported Alt tab actions from terminal focus
- **WHEN** the user presses any supported Alt tab shortcut while the embedded terminal has focus
- **THEN** the system handles the tab operation and prevents the chord from being forwarded to the shell

#### Scenario: Supported Alt tab actions from fullscreen terminal focus
- **WHEN** the user presses any supported Alt tab shortcut while the fullscreen terminal has focus
- **THEN** the system handles the tab operation and prevents the chord from being forwarded to the shell

#### Scenario: Generic editable and dialog contexts retain Alt input
- **WHEN** the user presses an Alt combination while the editor is in insert mode, or while a command, search, dialog, or other non-terminal text input is focused
- **THEN** the system SHALL NOT treat it as a tab operation

#### Scenario: Unrelated Alt and AltGr shortcuts remain available
- **WHEN** the user presses bare Alt, an unsupported Alt shortcut, or a `Ctrl+Alt` keyboard-layout shortcut in any context
- **THEN** the system SHALL NOT intercept it as a tab operation
