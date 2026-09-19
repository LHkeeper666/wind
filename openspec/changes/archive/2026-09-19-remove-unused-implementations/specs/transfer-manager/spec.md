## MODIFIED Requirements

### Requirement: Transfer enqueue replaces direct invoke
The system SHALL use the transfer scheduler for local copy/move and FTP upload/download tasks. The UI SHALL enqueue local copy/move and individual FTP file transfers through `transfer_enqueue`. FTP directory transfers SHALL retain `ftp_download_folder` and `ftp_upload_folder` as task-expansion entry points that enqueue work through the same scheduler. The UI SHALL NOT invoke retired direct file-transfer commands.

#### Scenario: Enqueue FTP upload batch
- **WHEN** user pastes local files into an FTP panel
- **THEN** the UI calls `transfer_enqueue` with all files as a batch
- **AND** the invoke returns immediately with transfer IDs
- **AND** the Transfer Manager displays the queued batch

#### Scenario: Enqueue local copy batch
- **WHEN** user pastes local files into a local panel
- **THEN** the UI calls `transfer_enqueue` with the copy/move tasks
- **AND** the old per-file `copy_file_async` path is no longer used

#### Scenario: Enqueue FTP download batch
- **WHEN** user pastes individual FTP files into a local panel
- **THEN** the UI calls `transfer_enqueue` with FTP download tasks
- **AND** the scheduler executes the transfers and publishes their progress

#### Scenario: Preserve FTP directory task expansion
- **WHEN** user pastes a directory across local and FTP panel boundaries
- **THEN** the appropriate folder-transfer command expands the directory into scheduler tasks
- **AND** removing the direct single-file commands does not remove the folder-transfer entry points

## ADDED Requirements

### Requirement: Legacy transfer commands are retired
The system SHALL remove implementations and Tauri registrations for `copy_file`, `move_file`, `permanent_delete`, `copy_file_async`, `move_file_async`, `delete_file_async`, `cancel_file_op`, `check_copy_conflicts`, `check_transfer_conflicts`, `check_ftp_upload_conflicts`, `ftp_download`, and `ftp_upload`. The system SHALL retain the active scheduler, cancellation, streaming conflict scanning, and recycle-bin deletion paths.

#### Scenario: Legacy transfer IPC is unavailable
- **WHEN** a client invokes any command in the retired transfer-command list
- **THEN** Tauri rejects the invocation because that command is no longer registered

#### Scenario: Conflict scanning remains available
- **WHEN** a paste operation requires conflict detection
- **THEN** the current `scan_transfer_conflicts`, `scan_ftp_upload_conflicts`, or `scan_ftp_download_conflicts` flow remains available as appropriate
- **AND** conflict decisions continue to use the current streaming events and dialogs

#### Scenario: Active deletion and cancellation remain available
- **WHEN** a user requests permanent deletion, moving a file to the recycle bin, or cancellation of a queued/running transfer
- **THEN** the application continues to use its existing scheduler, `delete_file`, and transfer-cancellation paths as appropriate
- **AND** the retired file-operation progress implementation is not required
