## ADDED Requirements

### Requirement: Sort by name
The system SHALL support sorting files by name (default).

#### Scenario: Sort by name
- **WHEN** user presses `sn` (s prefix + n)
- **THEN** system SHALL sort files alphabetically by name (directories first if enabled)
- **AND** system SHALL show a toast "Sorted by name"

### Requirement: Sort by size
The system SHALL support sorting files by size.

#### Scenario: Sort by size
- **WHEN** user presses `ss` (s prefix + s)
- **THEN** system SHALL sort files by size (smallest first, directories treated as 0)
- **AND** system SHALL show a toast "Sorted by size"

### Requirement: Sort by extension
The system SHALL support sorting files by extension.

#### Scenario: Sort by extension
- **WHEN** user presses `se` (s prefix + e)
- **THEN** system SHALL sort files alphabetically by extension
- **AND** system SHALL show a toast "Sorted by extension"

### Requirement: Reverse sort order
The system SHALL support reversing the current sort order.

#### Scenario: Reverse sort
- **WHEN** user presses `sr` (s prefix + r)
- **THEN** system SHALL reverse the current sort direction
- **AND** system SHALL show a toast "Sort reversed"

### Requirement: Directory-first toggle
The system SHALL support toggling directory-first sorting.

#### Scenario: Toggle directory first
- **WHEN** user presses `st` (s prefix + t)
- **THEN** system SHALL toggle between directories-first and mixed sorting
- **AND** system SHALL show a toast indicating the new mode

### Requirement: Sort state persistence
The system SHALL maintain sort settings across directory navigation.

#### Scenario: Sort persists on navigate
- **WHEN** user changes sort settings and then navigates to a different directory
- **THEN** the new directory SHALL use the same sort settings
