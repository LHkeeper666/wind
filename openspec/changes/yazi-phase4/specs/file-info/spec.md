## ADDED Requirements

### Requirement: File info display
The system SHALL display detailed file information in an overlay panel.

#### Scenario: Show file info
- **WHEN** user presses `i` on a file in the directory panel
- **THEN** system SHALL display an overlay with file details
- **AND** the overlay SHALL show: filename, full path, size (formatted), creation time, modification time, access time, file type, and Windows attributes

#### Scenario: File info for directory
- **WHEN** user presses `i` on a directory
- **THEN** system SHALL display the directory info including item count

#### Scenario: Close file info
- **WHEN** file info overlay is visible and user presses any key
- **THEN** system SHALL close the overlay and restore panel focus

#### Scenario: File info data
- **WHEN** file info is requested
- **THEN** the system SHALL retrieve file metadata from the Rust backend via `get_file_info` command
- **AND** the data SHALL include: name, path, size, is_dir, created, modified, accessed, is_readonly, is_hidden, is_system

### Requirement: File size formatting
The system SHALL format file sizes in human-readable format.

#### Scenario: Size formatting
- **WHEN** file size is displayed
- **THEN** sizes SHALL be shown as B, KB, MB, GB, TB with appropriate precision (e.g., "1.5 MB")
