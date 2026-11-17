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
- **THEN** system processes files sequentially, showing the confirmation modal for each conflict unless the user has chosen an apply-to-all action for the current paste

### Requirement: Conflict confirmation modal
The system SHALL display a custom-styled modal when a paste conflict is detected.

#### Scenario: Modal displays local conflict information
- **WHEN** a local-to-local `p` paste with multiple clipboard entries detects a conflicting file
- **THEN** system shows the conflicting file name and Overwrite, Skip, Keep Copy, Duplicate All, and Abort choices

#### Scenario: Single top-level file conflict
- **WHEN** a local-to-local `p` paste contains only one top-level file and it conflicts
- **THEN** system shows Overwrite, Skip, Keep Copy, and Abort without Duplicate All

#### Scenario: Other conflict flows retain their choices
- **WHEN** a conflict is detected for FTP transfer or archive extraction
- **THEN** system does not show the Keep Copy or Duplicate All choice

#### Scenario: Modal matches application theme
- **WHEN** the conflict modal is displayed
- **THEN** it SHALL use the application's CSS variables for colors and fonts, consistent with the rest of the UI

### Requirement: Keep a suffixed copy on local paste conflict
The system SHALL allow the incoming file in a local-to-local `p` paste to be written under a unique suffixed name without replacing the existing destination file.

#### Scenario: Copy a top-level file
- **WHEN** user chooses Keep Copy for `report.pdf` and `report.pdf` already exists at the destination
- **THEN** system copies the incoming file to the first available name `report (n).pdf`, starting with `report (1).pdf`, and leaves the existing file unchanged

#### Scenario: Copy an extensionless or leading-dot file
- **WHEN** user chooses Keep Copy for an extensionless name or a leading-dot name such as `.gitignore`
- **THEN** system appends ` (n)` to the complete name

#### Scenario: Keep a conflicting file inside a pasted directory
- **WHEN** user chooses Keep Copy for a file that conflicts inside a locally pasted directory
- **THEN** system merges the directory as before and writes that incoming file under a unique suffixed name in its corresponding destination subdirectory

#### Scenario: Cut paste keeps the existing destination
- **WHEN** user chooses Keep Copy during a local cut paste and the transfer succeeds
- **THEN** system leaves the existing destination file unchanged and removes the transferred source file

#### Scenario: Repeated or batched copy names
- **WHEN** a candidate suffixed name already exists or has been assigned to another file in the same paste batch
- **THEN** system selects the next available numeric suffix

#### Scenario: Duplicate all remaining local conflicts
- **WHEN** user chooses Duplicate All for a local `p` paste conflict
- **THEN** system keeps the current incoming file and every later conflicting file in that paste under unique suffixed names without further conflict prompts

#### Scenario: Duplicate all across directory and top-level files
- **WHEN** user chooses Duplicate All for a conflict inside a pasted directory and the same paste also contains top-level file conflicts
- **THEN** system applies Keep Copy to the remaining directory conflicts and the top-level file conflicts

#### Scenario: Duplicate all ends with the paste
- **WHEN** a paste using Duplicate All finishes and the user starts another `p` paste
- **THEN** the new paste prompts again for conflicts

#### Scenario: Destination appears before transfer executes
- **WHEN** another file appears at the selected Keep Copy destination before the transfer writes it
- **THEN** system SHALL NOT overwrite that file and SHALL choose another available suffix or fail without deleting the source

#### Scenario: Force paste
- **WHEN** user presses `P` to force a local paste
- **THEN** system retains existing overwrite behavior without offering Keep Copy

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

