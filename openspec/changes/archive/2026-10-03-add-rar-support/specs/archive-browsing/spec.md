## MODIFIED Requirements

### Requirement: Archive virtual directory navigation
The system SHALL allow users to enter a supported archive file (.zip, .tar, .tar.gz, .7z, .rar) as a virtual directory using the `l` key, and navigate within it using the same `j/k/l/h` keybindings as regular directories.

#### Scenario: Enter archive with l key
- **WHEN** user presses `l` on a supported archive file in the current panel
- **THEN** the current panel enters archive mode, displaying the archive's root-level entries as a directory listing
- **AND** the path bar shows the archive name with a package icon

#### Scenario: Enter encrypted archive with l key
- **WHEN** user presses `l` on an encrypted `zip`, `7z`, or `rar` archive file in the current panel
- **THEN** the system shows the password prompt
- **AND** after a correct password, the current panel enters archive mode and displays the archive's root-level entries as a directory listing
- **AND** the path bar shows the archive name with a package icon

#### Scenario: Navigate into subdirectory within archive
- **WHEN** user presses `l` on a directory entry within an archive
- **THEN** the panel displays entries within that subdirectory
- **AND** the path bar updates to show the nested internal path

#### Scenario: Exit archive at root level with h key
- **WHEN** user presses `h` at the root level of an archive
- **THEN** the panel exits archive mode and returns to the parent directory containing the archive file
- **AND** the archive file is selected in the listing

#### Scenario: Go up one level within archive with h key
- **WHEN** user presses `h` within a subdirectory of an archive
- **THEN** the panel navigates to the parent internal directory
- **AND** the path bar updates accordingly

#### Scenario: Preview file within archive
- **WHEN** user presses `l` or Enter on a file entry within an archive
- **THEN** the file content is loaded and displayed in the preview panel
- **AND** file type detection and previewer routing works identically to regular files

#### Scenario: Preview encrypted file within archive
- **WHEN** user presses `l` or Enter on a file entry within an encrypted `zip`, `7z`, or `rar` archive
- **THEN** the system shows the password prompt if the backend requires credentials
- **AND** after a correct password, the file content is loaded and displayed in the preview panel
- **AND** file type detection and previewer routing works identically to regular files

#### Scenario: Large file preview limited
- **WHEN** a file within an archive exceeds 10MB
- **THEN** the system reads only the first 1MB for preview
- **AND** displays a note that the preview is truncated

#### Scenario: Refresh archive directory
- **WHEN** user presses `R` in archive mode
- **THEN** the archive directory listing is re-read from the archive file
- **AND** the `..` parent directory entry is preserved (computed from `archiveState.internalPath` rather than stored in the file list)

#### Scenario: Parent directory entry in archive
- **WHEN** viewing a subdirectory within an archive
- **THEN** a `..` entry is shown at the top of the listing
- **AND** the `..` entry path points to the parent internal path
- **AND** pressing Enter on `..` navigates to the parent directory (or exits archive at root)

#### Scenario: Unsupported archive format
- **WHEN** user presses `l` on a file with an unsupported archive extension
- **THEN** the file opens in the preview panel as a regular file (existing behavior)

### Requirement: Archive format detection
The system SHALL automatically detect the archive format from the file extension.

#### Scenario: Detect ZIP format
- **WHEN** a file has extension `.zip`
- **THEN** the system treats it as a ZIP archive

#### Scenario: Detect TAR format
- **WHEN** a file has extension `.tar`
- **THEN** the system treats it as a TAR archive

#### Scenario: Detect TAR.GZ format
- **WHEN** a file has extension `.tar.gz` or `.tgz`
- **THEN** the system treats it as a gzip-compressed TAR archive

#### Scenario: Detect 7Z format
- **WHEN** a file has extension `.7z`
- **THEN** the system treats it as a 7Z archive

#### Scenario: Detect RAR format
- **WHEN** a file has extension `.rar`
- **THEN** the system treats it as a RAR archive

#### Scenario: Case-insensitive detection
- **WHEN** a file has extension `.ZIP` or `.Zip`
- **THEN** the system treats it as a ZIP archive

### Requirement: Visible archive password prompt
The system SHALL provide a visible password input for encrypted `zip`, `7z`, or `rar` archives when credentials are required for entering the archive or previewing an internal file.

#### Scenario: Preview shows password hint
- **WHEN** the user selects an encrypted `zip`, `7z`, or `rar` archive in the file panel
- **THEN** the preview panel shows that a password is required to preview the archive
- **AND** the system does not automatically open the password dialog

#### Scenario: Prompt appears when entering encrypted archive
- **WHEN** the user presses `l` on an encrypted `zip`, `7z`, or `rar` archive
- **THEN** the system shows a password input dialog before entering archive mode
- **AND** the password is shown in plain text while typing

#### Scenario: Wrong password keeps dialog open
- **WHEN** the entered password is rejected by the backend
- **THEN** the dialog remains open
- **AND** the system shows an error message indicating the password is incorrect
- **AND** the user can edit and resubmit the password

### Requirement: Archive preview shows root-level directory structure
The system SHALL display the root-level directory structure of an archive file in the preview panel when the archive file is selected.

#### Scenario: Preview ZIP archive root-level entries
- **WHEN** user selects a .zip file in the file panel
- **THEN** the preview panel SHALL display the archive's root-level entries
- **AND** directory entries SHALL be visually distinguished from file entries (e.g., trailing `/` and different styling)
- **AND** only direct children of the root SHALL be shown (one level deep)

#### Scenario: Preview encrypted archive root-level entries
- **WHEN** user selects an encrypted `zip`, `7z`, or `rar` file in the file panel
- **THEN** the preview panel shows that a password is required to preview the archive
- **AND** the system does not automatically open the password dialog
- **AND** after a correct password, the preview panel SHALL display the archive's root-level entries
- **AND** directory entries SHALL be visually distinguished from file entries (e.g., trailing `/` and different styling)
- **AND** only direct children of the root SHALL be shown (one level deep)

#### Scenario: Preview TAR archive root-level entries
- **WHEN** user selects a .tar file in the file panel
- **THEN** the preview panel SHALL display the archive's root-level entries

#### Scenario: Preview TAR.GZ archive root-level entries
- **WHEN** user selects a .tar.gz or .tgz file in the file panel
- **THEN** the preview panel SHALL display the archive's root-level entries

#### Scenario: Preview 7Z archive root-level entries
- **WHEN** user selects a .7z file in the file panel
- **THEN** the preview panel SHALL display the archive's root-level entries

#### Scenario: Preview RAR archive root-level entries
- **WHEN** user selects a .rar file in the file panel
- **THEN** the preview panel SHALL display the archive's root-level entries

#### Scenario: Preview shows file sizes
- **WHEN** the archive preview displays file entries
- **THEN** each file entry SHALL show its size in human-readable format (B/KB/MB/GB)
- **AND** directory entries SHALL NOT show a size

#### Scenario: Preview shows entry count summary
- **WHEN** the archive preview is displayed
- **THEN** a header SHALL show the archive name, total file count, and total size

#### Scenario: Empty archive preview
- **WHEN** an archive contains no entries
- **THEN** the preview panel SHALL display "Empty archive"

#### Scenario: Archive preview error handling
- **WHEN** the system fails to read the archive (e.g., corrupted file)
- **THEN** the preview panel SHALL display an error message
- **AND** SHALL NOT crash the application

#### Scenario: Unsupported archive format in preview
- **WHEN** a file has an extension not in the supported set (zip, tar, tar.gz, tgz, 7z, rar)
- **THEN** the ArchivePreviewer SHALL NOT match the file
- **AND** the file SHALL be handled by other previewers in the chain