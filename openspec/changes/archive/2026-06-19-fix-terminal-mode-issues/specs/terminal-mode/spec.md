## MODIFIED Requirements

### Requirement: Terminal initial mode
The terminal SHALL initialize in normal mode with stdin disabled. The terminal SHALL NOT steal keyboard focus on startup when it is not the active panel.

#### Scenario: App starts with file-tree as active panel
- **WHEN** the application starts
- **THEN** the terminal mode SHALL be 'normal'
- **AND** the terminal stdin SHALL be disabled
- **AND** keyboard focus SHALL NOT be on the xterm canvas
- **AND** the status bar SHALL show 'NORMAL'

#### Scenario: Shell spawns in background
- **WHEN** the terminal initializes and the shell spawns
- **THEN** shell output SHALL display in the terminal
- **AND** the terminal SHALL remain in normal mode
- **AND** the user SHALL be able to use Ctrl+W to switch panels immediately

### Requirement: Panel switch preserves terminal mode
When entering the terminal panel via keyboard (Ctrl+W j) or mouse click, the terminal SHALL remain in normal mode. The user MUST explicitly press 'i' to enter insert mode.

#### Scenario: Switch to terminal via Ctrl+W j
- **WHEN** the user presses Ctrl+W then j to switch to the terminal panel
- **THEN** the terminal mode SHALL be 'normal'
- **AND** the overlay SHALL be visible
- **AND** the status bar SHALL show 'NORMAL'

#### Scenario: Switch to terminal via mouse click
- **WHEN** the user clicks on the terminal panel
- **THEN** the terminal mode SHALL be 'normal'
- **AND** the terminal panel SHALL become the active panel

#### Scenario: User enters insert mode after switching
- **WHEN** the terminal is in normal mode and the user presses 'i'
- **THEN** the terminal mode SHALL change to 'insert'
- **AND** stdin SHALL be enabled
- **AND** the xterm canvas SHALL receive focus
- **AND** the status bar SHALL show 'INSERT'

### Requirement: Panel switch disables terminal stdin
When leaving the terminal panel via keyboard (Ctrl+W h/k/l) or switching to another panel, the terminal SHALL switch to normal mode to prevent xterm from capturing keyboard input.

#### Scenario: Switch away from terminal
- **WHEN** the user is in the terminal panel and presses Ctrl+W then h/k/l
- **THEN** the terminal mode SHALL be set to 'normal'
- **AND** stdin SHALL be disabled
- **AND** xterm SHALL lose focus
- **AND** the target panel SHALL receive focus
