## ADDED Requirements

### Requirement: Zip archive structure preview
The system SHALL display the entry structure of a .zip archive in the preview panel when the archive file is selected.

#### Scenario: Preview a zip file
- **WHEN** user selects a .zip file in the current directory panel
- **THEN** the preview panel displays a header with the archive filename, entry count, and total uncompressed size, followed by a tree listing of entries with names, types, and sizes

#### Scenario: Archive with nested directories
- **WHEN** a zip file contains nested directory paths (e.g., `src/main.rs`)
- **THEN** the preview displays entries preserving the path structure, with directories shown before files at each level

#### Scenario: Empty archive
- **WHEN** user selects an empty .zip file
- **THEN** the preview panel displays the archive header with 0 entries

### Requirement: Archive entry size display
The system SHALL display entry sizes in human-readable format (B, KB, MB, GB).

#### Scenario: Entry size formatting
- **WHEN** an archive entry is 2048 bytes uncompressed
- **THEN** the preview displays "2.0 KB"

### Requirement: Archive preview styling
The system SHALL render archive entries with a distinct header and visual indicators for directories vs files.

#### Scenario: Visual distinction
- **WHEN** the preview panel shows archive contents
- **THEN** the header shows a package icon (`📦`), directories show folder icons, and files show file icons
