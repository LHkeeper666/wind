## ADDED Requirements

### Requirement: Create file
The system SHALL allow users to create new files using the `a` key.

#### Scenario: Activate create file
- **WHEN** user presses `a` followed by a file name (e.g., `a filename.txt`)
- **THEN** system SHALL show an InputDialog in create-file mode

#### Scenario: Confirm create file
- **WHEN** user enters a file name and presses Enter
- **THEN** system SHALL create the file in the current directory and refresh the listing

#### Scenario: Create selects new file
- **WHEN** file creation completes successfully
- **THEN** the new file SHALL be selected in the directory listing

### Requirement: Create directory
The system SHALL allow users to create new directories.

#### Scenario: Activate create directory
- **WHEN** user presses `a/` (a followed by slash)
- **THEN** system SHALL show an InputDialog in create-directory mode

#### Scenario: Confirm create directory
- **WHEN** user enters a directory name and presses Enter
- **THEN** system SHALL create the directory in the current directory and refresh the listing

#### Scenario: Create conflict
- **WHEN** user enters a name that already exists
- **THEN** system SHALL show an error toast "File/directory already exists"
