## ADDED Requirements

### Requirement: Filter files by pattern
The system SHALL allow users to filter the file list by glob pattern.

#### Scenario: Apply filter
- **WHEN** user presses `f` in the directory panel
- **THEN** system SHALL show an input dialog with "Filter:" prompt
- **WHEN** user enters a glob pattern (e.g., `*.txt`) and presses Enter
- **THEN** system SHALL filter the file list to show only matching files (and `..` entry)
- **AND** system SHALL show a toast "Filter: pattern"

#### Scenario: Filter with wildcards
- **WHEN** user applies a filter pattern with `*` or `?`
- **THEN** the system SHALL match filenames against the pattern using glob rules
- **AND** `*` SHALL match any sequence of characters
- **AND** `?` SHALL match any single character

#### Scenario: Clear filter
- **WHEN** user presses `f` and enters an empty pattern (or presses Escape)
- **THEN** system SHALL clear the filter and show all files
- **AND** system SHALL show a toast "Filter cleared"

#### Scenario: Filter state indicator
- **WHEN** a filter is active
- **THEN** the panel header SHALL display the active filter pattern

### Requirement: Filter with sort
The system SHALL apply filter before sort.

#### Scenario: Filter and sort interaction
- **WHEN** both filter and sort are active
- **THEN** system SHALL first filter files by pattern, then sort the filtered results
