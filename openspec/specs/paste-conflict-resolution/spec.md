# paste-conflict-resolution Specification

## Purpose
TBD - created by archiving change cross-tab-file-operations. Update Purpose after archive.
## Requirements
### Requirement: Paste without conflict
The system SHALL directly execute the paste operation when no file name conflict exists.

#### Scenario: Paste file to new location
- **WHEN** user pastes a file and the destination directory does not contain a file with the same name
- **THEN** system executes the copy or move operation directly without prompting

### Requirement: Paste conflict detection
The system SHALL detect name conflicts before executing paste operations.

#### Scenario: Single file conflict
- **WHEN** user pastes a file and a file with the same name exists in the destination
- **THEN** system pauses and shows a confirmation modal

#### Scenario: Multiple files with conflicts
- **WHEN** user pastes multiple files and some have name conflicts
- **THEN** system processes files sequentially, showing the confirmation modal for each conflict

### Requirement: Conflict confirmation modal
The system SHALL display a custom-styled modal when a paste conflict is detected.

#### Scenario: Modal displays conflict information
- **WHEN** a paste conflict is detected
- **THEN** system shows a modal displaying the conflicting file name and three options: Overwrite, Skip, Abort

#### Scenario: Modal matches application theme
- **WHEN** the conflict modal is displayed
- **THEN** it SHALL use the application's CSS variables for colors and fonts, consistent with the rest of the UI

### Requirement: Overwrite option
The system SHALL allow users to overwrite the existing file.

#### Scenario: User chooses overwrite
- **WHEN** user selects Overwrite in the conflict modal
- **THEN** system deletes the existing file at the destination and executes the paste operation for that file

#### Scenario: Overwrite with cut operation
- **WHEN** user overwrites during a cut paste and the operation succeeds
- **THEN** system deletes both the original file and the overwritten file (the original is moved)

### Requirement: Skip option
The system SHALL allow users to skip conflicting files.

#### Scenario: User chooses skip
- **WHEN** user selects Skip in the conflict modal
- **THEN** system skips that file and continues processing remaining files

### Requirement: Abort option
The system SHALL allow users to abort the entire paste operation.

#### Scenario: User chooses abort
- **WHEN** user selects Abort in the conflict modal
- **THEN** system stops the paste operation entirely, remaining files are not processed

### Requirement: Move file backend command
The system SHALL provide a Rust command for moving files that handles same-drive and cross-drive scenarios.

#### Scenario: Move file on same drive
- **WHEN** source and destination are on the same drive letter
- **THEN** system uses `fs::rename` for an atomic move operation

#### Scenario: Move file across drives
- **WHEN** source and destination are on different drive letters
- **THEN** system copies the file to the destination and then deletes the source

#### Scenario: Move fails gracefully
- **WHEN** a move operation fails (e.g., permission denied)
- **THEN** system shows an error toast and does not corrupt source or destination

