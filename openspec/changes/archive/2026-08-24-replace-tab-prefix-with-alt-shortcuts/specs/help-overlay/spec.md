## ADDED Requirements

### Requirement: Help overlay documents Alt tab bindings
The declarative keybinding registry and help overlay SHALL display the complete Alt-based tab shortcut set and SHALL not present retired `t`-prefix tab operations.

#### Scenario: Help overlay shows current tab shortcut mapping
- **WHEN** the user opens the help overlay
- **THEN** the Tab group identifies Alt shortcuts and lists `Alt+T`, `Alt+C`, `Alt+R`, `Alt+N`, `Alt+H`, `Alt+L`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9` with their corresponding actions
