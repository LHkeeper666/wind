## MODIFIED Requirements

### Requirement: Panel resize
The system SHALL allow panels to be resized via mouse drag.

#### Scenario: Resize file tree panel
- **WHEN** user drags the border between file tree and editor
- **THEN** the file tree width changes accordingly

#### Scenario: Resize terminal panel
- **WHEN** user drags the border between editor and terminal
- **THEN** the terminal height changes accordingly

#### Scenario: Minimum size constraint
- **WHEN** user drags a panel border
- **THEN** panels respect minimum size constraints (100px minimum)

### Requirement: Panel persistence
The system SHALL persist panel sizes between sessions.

#### Scenario: Save panel sizes
- **WHEN** user closes the application
- **THEN** panel sizes are saved to configuration

#### Scenario: Restore panel sizes
- **WHEN** user opens the application
- **THEN** panel sizes are restored from saved configuration
