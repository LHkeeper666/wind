## ADDED Requirements

### Requirement: Directory content preview
The system SHALL display a directory's contents (one level) in the preview panel when the directory is selected via j/k navigation.

#### Scenario: Preview a directory with files and subdirectories
- **WHEN** user selects a directory in the current directory panel using j/k navigation
- **THEN** the preview panel displays a list of entries showing subdirectories first, then files, sorted alphabetically, each with name and file size (directories show no size)

#### Scenario: Preview an empty directory
- **WHEN** user selects an empty directory
- **THEN** the preview panel displays an empty state message

#### Scenario: Enter still navigates into directory
- **WHEN** user presses Enter on a directory in the current directory panel
- **THEN** the system navigates into that directory (existing behavior unchanged)

### Requirement: File size display
The system SHALL display file sizes in human-readable format (B, KB, MB, GB).

#### Scenario: File size formatting
- **WHEN** a file is 1536 bytes
- **THEN** the preview displays "1.5 KB"

#### Scenario: Directory entry has no size
- **WHEN** a directory entry is displayed
- **THEN** the size column is empty or hidden for that entry

### Requirement: Directory preview styling
The system SHALL render directory entries with visual indicators distinguishing directories from files.

#### Scenario: Visual distinction
- **WHEN** the preview panel shows directory contents
- **THEN** directories show a folder icon and files show a file icon, matching the existing panel aesthetic
