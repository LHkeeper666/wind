## ADDED Requirements

### Requirement: Local rename preserves requested letter casing
The system SHALL rename a local file or directory to the requested name when that name differs from the source only by letter casing and no other directory entry occupies the destination. A successful result SHALL leave the requested casing visible in the parent directory.

#### Scenario: Rename directory to lowercase
- **WHEN** a user renames a local directory `Foo` to `foo` in a case-insensitive directory
- **THEN** the operation succeeds and the directory entry is named `foo`

#### Scenario: Rename file to uppercase
- **WHEN** a user renames a local file `report.txt` to `REPORT.txt` in a case-insensitive directory
- **THEN** the operation succeeds and the file entry is named `REPORT.txt`

#### Scenario: Batch case-only rename
- **WHEN** a batch rename changes a local entry only by letter casing
- **THEN** that entry is renamed with the requested casing and is counted as successful

### Requirement: Local rename protects occupied destinations
The system SHALL reject a rename whose destination is occupied by a different directory entry and SHALL preserve both entries when rejecting it. It SHALL treat an unchanged source name as a no-op.

#### Scenario: Distinct target in case-sensitive directory
- **WHEN** a case-sensitive directory contains both `Foo` and `foo` and the user renames `Foo` to `foo`
- **THEN** the operation reports a destination conflict and neither entry is changed

#### Scenario: Distinct hard link target
- **WHEN** the requested destination is a distinct existing directory entry that refers to the same file object as the source
- **THEN** the operation reports a destination conflict and preserves both entries

#### Scenario: Unchanged name
- **WHEN** the requested name exactly matches the source entry name
- **THEN** the operation succeeds without changing the entry

### Requirement: Local rename reports incomplete casing changes
The system SHALL report success only when the requested name is present after the operation. If a temporary-name fallback cannot finish, it SHALL attempt to restore the original name and SHALL report the temporary path if restoration fails.

#### Scenario: Requested casing not applied by direct rename
- **WHEN** a direct case-only rename returns without storing the requested casing
- **THEN** the system completes the rename through a same-directory temporary name or reports a failure

#### Scenario: Fallback recovery fails
- **WHEN** the final rename from a temporary name fails and restoring the original name also fails
- **THEN** the operation reports failure and includes the temporary path for recovery
