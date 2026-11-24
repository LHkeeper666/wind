## ADDED Requirements

### Requirement: Unicode archive paths remain safe during browsing
The system SHALL resolve archive paths and list supported archives without a backend panic caused by non-ASCII characters in the path.

#### Scenario: Preview a 7z archive in a Unicode path
- **WHEN** a user previews a `.7z` archive whose directory or filename contains non-ASCII characters
- **THEN** archive listing proceeds to the archive reader or returns a normal archive or password error
- **AND** the backend does not return a task-join panic caused by an invalid character boundary

#### Scenario: Browse other supported archive formats in Unicode paths
- **WHEN** a user enters a supported ZIP, TAR, TAR.GZ, or RAR archive whose path contains non-ASCII characters
- **THEN** password resolution and listing complete or return a normal format-specific error without a character-boundary panic

### Requirement: RAR volume format recognition is consistent
The system SHALL recognize `.rar`, two-digit `.rXX`, and `.partN.rar` filename suffixes with ASCII digits and case-insensitive ASCII extensions consistently wherever archive format and password-cache grouping are determined.

#### Scenario: Valid RAR volume suffix
- **WHEN** a filename ends in `.r00`, `.R99`, `.part2.rar`, or `.PART12.RAR`
- **THEN** the backend treats it as a RAR volume in both format detection and password-cache grouping

#### Scenario: Misleading or incomplete suffix
- **WHEN** a filename contains `.part` before a different ending or ends in `.partX.rar` or `.r0`
- **THEN** the backend does not classify that suffix as a RAR volume
