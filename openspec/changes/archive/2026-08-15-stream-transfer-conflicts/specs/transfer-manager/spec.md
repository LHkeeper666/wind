## ADDED Requirements

### Requirement: Streaming conflict resolution
The system SHALL detect directory copy/move conflicts incrementally during a background scan, prompt the user on each conflict as it is found, and support applying a choice to all remaining conflicts.

#### Scenario: Prompt on first conflict without waiting for full scan
- **WHEN** user pastes (copies/moves) a directory into a target with conflicting files
- **THEN** the system scans in the background and prompts the user as soon as the first conflict is found
- **AND** the user does not have to wait for the entire directory tree to be scanned

#### Scenario: Apply to all conflicts
- **WHEN** user chooses "overwrite all" or "skip all" on a conflict prompt
- **THEN** the system applies that choice to all remaining conflicts without prompting again

#### Scenario: Apply to current file only
- **WHEN** user chooses "overwrite" or "skip" (without "all") on a conflict prompt
- **THEN** the system applies the choice only to the current file and prompts for the next conflict

#### Scenario: Skip conflicts still copies non-conflicting files
- **WHEN** user skips conflicting files during a directory copy/move
- **THEN** the system copies the non-conflicting files and skips the conflicting ones, reusing the existing skip list mechanism

#### Scenario: FTP transfers use streaming conflict prompts
- **WHEN** user pastes into or from an FTP target with conflicting files (upload or download)
- **THEN** the system scans and prompts on each conflict incrementally, with the same apply-to-all behavior as local transfers

#### Scenario: Loading indicator during conflict scan
- **WHEN** user presses paste and the system starts scanning for conflicts
- **THEN** the system shows a loading indicator ("checking conflicts…") until the scan completes and the transfer begins
