# multi-selection-statistics Specification

## Purpose

Show responsive, cancellable aggregate information for explicitly selected local files and directories.

## Requirements

### Requirement: Display selected-item statistics
The system SHALL show aggregate information when `i` is pressed in a local directory or project tree with an explicit selection. The aggregate SHALL show the number of distinct selected root items, recursively included files, recursively included directories, and total file bytes.

#### Scenario: Multiple files selected
- **WHEN** the user selects multiple local files and presses `i`
- **THEN** the information panel shows the selected-item count, the same number of files, zero directories, and the sum of their sizes
- **AND** the cursor item is excluded unless explicitly selected

#### Scenario: Selected directories and files
- **WHEN** the selection includes a local directory and a separate file and the user presses `i`
- **THEN** selected-item count reflects the two roots
- **AND** file and directory counts and total bytes include the directory's descendants and the separate file
- **AND** the selected directory root counts as a directory

#### Scenario: Excluded or overlapping project-tree items
- **WHEN** a selected project-tree directory has excluded descendants or another selected root overlaps it
- **THEN** excluded descendants are omitted and included paths are counted once

#### Scenario: No explicit selection
- **WHEN** the user presses `i` without an explicit selection
- **THEN** the existing information view for the cursor item is shown

### Requirement: Calculate selected-item statistics asynchronously
The system SHALL calculate recursive statistics outside the UI thread, show partial progress, and allow cancellation when the panel closes.

#### Scenario: Long-running traversal
- **WHEN** the user opens aggregate information for a large selection
- **THEN** the panel opens promptly with zero or partial values
- **AND** file count, directory count, and bytes update as traversal progresses without blocking UI interaction

#### Scenario: Cancel on close
- **WHEN** the user closes the aggregate panel during traversal
- **THEN** the active traversal is canceled and no later event updates the closed panel

#### Scenario: New calculation supersedes old calculation
- **WHEN** a second aggregate calculation starts before an earlier calculation's events have finished
- **THEN** the panel accepts progress only from the current calculation

### Requirement: Report incomplete statistics
The system SHALL avoid presenting partial statistics as complete when filesystem entries cannot be read. It SHALL skip symbolic links and junctions during recursive traversal.

#### Scenario: Unreadable entry
- **WHEN** traversal cannot read a selected item or descendant
- **THEN** traversal continues for readable entries and the result is marked incomplete

#### Scenario: Symbolic link or junction
- **WHEN** traversal encounters a symbolic link or junction
- **THEN** it does not follow or count that link's target
