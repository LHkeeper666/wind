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
The system SHALL allow users to paste clipboard contents into the current directory, routing to the appropriate backend based on source and destination path schemes. In project tree mode, the destination directory SHALL be derived from the focused node: a focused directory is the destination, and a focused file uses its parent directory. When the clipboard operation type is `extract`, the system SHALL extract the marked archive to the current directory.

#### Scenario: Paste copy operation
- **WHEN** user presses `p` with clipboard operation `copy`
- **THEN** system copies each file from clipboard to the current directory, routing to the appropriate backend
- **AND** local copies and individual local/FTP uploads or downloads use `transfer_enqueue`, while FTP directory transfers use the existing folder-transfer entry points
- **AND** the system does not invoke the retired `copy_file`, `copy_file_async`, `ftp_upload`, or `ftp_download` commands

#### Scenario: Paste cut operation
- **WHEN** user presses `p` with clipboard operation `cut`
- **THEN** system moves each file from clipboard to the current directory (copy + delete source), routing through the appropriate backend, and clears the cut visual indicators

#### Scenario: Paste extract operation
- **WHEN** user presses `p` with clipboard operation `extract`
- **THEN** system extracts the marked archive to the current directory
- **AND** displays extraction progress in the Transfer Manager
- **AND** clears the extract mark after extraction completes

#### Scenario: Extract to path uses same extraction rules
- **WHEN** extracting via `p` with operation type `extract`
- **THEN** the same subdirectory creation rules as `e` extraction apply: if the archive contains multiple top-level entries, extract into a subdirectory named after the archive; if the archive contains a single top-level directory, extract its contents directly

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

#### Scenario: Status bar shows extract mark
- **WHEN** clipboard has an entry with operation `extract`
- **THEN** status bar shows "Archive marked for extraction"

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

### Requirement: Clipboard extract mark operation
The system SHALL allow users to mark an archive file for extraction using the `E` key, which stores the archive path in the clipboard with operation type `extract`.

#### Scenario: Mark archive for extraction
- **WHEN** user presses `E` on an archive file (.zip, .tar, .tar.gz, .7z) with no multi-selection
- **THEN** the system stores the archive path in clipboard with operation type `extract`
- **AND** shows toast "Archive marked for extraction. Navigate to target and press p."

#### Scenario: Extract mark clears previous clipboard
- **WHEN** user presses `E` while clipboard already has entries of any type (copy, cut, or extract)
- **THEN** the system replaces clipboard contents with the new extract mark

#### Scenario: E key on non-archive file
- **WHEN** user presses `E` on a non-archive file
- **THEN** the system shows toast "E key only works on archive files"
- **AND** clipboard is unchanged

### Requirement: E/y/x mutual exclusion
The system SHALL treat extract mark (`E`), yank (`y`), and cut (`x`) as mutually exclusive operations. Setting any one SHALL clear the others.

#### Scenario: E clears yank mark
- **WHEN** user has files yanked (operation type `copy`) and presses `E` on an archive
- **THEN** the yank mark is cleared and replaced with the extract mark
- **AND** the status bar updates to show the extract mark

#### Scenario: y clears extract mark
- **WHEN** user has an archive marked for extraction (operation type `extract`) and presses `y` on files
- **THEN** the extract mark is cleared and replaced with the yank mark
- **AND** the status bar updates to show "N files yanked"

#### Scenario: x clears extract mark
- **WHEN** user has an archive marked for extraction (operation type `extract`) and presses `x` on files
- **THEN** the extract mark is cleared and replaced with the cut mark
- **AND** the status bar updates to show "N files cut"

### Requirement: Extract files from archive mode with x key
The system SHALL allow users to extract selected files from within an archive to an external directory using the `x` key when in archive mode.

#### Scenario: Extract single file from archive
- **WHEN** user presses `x` on a file entry in archive mode (no multi-selection)
- **THEN** the system extracts that file to the directory containing the archive file
- **AND** if the file already exists, prompts for overwrite confirmation

#### Scenario: Extract multiple selected files from archive
- **WHEN** user presses `x` with multiple files selected in archive mode
- **THEN** the system extracts all selected files to the directory containing the archive file
- **AND** displays progress in the Transfer Manager

#### Scenario: Extract directory from archive
- **WHEN** user presses `x` on a directory entry in archive mode
- **THEN** the system extracts the directory and all its contents recursively
- **AND** displays progress in the Transfer Manager

#### Scenario: x key in normal mode unchanged
- **WHEN** user presses `x` in normal directory mode (not in archive)
- **THEN** the system performs the existing cut operation (behavior unchanged)
