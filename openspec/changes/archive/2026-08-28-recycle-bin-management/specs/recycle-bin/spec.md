## ADDED Requirements

### Requirement: Enter recycle bin view

The system SHALL provide a keyboard shortcut `gr` to toggle the recycle bin view. When entering, the current directory panel SHALL display the contents of the Windows Recycle Bin. The parent panel SHALL display recycle bin statistics and shortcut hints.

#### Scenario: Enter recycle bin view
- **WHEN** user presses `g` then `r` in normal mode
- **THEN** the current panel switches to recycle bin mode, showing trashed files
- **AND** the parent panel shows recycle bin statistics (item count, total size) and shortcut hints

#### Scenario: Exit recycle bin view
- **WHEN** user presses `h` in recycle bin mode, or presses `gr` again
- **THEN** the current panel returns to the previously browsed directory

#### Scenario: Exit recycle bin view via cd command
- **WHEN** user executes `cd <path>` command from command palette in recycle bin mode
- **THEN** the recycle bin view exits and the current panel navigates to the specified path

### Requirement: List recycle bin contents

The system SHALL enumerate all items in the Windows Recycle Bin and display them as a file list. Each item SHALL show its original file name, and the preview panel SHALL show the original path, deletion date, and file size.

#### Scenario: Display recycle bin items
- **WHEN** recycle bin view is active
- **THEN** the system calls `trash::os_limited::list()` and displays all items
- **AND** each item shows its original file name
- **AND** items are sortable by name, deletion date, and size

#### Scenario: Empty recycle bin
- **WHEN** recycle bin is empty
- **THEN** the list displays an "empty" placeholder message

#### Scenario: Loading state
- **WHEN** recycle bin contents are being fetched
- **THEN** a loading indicator is shown while the async operation completes

### Requirement: Preview recycle bin files

The system SHALL support previewing files in the recycle bin. When a file is selected in the recycle bin list, the preview panel SHALL render the file content using the existing preview system.

#### Scenario: Preview text file in recycle bin
- **WHEN** user selects a text file in the recycle bin
- **THEN** the preview panel displays the file content using the existing text previewer
- **AND** the preview panel shows the file's original path, deletion date, and size

#### Scenario: Preview unsupported file type
- **WHEN** user selects a file type that has no matching previewer
- **THEN** the preview panel shows "Unsupported file type" message

### Requirement: Restore recycle bin items

The system SHALL allow restoring selected items from the recycle bin to their original locations. Multi-selection restore SHALL be supported.

#### Scenario: Restore single file
- **WHEN** user selects a file and presses `r` in recycle bin mode
- **THEN** the file is restored to its original path
- **AND** the recycle bin list is refreshed

#### Scenario: Restore file with name collision
- **WHEN** user attempts to restore a file whose original path already has a file with the same name
- **THEN** a confirmation dialog appears asking the user to skip or overwrite
- **AND** the restore operation proceeds according to the user's choice

#### Scenario: Restore multiple files
- **WHEN** user selects multiple files (multi-select) and presses `r`
- **THEN** all selected files are restored to their original paths
- **AND** the recycle bin list is refreshed after completion

#### Scenario: Restore failure
- **WHEN** restore fails (e.g., original drive no longer exists)
- **THEN** an error toast is displayed with the failure reason
- **AND** items that could not be restored remain in the list

### Requirement: Permanently delete recycle bin items

The system SHALL allow permanently deleting items from the recycle bin. This operation is irreversible and SHALL require confirmation.

#### Scenario: Permanent delete single file
- **WHEN** user selects a file and presses `d` in recycle bin mode
- **THEN** a confirmation dialog appears
- **AND** upon confirmation, the file is permanently deleted from the recycle bin
- **AND** the recycle bin list is refreshed

#### Scenario: Permanent delete multiple files
- **WHEN** user selects multiple files and presses `d`
- **THEN** a confirmation dialog appears showing the count of items to delete
- **AND** upon confirmation, all selected files are permanently deleted

#### Scenario: Cancel permanent delete
- **WHEN** confirmation dialog is shown for permanent delete
- **AND** user cancels the operation
- **THEN** no files are deleted and the recycle bin list remains unchanged

### Requirement: Empty entire recycle bin

The system SHALL allow emptying the entire recycle bin in one operation. This operation is irreversible and SHALL require confirmation.

#### Scenario: Empty recycle bin
- **WHEN** user presses `g` then `d` in recycle bin mode
- **THEN** a confirmation dialog appears
- **AND** upon confirmation, all items are permanently deleted from the recycle bin
- **AND** the recycle bin list shows "empty"

#### Scenario: Cancel empty recycle bin
- **WHEN** confirmation dialog is shown for emptying recycle bin
- **AND** user cancels the operation
- **THEN** the recycle bin is not emptied and the list remains unchanged

### Requirement: Disabled operations in recycle bin mode

The system SHALL disable operations that are not applicable in the recycle bin context. The clipboard operations (`y`/`x`), new file creation (`a`), and rename (`r` in normal meaning) SHALL be no-ops in recycle bin mode.

#### Scenario: Yank disabled in recycle bin
- **WHEN** user presses `y` on a recycle bin item
- **THEN** no action is taken (copying from recycle bin is not supported)

#### Scenario: Cut disabled in recycle bin
- **WHEN** user presses `x` on a recycle bin item
- **THEN** no action is taken

#### Scenario: New file disabled in recycle bin
- **WHEN** user presses `a` in recycle bin mode
- **THEN** no action is taken

### Requirement: Global shortcuts unaffected by recycle bin mode

The system SHALL ensure that global shortcuts (terminal toggle, file search, command palette, zoom, window navigation) work identically in recycle bin mode as in normal mode.

#### Scenario: Terminal toggle in recycle bin mode
- **WHEN** user presses Ctrl+` in recycle bin mode
- **THEN** the floating terminal toggles visibility
- **AND** the terminal's working directory is the last browsed directory (not the recycle bin)

#### Scenario: File search in recycle bin mode
- **WHEN** user presses Ctrl+P in recycle bin mode
- **THEN** the file search modal opens and searches the filesystem normally

#### Scenario: Command palette in recycle bin mode
- **WHEN** user presses `:` in recycle bin mode
- **THEN** the command palette opens normally