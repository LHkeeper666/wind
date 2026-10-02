# archive-operations Specification (Delta)

## Modified Requirements

### Requirement: Extract entire archive to current directory

The system SHALL extract an archive into a newly created subdirectory named after the archive file (without extension), with conflict detection when the subdirectory already contains files.

#### Scenario: Extract archive with e key to new subdirectory
- **WHEN** user presses `e` on an archive file in the current panel
- **THEN** the system creates a subdirectory named after the archive (without full extension) in the current directory
- **AND** extracts the entire archive into that subdirectory
- **AND** displays progress in the Transfer Manager

#### Scenario: Extract archive to existing empty subdirectory
- **WHEN** the target subdirectory already exists but is empty
- **THEN** the system extracts directly into the existing subdirectory without prompting

#### Scenario: Extract archive to existing non-empty subdirectory without conflicts
- **WHEN** the target subdirectory exists and contains files, but none share paths with archive entries
- **THEN** the system extracts directly into the subdirectory without prompting

#### Scenario: Extract archive with conflicts in target subdirectory
- **WHEN** the target subdirectory exists and contains files that share paths with archive entries
- **THEN** the system shows the streaming conflict dialog for each conflicting file
- **AND** offers options: Overwrite, Skip, All Overwrite, Ignore All, Cancel
- **AND** extracts with the user's resolution applied (skipped files are excluded)

#### Scenario: Extract encrypted archive with e key
- **WHEN** user presses `e` on an encrypted `zip` or `7z` archive file
- **THEN** the system shows the password prompt if required
- **AND** after correct password, proceeds with subdirectory creation and extraction
- **AND** conflict detection occurs after password is resolved

#### Scenario: Subdirectory name derivation
- **WHEN** extracting an archive file
- **THEN** the subdirectory name is the archive filename with the full extension stripped
- **AND** supported extensions: `.tar.gz`, `.tgz`, `.tar`, `.zip`, `.7z`

#### Scenario: Archive extraction skip_paths support
- **WHEN** the backend `extract_archive` command receives a `skip_paths` parameter
- **THEN** the extraction skips all entries whose relative paths are in the skip set
- **AND** skipped files are not written to the destination

## Unchanged Requirements

All other requirements in `archive-operations` remain unchanged:
- Mark archive for extract-to-path (`E` + `p`) — still extracts to user-navigated target directory (flat)
- Extract selected files from archive (`x` key) — still extracts to archive parent directory
- Password prompt behavior
- Delete/rename entries in ZIP
- Compress selected files
- E/y/x mutual exclusion
- C mark for compression
- Create files/directories inside ZIP