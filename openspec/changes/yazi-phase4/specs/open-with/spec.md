## ADDED Requirements

### Requirement: Open with default program
The system SHALL allow users to open files with the system default program.

#### Scenario: Open file with default program
- **WHEN** user presses `o` on a file
- **THEN** system SHALL open the file with the system default program for that file type

#### Scenario: Open directory in explorer
- **WHEN** user presses `o` on a directory
- **THEN** system SHALL open the directory in Windows Explorer

### Requirement: Open with interactive picker
The system SHALL allow users to choose which program to open a file with.

#### Scenario: Open with dialog
- **WHEN** user presses `O` (Shift+o) on a file
- **THEN** system SHALL invoke the Windows "Open With" dialog for that file
- **AND** the dialog SHALL allow the user to choose an application to open the file

#### Scenario: Open with for directory
- **WHEN** user presses `O` on a directory
- **THEN** system SHALL open the directory in Windows Explorer
