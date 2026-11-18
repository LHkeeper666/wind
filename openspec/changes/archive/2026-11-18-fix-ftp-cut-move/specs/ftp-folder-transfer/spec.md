## ADDED Requirements

### Requirement: Safe cut transfer completion
The system SHALL coordinate folder cut cleanup with the actual result of all child transfer tasks, including empty folders, for FTP-to-local and local-to-FTP transfers.

#### Scenario: Folder cut succeeds
- **WHEN** all child transfers finish successfully and destination content is verified
- **THEN** the system deletes the source directory tree and reports the move complete

#### Scenario: Folder cut fails
- **WHEN** any child transfer fails or is cancelled
- **THEN** the system preserves the source directory tree and reports the partial destination

#### Scenario: Empty folder cut
- **WHEN** a folder has no files
- **THEN** the system creates the destination folder and removes the source only after creation succeeds
