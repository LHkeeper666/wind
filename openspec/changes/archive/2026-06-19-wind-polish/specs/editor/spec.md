## MODIFIED Requirements

### Requirement: File editing
The system SHALL allow basic file editing.

#### Scenario: Open file for editing
- **WHEN** user selects a file in the file tree
- **THEN** the file content is loaded into the editor

#### Scenario: Edit file content
- **WHEN** user types in the editor
- **THEN** the content is updated in memory

#### Scenario: Save file
- **WHEN** user presses Ctrl+S
- **THEN** the modified content is saved to disk

#### Scenario: Modified indicator
- **WHEN** file content is modified
- **THEN** a modified indicator is shown in the tab

### Requirement: File creation
The system SHALL allow creating new files.

#### Scenario: Create new file
- **WHEN** user presses 'o' in file tree
- **THEN** a new file is created and opened for editing
