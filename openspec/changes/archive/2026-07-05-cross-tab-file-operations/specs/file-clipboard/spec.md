## ADDED Requirements

### Requirement: Clipboard yank operation
The system SHALL allow users to yank (copy) files to the clipboard for later pasting.

#### Scenario: Yank single file under cursor
- **WHEN** user presses `y` with no multi-selection and a file is under cursor
- **THEN** system stores that file in clipboard with operation type `copy` and shows toast "1 file yanked"

#### Scenario: Yank multiple selected files
- **WHEN** user presses `y` with multiple files selected
- **THEN** system stores all selected files in clipboard with operation type `copy` and shows toast "N files yanked"

#### Scenario: Yank clears previous clipboard
- **WHEN** user presses `y` while clipboard already has entries
- **THEN** system replaces clipboard contents with the new selection

### Requirement: Clipboard cut operation
The system SHALL allow users to cut files to the clipboard for later moving.

#### Scenario: Cut single file under cursor
- **WHEN** user presses `x` with no multi-selection and a file is under cursor
- **THEN** system stores that file in clipboard with operation type `cut`, marks file as cut in the UI, and shows toast "1 file cut"

#### Scenario: Cut multiple selected files
- **WHEN** user presses `x` with multiple files selected
- **THEN** system stores all selected files in clipboard with operation type `cut`, marks all as cut in the UI, and shows toast "N files cut"

#### Scenario: Cut files shown with visual indicator
- **WHEN** files are in clipboard with operation type `cut`
- **THEN** those files in the directory listing SHALL display with reduced opacity and an `x` marker on the left side

### Requirement: Clipboard paste operation
The system SHALL allow users to paste clipboard contents into the current directory.

#### Scenario: Paste copy operation
- **WHEN** user presses `p` with clipboard operation `copy`
- **THEN** system copies each file from clipboard to the current directory

#### Scenario: Paste cut operation
- **WHEN** user presses `p` with clipboard operation `cut`
- **THEN** system moves each file from clipboard to the current directory and clears the cut visual indicators

#### Scenario: Paste clears clipboard
- **WHEN** paste operation completes (all files processed or aborted)
- **THEN** system clears the clipboard and removes all cut visual indicators

#### Scenario: Paste with empty clipboard
- **WHEN** user presses `p` with empty clipboard
- **THEN** system shows toast "Clipboard empty" and does nothing

### Requirement: Clipboard status display
The system SHALL display clipboard state in the status bar.

#### Scenario: Status bar shows yank count
- **WHEN** clipboard has entries with operation `copy`
- **THEN** status bar shows "N files yanked"

#### Scenario: Status bar shows cut count
- **WHEN** clipboard has entries with operation `cut`
- **THEN** status bar shows "N files cut"

#### Scenario: Status bar hidden when empty
- **WHEN** clipboard is empty
- **THEN** status bar shows no clipboard indicator

### Requirement: Clipboard detail command
The system SHALL provide a command to view clipboard contents.

#### Scenario: View clipboard contents
- **WHEN** user types `:clip` in command palette
- **THEN** system shows a list of all files currently in clipboard with their paths

### Requirement: Clipboard clear command
The system SHALL provide a command to clear the clipboard.

#### Scenario: Clear clipboard
- **WHEN** user types `:clear` in command palette
- **THEN** system clears all clipboard entries, removes cut visual indicators, and shows toast "Clipboard cleared"
