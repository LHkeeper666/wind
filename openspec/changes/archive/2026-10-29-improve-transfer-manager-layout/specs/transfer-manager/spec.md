## MODIFIED Requirements

### Requirement: Transfer progress display
The system SHALL display each transfer operation with source, destination, progress, status, speed, and estimated time remaining (ETA) when applicable. Transfer rows SHALL use available panel width for file information and progress, and SHALL remain readable at narrow widths.

#### Scenario: Aligned transfer details
- **WHEN** running, queued, and completed transfers are displayed together
- **THEN** transfer sizes align in the same column across rows
- **AND** size, speed, and time values do not overlap adjacent columns

#### Scenario: Running transfer shows progress bar
- **WHEN** a transfer is in the "running" state and the panel has sufficient width
- **THEN** the panel displays: drag handle, direction icon, source → destination, percentage, progress bar, bytes done/total, speed, estimated time remaining (ETA), and cancel button in one row
- **AND** the source/destination regions and progress bar can expand into available horizontal space

#### Scenario: Running transfer in a narrow panel
- **WHEN** a transfer is running and the panel is too narrow to fit its information legibly in one line
- **THEN** the transfer information wraps or reflows within that row without horizontal scrolling
- **AND** the progress bar and cancel button remain visible and usable

#### Scenario: Queued transfer shows file size
- **WHEN** a transfer is in the "queued" state
- **THEN** the panel displays: drag handle, waiting icon, source → destination, file size, and "queued" label
- **AND** for a local directory copy/move, the file size reflects the recursive total size of the directory contents

#### Scenario: Completed transfer shows details
- **WHEN** a transfer completes successfully
- **THEN** the panel displays: checkmark icon, source → destination, file size, average speed, and elapsed time
- **AND** the information may reflow when the panel is narrow

#### Scenario: Failed transfer shows error
- **WHEN** a transfer fails
- **THEN** the panel displays: cross icon, source → destination, and error message
- **AND** hovering over the error shows the full error detail in a tooltip
- **AND** the information may reflow when the panel is narrow

## ADDED Requirements

### Requirement: Transfer Manager text readability
The system SHALL show Transfer Manager text at a readable default size and SHALL scale its text with the shared content zoom level.

#### Scenario: Compact title bar and rows
- **WHEN** the Transfer Manager is displayed at 100% zoom
- **THEN** its title bar and single-line transfer rows have vertical density comparable to the current directory panel

#### Scenario: Default zoom
- **WHEN** the content zoom level is 100%
- **THEN** primary transfer row text is at least 14px and secondary text is at least 12px

#### Scenario: Zoom changes while panel is open
- **WHEN** the user changes content zoom while the Transfer Manager is visible
- **THEN** its text changes proportionally with the shared zoom level
- **AND** rows and controls provide enough space for the scaled text

#### Scenario: Zoom reset
- **WHEN** the user resets content zoom to 100%
- **THEN** Transfer Manager text returns to its default size
