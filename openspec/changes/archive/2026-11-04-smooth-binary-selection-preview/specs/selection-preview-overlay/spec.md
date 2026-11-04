## ADDED Requirements

### Requirement: Binary selection preview keeps rendering work bounded
The selection preview SHALL render a bounded number of hex rows at a time, including during scrolling, while retaining navigable offsets and the ASCII column for the first 64 KiB of binary content.

#### Scenario: Selecting a torrent file
- **WHEN** the user selects a binary `.torrent` file in the current panel
- **THEN** the preview shows hex content without creating a DOM row for every byte range in the 64 KiB preview
- **AND** the current panel remains responsive to further selection changes

#### Scenario: Scrolling through hex content
- **WHEN** the user scrolls the binary selection preview
- **THEN** the visible rows show the correct byte offsets and ASCII values
- **AND** the number of mounted hex rows remains bounded

### Requirement: Binary preview reports source file size
The selection preview SHALL report the source file size and a truncation notice when only part of a binary file is shown.

#### Scenario: Binary file exceeds displayed bytes
- **WHEN** a binary file larger than 64 KiB is selected
- **THEN** the notice reports the source file size and that only the first 64 KiB is displayed

#### Scenario: Selection changes during loading
- **WHEN** the user selects another item while a binary preview is loading or rendering
- **THEN** the old preview does not replace the new selection and its rendering resources are released
