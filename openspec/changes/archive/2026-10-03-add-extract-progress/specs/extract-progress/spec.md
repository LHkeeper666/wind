## ADDED Requirements

### Requirement: Extraction tasks managed by TransferScheduler
The system SHALL route all archive extraction operations through the TransferScheduler, treating extraction as a transfer operation with queue management, slot-based concurrency, and history persistence.

#### Scenario: Extraction enqueued with immediate UI feedback
- **WHEN** user triggers extraction via `e`, `E`+`p`, or `y`+`p`
- **THEN** the system invokes `extract_enqueue` which creates a task in the TransferScheduler
- **AND** the scheduler immediately emits `transfer-progress` with `status: "queued"` and `total_bytes: 0`
- **AND** the Transfer Manager displays a new entry for the extraction task

#### Scenario: Extraction scanning updates total bytes
- **WHEN** an extraction task is enqueued and the backend scans archive entries for sizes
- **THEN** the scheduler emits `transfer-progress` with the computed `total_bytes`
- **AND** the Transfer Manager entry updates to show the total extraction size

#### Scenario: Extraction respects concurrency slots
- **WHEN** multiple extraction tasks are queued and `extract_max_slots` is 2
- **THEN** at most 2 extractions run concurrently
- **AND** remaining extractions stay queued until a slot frees

#### Scenario: Extraction history persisted
- **WHEN** an extraction task completes (done, failed, or cancelled)
- **THEN** the task record is saved to `transfer-history.json` alongside file transfer records
- **AND** the record includes source archive path, destination, total bytes, and final status

### Requirement: Extraction progress tracking
The system SHALL report extraction progress per-file at 100ms intervals, displaying progress in the Transfer Manager with the same UI as file transfers.

#### Scenario: Running extraction shows progress
- **WHEN** an extraction is in "running" state
- **THEN** the Transfer Manager displays: extract icon, archive path → destination, percentage, mini progress bar, bytes done/total, and cancel button

#### Scenario: Extraction speed and ETA
- **WHEN** an extraction is running with known total bytes
- **THEN** the frontend computes speed via exponential moving average (same as transfers)
- **AND** an estimated time remaining is displayed when speed is measurable

#### Scenario: Completed extraction shows summary
- **WHEN** an extraction completes successfully
- **THEN** the Transfer Manager displays: checkmark icon, archive path → destination, total bytes extracted, and elapsed time

### Requirement: Extraction cancellation
The system SHALL allow users to cancel running extractions, preserving partially extracted files.

#### Scenario: Cancel running extraction
- **WHEN** user clicks the cancel button on a running extraction
- **THEN** the system sets the cancellation flag
- **AND** the current file finishes writing, then the extraction stops
- **AND** already-extracted files remain on disk (no rollback)
- **AND** the task status changes to "cancelled"

#### Scenario: Cancel queued extraction
- **WHEN** user cancels a queued extraction before it starts
- **THEN** the task is removed from the queue with "cancelled" status
- **AND** no files are written to disk

#### Scenario: Cancel all includes extractions
- **WHEN** user clicks "Cancel All" in the Transfer Manager
- **THEN** all queued and running extraction tasks are cancelled alongside file transfers

### Requirement: Extract operation type in Transfer Manager
The system SHALL display extraction tasks in the Transfer Manager with a distinct operation type label and icon.

#### Scenario: Extract operation label
- **WHEN** an extraction task appears in the Transfer Manager
- **THEN** the batch divider shows "Extract" as the operation type label
- **AND** the running icon is distinct from copy/download icons

#### Scenario: Extract task in active count
- **WHEN** extraction tasks are queued or running
- **THEN** they are included in the active transfer count shown in the header and status bar

### Requirement: All extraction paths use extract_enqueue
The system SHALL route all extraction triggers through the `extract_enqueue` Tauri command.

#### Scenario: e key extraction uses scheduler
- **WHEN** user presses `e` on an archive file
- **THEN** after conflict detection completes, the system calls `extract_enqueue` with the archive path, destination, skip_paths, and password
- **AND** the extraction appears in the Transfer Manager

#### Scenario: E+p extraction uses scheduler
- **WHEN** user marks an archive with `E` and presses `p` in the target directory
- **THEN** the system calls `extract_enqueue` with the archive path, current directory, and password
- **AND** the extraction appears in the Transfer Manager

#### Scenario: y+p extraction from archive browser uses scheduler
- **WHEN** user yanks entries from within an archive (`y`) and presses `p` in a target directory
- **THEN** the system calls `extract_enqueue` with `internal_paths` set to the yanked entry paths
- **AND** the extraction appears in the Transfer Manager

### Requirement: Extraction slot configuration
The system SHALL allow configuration of maximum concurrent extraction tasks.

#### Scenario: Default extraction concurrency
- **WHEN** Wind starts with no prior configuration
- **THEN** the maximum concurrent extraction tasks is 2

#### Scenario: Configure extraction slots
- **WHEN** user or system calls `transfer_set_extract_slots` with a value between 1 and 8
- **THEN** the extraction slot limit updates and affects subsequent dispatch decisions