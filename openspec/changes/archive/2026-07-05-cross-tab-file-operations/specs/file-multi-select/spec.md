## ADDED Requirements

### Requirement: Toggle individual file selection
The system SHALL allow users to toggle selection of individual files using Space key.

#### Scenario: Select a file with Space
- **WHEN** user presses Space on an unselected file in the current directory panel
- **THEN** system adds that file to the selection set and moves cursor down one position

#### Scenario: Deselect a file with Space
- **WHEN** user presses Space on an already-selected file in the current directory panel
- **THEN** system removes that file from the selection set and moves cursor down one position

#### Scenario: Space skips parent directory entry
- **WHEN** user presses Space on the `..` entry
- **THEN** system does not toggle selection and moves cursor down one position

### Requirement: Select all files
The system SHALL allow users to select or deselect all files at once.

#### Scenario: Select all with v
- **WHEN** user presses `v` with no files currently selected
- **THEN** system selects all files in the current directory (excluding `..`)

#### Scenario: Deselect all with v
- **WHEN** user presses `v` with some files currently selected
- **THEN** system deselects all files

### Requirement: Multi-select visual feedback
The system SHALL provide clear visual feedback for selected files.

#### Scenario: Selected file visual indicator
- **WHEN** one or more files are in the selection set
- **THEN** each selected file SHALL display with a distinct background color different from the cursor highlight

#### Scenario: Selection count in status
- **WHEN** files are selected
- **THEN** the directory panel or status area shows the count of selected files

### Requirement: Multi-select resets on navigation
The system SHALL clear the selection when the user navigates to a different directory.

#### Scenario: Selection cleared on directory change
- **WHEN** user navigates to a different directory (via Enter, h, l, or cd command)
- **THEN** the selection set is cleared

### Requirement: Multi-select works with yank and cut
The system SHALL use the selection set for yank and cut operations.

#### Scenario: Yank respects selection
- **WHEN** user presses `y` with files selected
- **THEN** system yanks all selected files (not just the one under cursor)

#### Scenario: Cut respects selection
- **WHEN** user presses `x` with files selected
- **THEN** system cuts all selected files (not just the one under cursor)

#### Scenario: Yank without selection uses cursor
- **WHEN** user presses `y` with no files selected
- **THEN** system yanks only the file under cursor

#### Scenario: Cut without selection uses cursor
- **WHEN** user presses `x` with no files selected
- **THEN** system cuts only the file under cursor
