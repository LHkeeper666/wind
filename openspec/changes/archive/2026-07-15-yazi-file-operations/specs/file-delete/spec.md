## ADDED Requirements

### Requirement: Delete confirmation
The system SHALL require confirmation before deleting files.

#### Scenario: Delete single file
- **WHEN** user presses `d` on a file
- **THEN** system SHALL show a confirmation dialog asking to move the file to trash

#### Scenario: Delete selected files
- **WHEN** user presses `d` with multiple files selected
- **THEN** system SHALL show a confirmation dialog with the count of files to be deleted

#### Scenario: Delete confirmation dialog
- **WHEN** the delete confirmation is shown
- **THEN** it SHALL display the file name(s) and a confirmation message
- **THEN** it SHALL have Yes/No options
