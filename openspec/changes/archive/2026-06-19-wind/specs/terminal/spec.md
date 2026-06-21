## ADDED Requirements

### Requirement: Terminal shell integration
The system SHALL provide an integrated terminal using xterm.js and Windows ConPTY, supporting multiple shell types.

#### Scenario: Start terminal with default shell
- **WHEN** Wind starts or user opens a new terminal
- **THEN** a terminal session starts with the system's default shell (PowerShell or cmd)

#### Scenario: Switch shell type
- **WHEN** user selects a different shell from the terminal menu
- **THEN** a new terminal session starts with the selected shell (cmd, PowerShell, Git Bash, WSL)

#### Scenario: Terminal input/output
- **WHEN** user types commands in the terminal
- **THEN** input is sent to the shell via ConPTY stdin, and output is rendered in xterm.js

### Requirement: Terminal ConPTY backend
The system SHALL use Windows ConPTY (Console Pseudo Terminal) as the terminal backend.

#### Scenario: ConPTY process creation
- **WHEN** a terminal session starts
- **THEN** a ConPTY process is created with the specified shell executable

#### Scenario: ConPTY output streaming
- **WHEN** the shell produces output
- **THEN** output is streamed from ConPTY stdout to xterm.js for rendering

#### Scenario: ConPTY resize
- **WHEN** the terminal panel is resized
- **THEN** the ConPTY dimensions are updated to match the new terminal size

### Requirement: Terminal mouse support
The system SHALL support mouse interactions in the terminal.

#### Scenario: Click to position cursor
- **WHEN** user clicks at a position in the terminal
- **THEN** the click event is forwarded to the shell for cursor positioning

#### Scenario: Scroll terminal history
- **WHEN** user scrolls the mouse wheel in the terminal
- **THEN** the terminal scrolls through its output history

### Requirement: Terminal panel integration
The system SHALL integrate the terminal as a panel within the Wind layout.

#### Scenario: Terminal follows file tree directory
- **WHEN** user navigates to a directory in the file tree
- **THEN** the terminal's working directory can be synced to that directory

#### Scenario: Terminal panel visibility
- **WHEN** user presses the terminal toggle shortcut
- **THEN** the terminal panel shows/hides while preserving its state
