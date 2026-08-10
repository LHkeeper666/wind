# transfer-manager Specification

## Purpose
Global transfer management panel for viewing and controlling all background file operations (local copy/move/delete, FTP upload/download) with progress tracking, queue management, and history persistence.

## Requirements
### Requirement: Transfer Manager panel visibility
The system SHALL provide a toggleable Transfer Manager panel accessible via `Ctrl+T` that displays all background file transfer operations globally (across all tabs).

#### Scenario: Open Transfer Manager via shortcut
- **WHEN** user presses `Ctrl+T` and the Transfer Manager is hidden
- **THEN** the Transfer Manager panel appears at the bottom of the window with default height 250px
- **AND** the panel displays all active, queued, and historical transfers

#### Scenario: Close Transfer Manager via shortcut
- **WHEN** user presses `Ctrl+T` and the Transfer Manager is visible
- **THEN** the Transfer Manager panel hides
- **AND** any ongoing transfers continue running in the background

#### Scenario: Close Transfer Manager via Escape
- **WHEN** the Transfer Manager panel is focused and user presses Escape
- **THEN** the panel closes and focus returns to the previously active column

#### Scenario: Transfers persist across panel close
- **WHEN** the Transfer Manager panel is closed during active transfers
- **THEN** transfers continue executing in the background
- **AND** re-opening the panel shows current transfer progress

### Requirement: Transfer Manager panel resize
The system SHALL allow users to resize the Transfer Manager panel via a drag handle.

#### Scenario: Resize via drag
- **WHEN** user drags the handle at the top of the Transfer Manager panel
- **THEN** the panel height adjusts between 100px and 80% of window height
- **AND** the height is remembered for the session

### Requirement: Transfer progress display
The system SHALL display each transfer operation with source, destination, progress, and status.

#### Scenario: Running transfer shows progress bar
- **WHEN** a transfer is in the "running" state
- **THEN** the panel displays: drag handle, direction icon, source → destination, percentage, mini progress bar, bytes done/total, speed, and cancel button on a single line

#### Scenario: Queued transfer shows file size
- **WHEN** a transfer is in the "queued" state
- **THEN** the panel displays: drag handle, waiting icon, source → destination, file size, and "queued" label

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

#### Scenario: Active batch divider
- **WHEN** a batch has at least one queued or running transfer
- **THEN** a divider line shows "Now" with operation type and progress summary (e.g., "Uploading (2/5 done)")

### Requirement: Transfer cancellation
The system SHALL allow users to cancel queued and running transfers individually.

#### Scenario: Cancel running transfer via button
- **WHEN** user clicks the cancel button on a running transfer
- **THEN** the transfer is cancelled, partial file cleaned up, and status changes to "cancelled"

#### Scenario: Cancel queued transfer via keyboard
- **WHEN** user selects a queued transfer and presses `c`
- **THEN** the transfer is removed from the queue with "cancelled" status

#### Scenario: Cancel all transfers
- **WHEN** user clicks "Cancel All" in the Transfer Manager header
- **THEN** all queued transfers are cancelled immediately and running transfers are requested to cancel

### Requirement: Transfer retry
The system SHALL allow users to retry failed transfers.

#### Scenario: Retry failed transfer
- **WHEN** user clicks the retry button on a failed transfer
- **THEN** the transfer is re-enqueued at the end of the queue with a new transfer ID

### Requirement: Queue reordering
The system SHALL allow users to reorder queued (not running) transfers via drag-and-drop or keyboard shortcuts.

#### Scenario: Drag to reorder queued items
- **WHEN** user drags a queued item's drag handle and drops it between two other queued items
- **THEN** the queue order is updated and the scheduler picks the next task from the new head of the queue

#### Scenario: Keyboard reorder
- **WHEN** user selects a queued item and presses `Ctrl+Shift+K`
- **THEN** the selected item moves up one position in the queue

#### Scenario: Running items are not reorderable
- **WHEN** a transfer is in "running" state
- **THEN** its drag handle is not interactive and keyboard reorder has no effect on it

### Requirement: Scroll behavior
The system SHALL manage scroll position to balance history browsing and live progress visibility.

#### Scenario: Auto-scroll when at bottom
- **WHEN** the user is scrolled to the bottom of the transfer list and a new progress event arrives
- **THEN** the view automatically scrolls to keep the latest content visible

#### Scenario: Pause auto-scroll when browsing history
- **WHEN** the user scrolls up to view historical transfers
- **THEN** auto-scroll is suspended and a floating "↓ back to bottom" button appears

#### Scenario: Jump to bottom
- **WHEN** user presses `G` or clicks the floating button
- **THEN** the view scrolls to the bottom and auto-scroll resumes

### Requirement: Transfer history persistence
The system SHALL persist completed, failed, and cancelled transfer records to `~/.local/share/wind/transfer-history.json`.

#### Scenario: Save on transfer completion
- **WHEN** a transfer reaches done, failed, or cancelled status
- **THEN** the system writes the transfer record to the history file

#### Scenario: Restore history on startup
- **WHEN** Wind starts
- **THEN** the system reads transfer-history.json and populates the Transfer Manager with historical records

#### Scenario: History retention
- **WHEN** the history file exceeds 500 entries or a record is older than 7 days (3 days for failed)
- **THEN** the system removes oldest/expired records during the next save

### Requirement: Transfer enqueue replaces direct invoke
The system SHALL use `transfer_enqueue` as the unified entry point for all file transfer operations, replacing direct `ftp_download`/`ftp_upload`/`copy_file_async` invokes from the UI layer.

#### Scenario: Enqueue FTP upload batch
- **WHEN** user pastes local files into an FTP panel
- **THEN** the UI calls `transfer_enqueue` with all files as a batch
- **AND** the invoke returns immediately with transfer IDs
- **AND** the Transfer Manager displays the queued batch

#### Scenario: Enqueue local copy batch
- **WHEN** user pastes local files into a local panel
- **THEN** the UI calls `transfer_enqueue` with the copy/move tasks
- **AND** the old per-file `copy_file_async` path is no longer used

### Requirement: Concurrent transfer limits
The system SHALL enforce concurrency limits to avoid overwhelming FTP servers or local disk I/O.

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

### Requirement: Status bar transfer indicator
The system SHALL display the count of active transfers in the status bar.

#### Scenario: Active transfer count
- **WHEN** there are active (queued + running) transfers
- **THEN** the status bar displays an indicator like "↑↓ 3 transfers"

#### Scenario: Click indicator to open panel
- **WHEN** user clicks the transfer indicator in the status bar
- **THEN** the Transfer Manager panel opens

### Requirement: Keyboard navigation in Transfer Manager
The system SHALL support vim-style keyboard navigation within the Transfer Manager panel.

#### Scenario: Select items with j/k
- **WHEN** the Transfer Manager panel is focused and user presses `j`
- **THEN** the selection moves to the next transfer entry
- **AND** pressing `k` moves selection to the previous entry

#### Scenario: Jump to top/bottom
- **WHEN** the Transfer Manager panel is focused and user presses `gg`
- **THEN** selection moves to the first entry
- **AND** pressing `G` moves selection to the last entry

#### Scenario: Clear completed entries
- **WHEN** user presses `x` on a selected done or failed entry
- **THEN** the entry is removed from the visible list
