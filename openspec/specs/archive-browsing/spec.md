# archive-browsing Specification

## Purpose
TBD - created by archiving change archive-browsing-and-management. Update Purpose after archive.
## Requirements
### Requirement: Archive virtual directory navigation
The system SHALL allow users to enter a supported archive file (.zip, .tar, .tar.gz, .7z) as a virtual directory using the `l` key, and navigate within it using the same `j/k/l/h` keybindings as regular directories.

#### Scenario: Enter archive with l key
- **WHEN** user presses `l` on a supported archive file in the current panel
- **THEN** the current panel enters archive mode, displaying the archive's root-level entries as a directory listing
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

### Requirement: ArchiveState management
The system SHALL maintain an `ArchiveState` in the layout store that tracks the current archive path, internal path, and format when browsing an archive.

#### Scenario: ArchiveState set on enter
- **WHEN** user enters an archive
- **THEN** `archiveState` is set with `archivePath`, `internalPath: ""`, and auto-detected `format`

#### Scenario: ArchiveState updated on navigate
- **WHEN** user navigates to a subdirectory within an archive
- **THEN** `archiveState.internalPath` is updated to the new internal path

#### Scenario: ArchiveState cleared on exit
- **WHEN** user exits the archive
- **THEN** `archiveState` is set to null

#### Scenario: ArchiveState prevents entering sub-archives
- **WHEN** user is already in archive mode and presses `l` on a nested archive file
- **THEN** the nested archive file is previewed as a regular file, not entered
- **AND** `archiveState` remains unchanged

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

#### Scenario: Case-insensitive detection
- **WHEN** a file has extension `.ZIP` or `.Zip`
- **THEN** the system treats it as a ZIP archive

