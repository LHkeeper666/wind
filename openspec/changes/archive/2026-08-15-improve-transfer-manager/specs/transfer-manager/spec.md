## MODIFIED Requirements

### Requirement: Transfer progress display
The system SHALL display each transfer operation with source, destination, progress, status, speed, and estimated time remaining (ETA).

#### Scenario: Running transfer shows progress bar
- **WHEN** a transfer is in the "running" state
- **THEN** the panel displays: drag handle, direction icon, source → destination, percentage, mini progress bar, bytes done/total, speed, estimated time remaining (ETA), and cancel button on a single line

#### Scenario: Queued transfer shows file size
- **WHEN** a transfer is in the "queued" state
- **THEN** the panel displays: drag handle, waiting icon, source → destination, file size, and "queued" label
- **AND** for a local directory copy/move, the file size reflects the recursive total size of the directory contents

#### Scenario: Completed transfer shows compact single line
- **WHEN** a transfer completes successfully
- **THEN** the panel displays: checkmark icon, source → destination, file size, average speed, and elapsed time on a single line

#### Scenario: Failed transfer shows error
- **WHEN** a transfer fails
- **THEN** the panel displays: cross icon, source → destination, and error message on a single line
- **AND** hovering over the error shows the full error detail in a tooltip

### Requirement: Transfer cancellation
The system SHALL allow users to cancel transfers individually, and to cancel all queued and running transfers atomically via a single backend call.

#### Scenario: Cancel running transfer via button
- **WHEN** user clicks the cancel button on a running transfer
- **THEN** the transfer is cancelled, partial file cleaned up, and status changes to "cancelled"

#### Scenario: Cancel queued transfer via keyboard
- **WHEN** user selects a queued transfer and presses `c`
- **THEN** the transfer is removed from the queue with "cancelled" status

#### Scenario: Cancel all transfers atomically
- **WHEN** user clicks "Cancel All" in the Transfer Manager header
- **THEN** the UI invokes a single backend cancel-all command
- **AND** all queued transfers are removed and marked cancelled without being dispatched or promoted to running
- **AND** all running transfers have their cancel flag set
- **AND** the panel updates once rather than once per cancelled transfer

## ADDED Requirements

### Requirement: Estimated time remaining (ETA)
The system SHALL compute and display an estimated time remaining for running transfers.

#### Scenario: ETA shown when speed available
- **WHEN** a transfer is running with a known total size and a measurable transfer speed
- **THEN** the panel displays an estimated time remaining based on remaining bytes divided by a rolling-average speed

#### Scenario: ETA hidden when speed unavailable
- **WHEN** a transfer is running but the measured speed is zero or the total size is unknown
- **THEN** no ETA is displayed

#### Scenario: ETA for local transfers
- **WHEN** a local copy, move, or delete is running
- **THEN** the ETA is computed from frontend-observed progress, regardless of backend-emitted speed
