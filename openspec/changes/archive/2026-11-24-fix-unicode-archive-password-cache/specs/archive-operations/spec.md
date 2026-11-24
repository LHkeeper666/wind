## ADDED Requirements

### Requirement: Archive password resolution supports Unicode paths
The system SHALL safely resolve and cache archive passwords for supported archive formats when the archive path contains non-ASCII characters, including for extraction operations.

#### Scenario: Extract encrypted archive at a Unicode path
- **WHEN** a user supplies a correct password while extracting an encrypted ZIP, 7z, or RAR archive at a Unicode path
- **THEN** password resolution proceeds without a character-boundary panic
- **AND** extraction proceeds or returns a normal archive-specific error

#### Scenario: Share password between RAR volumes
- **WHEN** a password has been cached for one volume of a `.partN.rar` or `.rXX` RAR set
- **THEN** another valid volume of that set resolves the same cached password through the first-volume key
- **AND** the filename prefix is preserved even when it contains non-ASCII characters

#### Scenario: Preserve padded RAR volume numbering
- **WHEN** a RAR volume is named with padded numbering such as `.part02.rar`
- **THEN** its first-volume cache key refers to `.part01.rar`
