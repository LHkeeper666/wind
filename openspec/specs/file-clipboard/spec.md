# file-clipboard Specification

## Purpose
TBD - created by archiving change cross-tab-file-operations. Update Purpose after archive.
## Requirements
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
The system SHALL allow users to paste clipboard contents into the current directory, routing to the appropriate backend based on source and destination path schemes. In project tree mode, the destination directory SHALL be derived from the focused node: a focused directory is the destination, and a focused file uses its parent directory.

#### Scenario: Paste copy operation
- **WHEN** user presses `p` with clipboard operation `copy`
- **THEN** system copies each file from clipboard to the current directory, routing to local copy, ftp_upload, or ftp_download as appropriate

#### Scenario: Paste cut operation
- **WHEN** user presses `p` with clipboard operation `cut`
- **THEN** system moves each file from clipboard to the current directory (copy + delete source), routing through the appropriate backend, and clears the cut visual indicators

#### Scenario: Paste from selected project directory
- **WHEN** user presses `p` in project tree mode with a directory node focused
- **THEN** system pastes clipboard contents into that focused directory

#### Scenario: Paste from selected project file
- **WHEN** user presses `p` in project tree mode with a file node focused
- **THEN** system pastes clipboard contents into the focused file's parent directory

#### Scenario: Paste clears clipboard
- **WHEN** paste operation completes (all files processed or aborted)
- **THEN** system clears the clipboard and removes all cut visual indicators

#### Scenario: Paste with empty clipboard
- **WHEN** user presses `p` with empty clipboard
- **THEN** system shows toast "Clipboard empty" and does nothing

#### Scenario: Force paste without conflict prompt
- **WHEN** user presses `P` (Shift+p) with clipboard entries
- **THEN** system pastes all files, overwriting any conflicts without prompting

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

