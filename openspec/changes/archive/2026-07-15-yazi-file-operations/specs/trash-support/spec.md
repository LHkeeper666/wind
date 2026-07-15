## ADDED Requirements

### Requirement: Move to trash
The system SHALL allow users to move files to the system recycle bin.

#### Scenario: Delete with d key
- **WHEN** user presses `d` on a file in the directory panel
- **THEN** system SHALL show a confirmation dialog
- **WHEN** user confirms
- **THEN** system SHALL move the file to the system recycle bin (not permanent delete)

#### Scenario: Delete directory with d key
- **WHEN** user presses `d` on a directory
- **THEN** system SHALL move the entire directory to the recycle bin

#### Scenario: Trash operation feedback
- **WHEN** a file is successfully moved to trash
- **THEN** system SHALL show a toast "Moved to trash: filename"

### Requirement: Permanent delete
The system SHALL allow users to permanently delete files.

#### Scenario: Permanent delete with D key
- **WHEN** user presses `D` (Shift+d) on a file
- **THEN** system SHALL show a confirmation dialog with a warning
- **WHEN** user confirms
- **THEN** system SHALL permanently delete the file

#### Scenario: Permanent delete warning
- **WHEN** the permanent delete confirmation dialog is shown
- **THEN** it SHALL display a warning that the operation cannot be undone
