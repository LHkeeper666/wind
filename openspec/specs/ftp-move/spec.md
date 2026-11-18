# ftp-move Specification

## Purpose

Define safe cut moves for files and directories on one FTP connection.

## Requirements

### Requirement: Same-connection FTP cut move
The system SHALL move files and directories within one FTP connection using server-side rename when possible and a tracked transfer fallback when rename fails.

#### Scenario: Server accepts rename
- **WHEN** a user cuts an FTP entry and pastes it elsewhere on the same connection, and rename succeeds
- **THEN** the system reports that entry as moved and refreshes source and destination listings

#### Scenario: Server rejects directory rename
- **WHEN** rename of an FTP directory fails with 550
- **THEN** the system recursively enumerates the directory and transfers its contents to the destination with progress and cancellation
- **AND** the system never issues RETR on the directory itself

#### Scenario: Empty directory fallback
- **WHEN** the source directory has no files and rename fails
- **THEN** the system creates the target directory before removing the source directory

#### Scenario: Source is an ancestor of destination
- **WHEN** a user attempts to move a directory into itself or its descendant
- **THEN** the system rejects the move before creating or deleting remote content

### Requirement: Move completion barrier
The system SHALL preserve the complete source until every destination transfer in the move succeeds and destination content is verified.

#### Scenario: All child transfers succeed
- **WHEN** every file in a directory move has been transferred and verified
- **THEN** the system deletes source files and directories in safe bottom-up order and marks the move complete

#### Scenario: Transfer fails or is cancelled
- **WHEN** any child transfer fails or the user cancels the move
- **THEN** the source remains intact, the move is marked failed or cancelled, and any partial destination is disclosed

#### Scenario: Cleanup fails
- **WHEN** destination verification succeeds but a source deletion fails
- **THEN** the system reports a partial move, retains remaining source content, and does not claim success

#### Scenario: Source changes during transfer
- **WHEN** the source no longer matches the enumerated content before cleanup
- **THEN** the system aborts source cleanup and reports the mismatch

### Requirement: FTP move conflict handling
The system SHALL resolve destination conflicts before modifying the destination and SHALL not silently overwrite existing content.

#### Scenario: Existing destination
- **WHEN** the target already contains an entry with the same path and force paste is not requested
- **THEN** the system applies the user's paste conflict choice before transfer or rename

#### Scenario: Retry after partial destination
- **WHEN** a previous move left partial target content and the user retries
- **THEN** the system detects those conflicts and preserves source content until the retry completes
