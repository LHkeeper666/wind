## Purpose

Define how directory lists and project trees reveal complete filenames within narrow columns.

## Requirements

### Requirement: Selected entry name is fully visible
The system SHALL wrap the complete name of the selected directory entry within its list or tree row, without requiring a column resize or opening file information.

#### Scenario: Navigate ordinary directory entries
- **WHEN** the user selects an entry in the ordinary directory list
- **THEN** that row expands to display the complete filename or directory name

#### Scenario: Navigate project tree entries
- **WHEN** the user selects an entry in project tree mode
- **THEN** that tree row expands to display the complete name

#### Scenario: Selection changes or becomes invalid
- **WHEN** navigation, filtering, or a view change selects a different entry
- **THEN** the newly selected row displays its complete name and the previously selected row returns to compact display
- **WHEN** there is no valid selected entry
- **THEN** no row remains expanded due to stale selection

### Requirement: Selected name fits the available column
The system SHALL wrap the selected name within the available row width, including long segments without spaces.

#### Scenario: Long selected name
- **WHEN** the selected name exceeds the available width
- **THEN** the complete name is displayed in the selected row without widening the panel

### Requirement: Hover reveals complete names
The system SHALL expose the complete name of an entry when the pointer hovers over its name in the ordinary directory list or project tree.

#### Scenario: Hover a truncated list name
- **WHEN** the user hovers over a truncated ordinary list filename
- **THEN** the complete filename is available in a tooltip

#### Scenario: Hover a truncated tree name
- **WHEN** the user hovers over a truncated project tree filename
- **THEN** the complete filename is available in a tooltip

### Requirement: Unselected rows remain compact
The system SHALL preserve single-line ellipsis for unselected directory and project tree rows.

#### Scenario: Keyboard navigation through long names
- **WHEN** the user moves selection across entries with long names
- **THEN** only the selected row wraps, other rows remain single-line, and column ratios do not change
