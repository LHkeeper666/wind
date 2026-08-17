## ADDED Requirements

### Requirement: Transfer conflict detection
The system SHALL detect filename conflicts (files only, not directories) before copying or moving directories, and before FTP uploads/downloads, so that existing destination files are not silently overwritten. Directories always merge into existing directories or are created if absent.

#### Scenario: Local directory copy detects internal conflicts
- **WHEN** user pastes (copies/moves) a directory into a target directory that already contains a file with the same name inside it
- **THEN** the system detects the conflicting files by comparing the directory trees
- **AND** presents a summary confirmation (overwrite / skip / abort) before transferring

#### Scenario: Directories merge without conflict
- **WHEN** user copies/moves a directory into a target that already contains a same-named directory but no same-named files inside it
- **THEN** the system merges into the existing directory without prompting a conflict
- **AND** only file-name collisions inside trigger conflict prompts

#### Scenario: Skip conflicts still copies non-conflicting files
- **WHEN** user chooses "skip" on a directory with internal conflicts
- **THEN** the system copies only the non-conflicting files and skips the conflicting files, preserving existing destination files

#### Scenario: Force paste skips conflict confirmation
- **WHEN** user presses `P` (shift+p) to force paste
- **THEN** the transfer proceeds without conflict confirmation and overwrites existing files

#### Scenario: FTP upload detects remote conflicts
- **WHEN** user uploads a file or directory to an FTP target that already contains a same-named entry
- **THEN** the system lists the remote target directory and detects the conflict
- **AND** presents a confirmation before overwriting

#### Scenario: FTP download detects local conflicts
- **WHEN** user downloads a file or directory from FTP to a local target that already contains a same-named entry
- **THEN** the system detects the conflict via local file existence and presents a confirmation

### Requirement: Cancel cleanup for partial directory transfers
The system SHALL clean up partially transferred content when a directory copy/move is cancelled.

#### Scenario: Cancel directory copy to new target rolls back
- **WHEN** user cancels a directory copy/move whose target directory did not exist before the transfer
- **THEN** the system removes the entire target directory, including all partially copied files and created subdirectories

#### Scenario: Cancel directory copy over existing target removes only new files
- **WHEN** user cancels a directory copy/move whose target directory already existed before the transfer
- **THEN** the system removes only the files written by this transfer, preserving pre-existing destination content
