# rar-archive-support Specification

## Purpose
RAR format read-only support including multipart archives (.partN.rar and .rar/.rXX naming conventions), password-protected archives, and archive preview.

## Requirements
### Requirement: RAR format detection
The system SHALL detect RAR archive files from their extension and treat them as a supported archive format.

#### Scenario: Detect single RAR file
- **WHEN** a file has extension `.rar` and is not part of a multipart collection
- **THEN** the system treats it as a RAR archive

#### Scenario: Detect RAR5 multipart with partN naming
- **WHEN** a file matches the pattern `*.partN.rar` (e.g., `archive.part1.rar`, `archive.part2.rar`)
- **THEN** the system treats each file as a RAR archive

#### Scenario: Detect RAR4 multipart with rXX naming
- **WHEN** a file matches the pattern `*.rar` or `*.rXX` where XX is digits (e.g., `archive.rar`, `archive.r00`, `archive.r01`)
- **THEN** the system treats each file as a RAR archive

#### Scenario: Case-insensitive RAR detection
- **WHEN** a file has extension `.RAR` or `.Rar`
- **THEN** the system treats it as a RAR archive

### Requirement: RAR read-only operations
The system SHALL support read-only operations on RAR archives. Write operations (create, delete, rename, add files) are not supported.

#### Scenario: RAR supports listing
- **WHEN** user enters a RAR archive
- **THEN** the system lists all entries in the archive

#### Scenario: RAR supports preview
- **WHEN** user selects a file inside a RAR archive
- **THEN** the system reads and previews the file content

#### Scenario: RAR supports extraction
- **WHEN** user presses `x` or `e` on entries in a RAR archive
- **THEN** the system extracts the files to the target directory

#### Scenario: RAR does not support write operations
- **WHEN** user attempts delete, rename, create file, or create directory in a RAR archive
- **THEN** the system shows a toast "This operation is only supported for ZIP archives"
- **AND** no action is taken

### Requirement: RAR multipart auto-first-volume
The system SHALL automatically navigate to the first volume when the user opens any volume of a multipart RAR archive.

#### Scenario: Open first volume of partN collection
- **WHEN** user opens `archive.part1.rar`
- **THEN** the system opens the archive starting from the first volume
- **AND** the unrar library automatically reads all subsequent volumes

#### Scenario: Open non-first volume of partN collection
- **WHEN** user opens `archive.part3.rar`
- **THEN** the system automatically redirects to `archive.part1.rar`
- **AND** opens the archive starting from the first volume
- **AND** the unrar library automatically reads all subsequent volumes

#### Scenario: Open RAR4 first volume
- **WHEN** user opens `archive.rar` (the first volume of a RAR4 multipart set)
- **THEN** the system opens the archive starting from this volume
- **AND** the unrar library automatically reads `.r00`, `.r01`, etc.

#### Scenario: Open RAR4 non-first volume
- **WHEN** user opens `archive.r02` (a non-first volume)
- **THEN** the system automatically redirects to `archive.rar`
- **AND** opens the archive starting from the first volume

### Requirement: RAR password support
The system SHALL support password-protected RAR archives using the same password prompt flow as ZIP and 7z archives. Multipart RAR archives share a unified password cache so entering the password for one volume applies to all volumes.

#### Scenario: Enter encrypted RAR archive
- **WHEN** user presses `l` on an encrypted RAR archive
- **THEN** the system shows the password prompt
- **AND** after a correct password, enters archive mode

#### Scenario: Wrong password for RAR
- **WHEN** user enters an incorrect password for a RAR archive
- **THEN** the dialog remains open with an error message

#### Scenario: Preview encrypted RAR file
- **WHEN** user selects a file inside an encrypted RAR archive
- **THEN** the system shows the password prompt if required
- **AND** after a correct password, previews the file content

#### Scenario: Extract encrypted RAR
- **WHEN** user extracts files from an encrypted RAR archive
- **THEN** the system shows the password prompt if required
- **AND** after a correct password, proceeds with extraction

#### Scenario: Unified password cache for multipart RAR
- **WHEN** user enters a password for any volume of a multipart RAR set
- **THEN** the password is cached under the first volume's canonical path
- **AND** accessing any other volume in the same set reuses the cached password without prompting

### Requirement: RAR archive preview
The system SHALL display the root-level directory structure of a RAR archive in the preview panel when the file is selected.

#### Scenario: Preview RAR archive entries
- **WHEN** user selects a `.rar` file in the file panel
- **THEN** the preview panel displays the archive's root-level entries
- **AND** directory entries are visually distinguished from file entries
- **AND** only direct children of the root are shown (one level deep)

#### Scenario: Preview multipart RAR
- **WHEN** user selects any volume of a multipart RAR set
- **THEN** the preview panel displays the archive's root-level entries from the first volume
- **AND** the preview shows content from all volumes combined

#### Scenario: Preview encrypted RAR
- **WHEN** user selects an encrypted RAR file
- **THEN** the preview panel shows that a password is required
- **AND** after a correct password, displays the root-level entries