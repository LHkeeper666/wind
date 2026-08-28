## ADDED Requirements

### Requirement: Extract selected files from archive
The system SHALL allow users to extract selected files from within an archive to an external directory using the `x` key in archive mode.

#### Scenario: Extract single file
- **WHEN** user presses `x` on a file entry in archive mode (no multi-selection)
- **THEN** the system extracts that file to the current working directory (the directory containing the archive file)
- **AND** if the file already exists, prompts for overwrite confirmation

#### Scenario: Extract multiple selected files
- **WHEN** user presses `x` with multiple files selected in archive mode
- **THEN** the system extracts all selected files to the current working directory
- **AND** displays progress in the Transfer Manager

#### Scenario: Extract directory from archive
- **WHEN** user presses `x` on a directory entry in archive mode
- **THEN** the system extracts the directory and all its contents recursively
- **AND** displays progress in the Transfer Manager

### Requirement: Delete entries from ZIP archive
The system SHALL allow users to delete entries from within a ZIP archive using the `d` key in archive mode. This operation is only available for ZIP archives.

#### Scenario: Delete single entry from ZIP
- **WHEN** user presses `d` on a file entry in ZIP archive mode
- **THEN** the system prompts for confirmation
- **AND** upon confirmation, removes the entry from the ZIP archive
- **AND** refreshes the archive directory listing

#### Scenario: Delete multiple entries from ZIP
- **WHEN** user presses `d` with multiple files selected in ZIP archive mode
- **THEN** the system prompts for confirmation with the count of entries
- **AND** upon confirmation, removes all selected entries from the ZIP archive

#### Scenario: Delete not available for non-ZIP archives
- **WHEN** user presses `d` in a tar, tar.gz, or 7z archive
- **THEN** the system shows a toast "Delete is only supported for ZIP archives"
- **AND** no action is taken

### Requirement: Rename entry in ZIP archive
The system SHALL allow users to rename entries within a ZIP archive using the `r` key in archive mode. This operation is only available for ZIP archives.

#### Scenario: Rename entry in ZIP
- **WHEN** user presses `r` on an entry in ZIP archive mode
- **THEN** the system shows an inline rename input
- **AND** upon confirmation, renames the entry within the ZIP archive
- **AND** refreshes the archive directory listing

#### Scenario: Rename not available for non-ZIP archives
- **WHEN** user presses `r` in a tar, tar.gz, or 7z archive
- **THEN** the system shows a toast "Rename is only supported for ZIP archives"
- **AND** no action is taken

### Requirement: Compress selected files into ZIP archive
The system SHALL allow users to create a ZIP archive from selected files and directories using the `c` key in normal directory mode.

#### Scenario: Compress single file
- **WHEN** user selects a file and presses `c`
- **THEN** the system prompts for an archive name (defaulting to the file name with .zip extension)
- **AND** upon confirmation, creates a ZIP archive containing the file in the current directory
- **AND** displays progress in the Transfer Manager

#### Scenario: Compress multiple files
- **WHEN** user selects multiple files and presses `c`
- **THEN** the system prompts for an archive name
- **AND** upon confirmation, creates a ZIP archive containing all selected files and directories

#### Scenario: Compress shortcut unavailable in archive mode
- **WHEN** user presses `c` in archive mode
- **THEN** the key is ignored (no compression within archives)

#### Scenario: Default archive name
- **WHEN** the prompt for archive name appears
- **THEN** the default name is derived from the selected file/directory name, or "archive.zip" if multiple items are selected

### Requirement: Extract entire archive to current directory
The system SHALL allow users to extract an entire archive to the current directory using the `e` key in normal directory mode.

#### Scenario: Extract archive with e key
- **WHEN** user presses `e` on an archive file in the current panel
- **THEN** the system extracts the entire archive to the current directory
- **AND** displays progress in the Transfer Manager

#### Scenario: Extract archive creates subdirectory
- **WHEN** the archive contains multiple top-level entries
- **THEN** the system extracts into a subdirectory named after the archive (without extension)

#### Scenario: Extract archive with single top-level directory
- **WHEN** the archive contains exactly one top-level directory
- **THEN** the system extracts the contents of that directory directly to the target

### Requirement: Mark archive for extract-to-path
The system SHALL allow users to mark an archive for later extraction using the `E` key, then navigate to a target directory and press `p` to extract there.

#### Scenario: Mark archive with E key
- **WHEN** user presses `E` on an archive file
- **THEN** the system marks the archive as extract source
- **AND** shows a toast "Archive marked for extraction. Navigate to target and press p."
- **AND** clears any existing yank/cut marks

#### Scenario: Extract marked archive with p key
- **WHEN** user has an archive marked for extraction and presses `p`
- **THEN** the system extracts the archive to the current directory
- **AND** displays progress in the Transfer Manager
- **AND** clears the extract mark

#### Scenario: Extract to path uses same extraction rules
- **WHEN** extracting via `E` + `p`
- **THEN** the same subdirectory creation rules as `e` extraction apply

### Requirement: E/y/x mutual exclusion
The system SHALL treat extract mark (`E`), yank (`y`), and cut (`x`) as mutually exclusive operations, where marking one clears the others.

#### Scenario: E clears yank mark
- **WHEN** user has files yanked and presses `E` on an archive
- **THEN** the yank mark is cleared and replaced with the extract mark

#### Scenario: y clears extract mark
- **WHEN** user has an archive marked for extraction and presses `y` on files
- **THEN** the extract mark is cleared and replaced with the yank mark

#### Scenario: x clears extract mark
- **WHEN** user has an archive marked for extraction and presses `x` on files
- **THEN** the extract mark is cleared and replaced with the cut mark

#### Scenario: Status bar reflects current mark type
- **WHEN** an extract mark is active
- **THEN** the status bar shows "Archive marked for extraction"
- **WHEN** a compress mark is active
- **THEN** the status bar shows "N files marked for compression"
- **WHEN** a yank mark is active
- **THEN** the status bar shows "N files yanked"
- **WHEN** a cut mark is active
- **THEN** the status bar shows "N files cut"

### Requirement: Mark files for compression
The system SHALL allow users to mark selected files for compression using the `C` key, then press `p` to open a compress dialog at the target directory.

#### Scenario: Mark files with C key
- **WHEN** user selects files and presses `C` in normal directory mode
- **THEN** the system marks the files for compression
- **AND** shows a toast "N files marked for compression. Press p to compress."
- **AND** clears any existing yank/cut/extract marks

#### Scenario: Compress marked files with p key
- **WHEN** user has files marked for compression and presses `p`
- **THEN** the system shows a dialog prompting for archive name
- **AND** the default name is derived from the single file name or "archive.zip" for multiple files
- **AND** upon confirmation, creates a ZIP archive in the current directory

#### Scenario: Compress mark shortcut unavailable in archive mode
- **WHEN** user presses `C` in archive mode
- **THEN** the key is ignored

#### Scenario: C/y/x/E mutual exclusion
- **WHEN** user has a yank/cut/extract mark active and presses `C`
- **THEN** the previous mark is cleared and replaced with the compress mark

### Requirement: Create files and directories inside ZIP archive
The system SHALL allow users to create new files and directories within a ZIP archive using the `a` and `a/` keys. This operation is only available for ZIP archives.

#### Scenario: Create empty file in ZIP
- **WHEN** user presses `a` in ZIP archive mode
- **THEN** the system shows an input dialog for the file name
- **AND** upon confirmation, creates an empty file entry in the ZIP archive
- **AND** refreshes the archive directory listing

#### Scenario: Create directory in ZIP
- **WHEN** user presses `a/` in ZIP archive mode
- **THEN** the system shows an input dialog for the directory name
- **AND** upon confirmation, creates a directory entry in the ZIP archive
- **AND** refreshes the archive directory listing

#### Scenario: Create entry not available for non-ZIP archives
- **WHEN** user presses `a` or `a/` in a tar, tar.gz, or 7z archive
- **THEN** the system shows a toast "Create is only supported for ZIP archives"
- **AND** no action is taken

#### Scenario: Create entry fails if already exists
- **WHEN** user tries to create an entry with a name that already exists in the archive
- **THEN** the system shows an error toast "Entry already exists"