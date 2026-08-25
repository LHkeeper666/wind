## MODIFIED Requirements

### Requirement: Help overlay documents Alt tab bindings
The declarative keybinding registry and help overlay SHALL display the complete Alt-based tab shortcut set and SHALL not present retired `t`-prefix operations or terminal Insert/Normal mode controls.

#### Scenario: Help overlay shows current tab shortcut mapping
- **WHEN** the user opens the help overlay
- **THEN** the Tab group identifies `Alt+N` as new tab, `Alt+M` as MRU switching, and lists `Alt+U`, `Alt+R`, `Alt+H`, `Alt+L`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9` with their corresponding actions

#### Scenario: Help overlay treats terminal as one focus context
- **WHEN** the user opens the help overlay
- **THEN** the terminal shortcut documentation does not distinguish Insert and Normal modes and states that supported Alt tab shortcuts apply while terminal focus is active
