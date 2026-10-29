## MODIFIED Requirements

### Requirement: Archive virtual directory navigation
The system SHALL allow users to enter a supported archive file (.zip, .tar, .tar.gz, .7z, .rar) as a virtual directory using the `l` key, and navigate within it using the same `j/k/l/h` keybindings as regular directories. Archive entry content preview SHALL respect the backend size limit.

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
- **WHEN** user presses `l` or Enter on a file entry within a ZIP, TAR, TAR.GZ, or 7Z archive whose uncompressed size is at most 10 MiB and whose size can be bounded during reading
- **THEN** the file content is loaded and displayed in the preview panel
- **AND** file type detection and previewer routing work as for regular files, except for video entries

#### Scenario: Preview encrypted file within archive
- **WHEN** user presses `l` or Enter on a file entry within an encrypted `zip`, `7z`, or `rar` archive and its content is eligible for preview
- **THEN** the system shows the password prompt if the backend requires credentials
- **AND** after a correct password, the file content is loaded and displayed in the preview panel
- **AND** the same content size limit applies

#### Scenario: Large file preview limited
- **WHEN** a file within an archive exceeds 10 MiB of uncompressed content
- **THEN** the system SHALL NOT read or transmit the entry content for preview
- **AND** the preview SHALL show the file type, available size, and a clear size-limit message

#### Scenario: Video entry preview
- **WHEN** user selects a video entry within an archive
- **THEN** the system SHALL NOT request the complete video bytes for preview
- **AND** the preview SHALL show available file information and instruct the user to extract the video before opening it in the system player

#### Scenario: RAR entry preview
- **WHEN** user selects a file entry within a RAR archive
- **THEN** the system SHALL show file information and an extract-to-open instruction without reading its content
- **AND** RAR directory browsing and full extraction SHALL remain available

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

## ADDED Requirements

### Requirement: Archive preview content is bounded before allocation
The archive preview backend MUST enforce a 10 MiB limit on the uncompressed bytes of a single entry before allocating or returning its content. This protection SHALL apply to all supported archive formats and to callers that invoke the preview command directly. A declared size alone MUST NOT allow unbounded output from a malformed entry. Formats without a bounded content reader, currently RAR, SHALL return a no-content placeholder for every entry.

#### Scenario: Oversized entry requested directly
- **WHEN** the archive preview command is called for an entry larger than 10 MiB
- **THEN** it SHALL return a structured no-content result that identifies the size-limit reason
- **AND** it SHALL NOT allocate or transmit the full entry bytes

#### Scenario: Entry size cannot be safely bounded
- **WHEN** an archive format or entry has no trustworthy size and its reader cannot enforce the output limit
- **THEN** the backend SHALL return a no-content result before attempting an unbounded read

#### Scenario: Extraction remains available
- **WHEN** an archive entry is too large for content preview
- **THEN** the user SHALL still be able to extract it through the existing transfer path
