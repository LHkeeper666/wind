## ADDED Requirements

### Requirement: Hidden file detection
The system SHALL detect hidden files and directories.

#### Scenario: Windows hidden attribute
- **WHEN** a file has the FILE_ATTRIBUTE_HIDDEN attribute on Windows
- **THEN** the FileEntry SHALL have is_hidden set to true

#### Scenario: Dot prefix files
- **WHEN** a file name starts with a dot (.)
- **THEN** the FileEntry SHALL have is_hidden set to true

### Requirement: Hidden files filter
The system SHALL allow users to toggle visibility of hidden files.

#### Scenario: Default hidden files hidden
- **WHEN** the application starts
- **THEN** hidden files SHALL NOT be displayed in directory listings

#### Scenario: Toggle hidden files with period key
- **WHEN** user presses `.` in the directory panel
- **THEN** the system SHALL toggle the display of hidden files

#### Scenario: Hidden files shown state persisted per tab
- **WHEN** user toggles hidden files in one tab
- **THEN** other tabs SHALL NOT be affected

#### Scenario: Hidden file visual indicator
- **WHEN** hidden files are displayed
- **THEN** they SHALL appear with reduced opacity or distinct styling
