## ADDED Requirements

### Requirement: InputDialog component
The system SHALL provide an inline input dialog component for user text input within directory panels.

#### Scenario: InputDialog appearance
- **WHEN** InputDialog is activated
- **THEN** it SHALL display at the top of the directory panel content area, below the panel header

#### Scenario: InputDialog auto-focus
- **WHEN** InputDialog is shown
- **THEN** the input field SHALL automatically receive focus

#### Scenario: InputDialog confirm with Enter
- **WHEN** user presses Enter in InputDialog
- **THEN** the dialog SHALL call the onConfirm callback with the current input value

#### Scenario: InputDialog cancel with Escape
- **WHEN** user presses Escape in InputDialog
- **THEN** the dialog SHALL call the onCancel callback and close

#### Scenario: InputDialog restores panel focus
- **WHEN** InputDialog is closed (confirm or cancel)
- **THEN** focus SHALL return to the directory panel

### Requirement: InputDialog modes
The system SHALL support different input dialog modes for different operations.

#### Scenario: Rename mode
- **WHEN** InputDialog is in rename mode
- **THEN** the input SHALL be pre-filled with the current file name

#### Scenario: Create file mode
- **WHEN** InputDialog is in create file mode
- **THEN** the input SHALL show a placeholder "New file name"

#### Scenario: Create directory mode
- **WHEN** InputDialog is in create directory mode
- **THEN** the input SHALL show a placeholder "New directory name"
