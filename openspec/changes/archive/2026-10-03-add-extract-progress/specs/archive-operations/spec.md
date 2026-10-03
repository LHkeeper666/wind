## MODIFIED Requirements

### Requirement: Extract selected files from archive
The system SHALL allow users to extract selected files from within an archive to an external directory using the `y` (yank) then `p` (paste) workflow in archive mode. The `x` key extraction is removed.

#### Scenario: Extract via yank and paste
- **WHEN** user presses `y` on file entries in archive mode to yank them
- **AND** navigates to a target directory and presses `p`
- **THEN** the system extracts the yanked files to the target directory
- **AND** the extraction appears in the Transfer Manager with progress

#### Scenario: Extract encrypted files via yank and paste
- **WHEN** user yanks files from within an encrypted `zip`, `7z`, or `rar` archive and pastes
- **THEN** the system shows the password prompt if the backend requires credentials
- **AND** after a correct password, the extraction proceeds with progress in the Transfer Manager

#### Scenario: x key no longer extracts
- **WHEN** user presses `x` in archive mode
- **THEN** no extraction occurs
- **AND** the key is available for other purposes or ignored

### Requirement: Extract entire archive to current directory

The system SHALL extract an archive into a newly created subdirectory named after the archive file (without extension), with conflict detection when the subdirectory already contains files. The extraction SHALL be routed through the TransferScheduler with progress display in the Transfer Manager.

#### Scenario: Extract archive with e key to new subdirectory
- **WHEN** user presses `e` on an archive file in the current panel
- **THEN** the system creates a subdirectory named after the archive (without full extension) in the current directory
- **AND** after conflict detection, invokes `extract_enqueue` to schedule the extraction
- **AND** the extraction appears in the Transfer Manager with progress tracking

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
- **WHEN** user presses `e` on an encrypted `zip`, `7z`, or `rar` archive file
- **THEN** the system shows the password prompt if required
- **AND** after correct password, proceeds with subdirectory creation and extraction
- **AND** conflict detection occurs after password is resolved

#### Scenario: Subdirectory name derivation
- **WHEN** extracting an archive file
- **THEN** the subdirectory name is the archive filename with the full extension stripped
- **AND** supported extensions: `.tar.gz`, `.tgz`, `.tar`, `.zip`, `.7z`, `.rar`

#### Scenario: Archive extraction skip_paths support
- **WHEN** the backend `extract_enqueue` command receives a `skip_paths` parameter
- **THEN** the extraction skips all entries whose relative paths are in the skip set
- **AND** skipped files are not written to the destination

### Requirement: Mark archive for extract-to-path
The system SHALL allow users to mark an archive for later extraction using the `E` key, then navigate to a target directory and press `p` to extract there. The extraction SHALL be routed through the TransferScheduler.

#### Scenario: Mark archive with E key
- **WHEN** user presses `E` on an archive file
- **THEN** the system marks the archive as extract source
- **AND** shows a toast "Archive marked for extraction. Navigate to target and press p."
- **AND** clears any existing yank/cut marks

#### Scenario: Mark encrypted archive with E key
- **WHEN** user presses `E` on an encrypted `zip`, `7z`, or `rar` archive file
- **THEN** the system shows the password prompt if the backend requires credentials
- **AND** after a correct password, the archive is marked as extract source
- **AND** the password remains cached for the later `p` extraction during the same application session

#### Scenario: Extract marked archive with p key
- **WHEN** user has an archive marked for extraction and presses `p`
- **THEN** the system invokes `extract_enqueue` to schedule extraction to the current directory
- **AND** the extraction appears in the Transfer Manager with progress tracking
- **AND** clears the extract mark

#### Scenario: Extract to path uses same extraction rules
- **WHEN** extracting via `E` + `p`
- **THEN** the same subdirectory creation rules as `e` extraction apply

## REMOVED Requirements

### Requirement: Extract selected files from archive (x key)
**Reason**: The `x` key extraction in archive browser mode is removed. Users should use `y` (yank) → navigate → `p` (paste) to extract selected entries. This simplifies the keybinding surface and routes all extraction through the scheduler.
**Migration**: Use `y` to yank desired entries, navigate to target directory, press `p` to extract.