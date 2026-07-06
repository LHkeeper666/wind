## ADDED Requirements

### Requirement: Async file copy with progress

The system SHALL copy files and directories asynchronously, emitting progress events to the frontend.

#### Scenario: Copy single large file
- **WHEN** user invokes copy of a 1GB file
- **THEN** the operation returns immediately with an `op_id`
- **AND** the backend scans the source to compute total bytes
- **AND** the backend emits `op-scan-complete` with `{ op_id, total_bytes }`
- **AND** the backend emits `op-progress` events at least every 100ms with `{ op_id, bytes_done, total_bytes, current_file }` during the copy
- **AND** the backend emits `op-complete` with `{ op_id }` when done

#### Scenario: Copy directory recursively
- **WHEN** user invokes copy of a directory with 500 files
- **THEN** the backend scans the directory tree to compute `total_files` and `total_bytes`
- **AND** the `op-scan-complete` event includes both `total_files` and `total_bytes`
- **AND** `op-progress` events include `files_done` and `total_files` fields

### Requirement: Cancel in-progress file operation

The system SHALL support cancelling a running file operation.

#### Scenario: User cancels a copy
- **WHEN** user invokes `cancel_file_op` with a valid `op_id`
- **AND** the operation is still running
- **THEN** the operation stops at the next chunk boundary (within 64KB)
- **AND** partial destination files are cleaned up
- **AND** the backend emits `op-cancelled` with `{ op_id }`

#### Scenario: Cancel an already-completed operation
- **WHEN** user invokes `cancel_file_op` for a completed operation
- **THEN** the command returns a no-op and does not affect the completed result

### Requirement: Conflict pre-check before execution

The system SHALL check for destination conflicts before starting a copy or move operation.

#### Scenario: Destination exists — user overwrites
- **WHEN** a copy would overwrite an existing file
- **THEN** the pre-check command returns a list of conflicting paths
- **AND** the frontend displays a conflict resolution dialog
- **AND** user can choose Overwrite, Skip, or Abort per conflict

#### Scenario: No conflicts found
- **WHEN** no files at the destination paths exist
- **THEN** the pre-check returns an empty list
- **AND** the operation proceeds directly to execution

### Requirement: Progress bar in bottom status bar

The system SHALL display file operation progress in a bottom status bar component.

#### Scenario: Single operation in progress
- **WHEN** a file copy is running
- **THEN** a progress bar appears at the bottom of the window
- **AND** the bar shows: operation icon, current file name, percentage, bytes done / total bytes, cancel button

#### Scenario: Multiple concurrent operations
- **WHEN** two or more file operations are running simultaneously
- **THEN** each operation has its own progress bar stacked vertically
- **AND** cancelling one operation does not affect others

#### Scenario: Operation completes
- **WHEN** a file operation finishes successfully
- **THEN** the progress bar shows "✓ Done" in green
- **AND** the bar fades out and is removed after 3 seconds

#### Scenario: Operation fails
- **WHEN** a file operation encounters an error
- **THEN** the progress bar shows "✕" and the error message in red
- **AND** the bar remains until the user manually closes it

### Requirement: Event throttling

The system SHALL throttle progress events to prevent frontend overload.

#### Scenario: Very fast copy operation
- **WHEN** the backend copies data faster than 100ms per chunk
- **THEN** progress events are emitted at most once every 100ms
- **AND** the final `op-complete` event is always emitted regardless of throttling

### Requirement: Same-drive move optimization

The system SHALL use `fs::rename` for same-drive moves (instant, no progress needed).

#### Scenario: Same-drive move
- **WHEN** source and destination are on the same drive letter
- **THEN** the move uses `fs::rename` which is atomic and instant
- **AND** `op-complete` is emitted immediately after rename succeeds

#### Scenario: Cross-drive move
- **WHEN** source and destination are on different drive letters
- **THEN** the move is implemented as async copy + delete
- **AND** progress events are emitted during the copy phase
