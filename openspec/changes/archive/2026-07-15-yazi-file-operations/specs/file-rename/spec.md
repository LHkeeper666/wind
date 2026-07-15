## ADDED Requirements

### Requirement: Rename file
The system SHALL allow users to rename files using the `r` key.

#### Scenario: Activate rename
- **WHEN** user presses `r` on a file in the directory panel
- **THEN** system SHALL show an InputDialog pre-filled with the current file name

#### Scenario: Confirm rename
- **WHEN** user enters a new name and presses Enter
- **THEN** system SHALL rename the file and refresh the directory listing

#### Scenario: Rename selects renamed file
- **WHEN** rename completes successfully
- **THEN** the renamed file SHALL be selected in the directory listing

#### Scenario: Rename conflict
- **WHEN** user enters a name that already exists
- **THEN** system SHALL show an error toast "File already exists"

#### Scenario: Cancel rename
- **WHEN** user presses Escape during rename
- **THEN** the rename operation SHALL be cancelled and no changes made
