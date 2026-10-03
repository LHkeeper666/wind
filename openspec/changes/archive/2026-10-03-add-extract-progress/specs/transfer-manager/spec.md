## MODIFIED Requirements

### Requirement: Transfer progress display
The system SHALL display each transfer operation (including extraction) with source, destination, progress, status, speed, and estimated time remaining (ETA).

#### Scenario: Running transfer shows progress bar
- **WHEN** a transfer is in the "running" state
- **THEN** the panel displays: drag handle, direction icon, source → destination, percentage, mini progress bar, bytes done/total, speed, estimated time remaining (ETA), and cancel button on a single line

#### Scenario: Queued transfer shows file size
- **WHEN** a transfer is in the "queued" state
- **THEN** the panel displays: drag handle, waiting icon, source → destination, file size, and "queued" label
- **AND** for a local directory copy/move, the file size reflects the recursive total size of the directory contents
- **AND** for an extraction task with unknown total bytes (scanning), the panel displays "scanning..." instead of file size

#### Scenario: Completed transfer shows compact single line
- **WHEN** a transfer completes successfully
- **THEN** the panel displays: checkmark icon, source → destination, file size, average speed, and elapsed time on a single line

#### Scenario: Failed transfer shows error
- **WHEN** a transfer fails
- **THEN** the panel displays: cross icon, source → destination, and error message on a single line
- **AND** hovering over the error shows the full error detail in a tooltip

### Requirement: Batch grouping with dividers
The system SHALL group transfers by batch using divider lines, with the most recent batch at the bottom.

#### Scenario: Completed batch divider
- **WHEN** all transfers in a batch are done, failed, or cancelled
- **THEN** a divider line separates that batch from other batches showing: completion date, operation type, and file count/size summary
- **AND** the operation type label for extraction batches is "Extract"

#### Scenario: Active batch divider
- **WHEN** a batch has at least one queued or running transfer
- **THEN** a divider line shows "Now" with operation type and progress summary (e.g., "Extract (2/5 done)")

### Requirement: Transfer enqueue replaces direct invoke
The system SHALL use the transfer scheduler for local copy/move, FTP upload/download, and extraction tasks. The UI SHALL enqueue local copy/move, individual FTP file transfers, and archive extractions through their respective scheduler entry points. The UI SHALL NOT invoke retired direct file-transfer or extraction commands.

#### Scenario: Enqueue extraction batch
- **WHEN** user triggers an archive extraction via `e`, `E`+`p`, or `y`+`p`
- **THEN** the UI calls `extract_enqueue` with the extraction parameters
- **AND** the invoke returns immediately with a transfer ID
- **AND** the Transfer Manager displays the queued extraction

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

### Requirement: Concurrent transfer limits
The system SHALL enforce concurrency limits to avoid overwhelming FTP servers, local disk I/O, or CPU with decompression.

#### Scenario: FTP same-connection limit
- **WHEN** 5 uploads are queued for the same FTP connection
- **THEN** at most 2 uploads run simultaneously on that connection
- **AND** the remaining 3 stay queued until a slot frees

#### Scenario: FTP different-connection independence
- **WHEN** uploads are queued for two different FTP connections (connection-A and connection-B)
- **THEN** each connection allows up to 2 concurrent transfers independently

#### Scenario: Local same-drive serial
- **WHEN** multiple copy tasks are queued between folders on the same physical drive
- **THEN** they execute one at a time (serial)

#### Scenario: Local cross-drive parallel
- **WHEN** multiple copy tasks are queued between different physical drives
- **THEN** up to 2 tasks execute concurrently

#### Scenario: Extraction concurrency independent of transfers
- **WHEN** extraction tasks and file transfer tasks are both queued
- **THEN** extraction tasks use a separate slot pool (default max 2) independent of local and FTP transfer slots
- **AND** an extraction does not block or delay file transfers, and vice versa