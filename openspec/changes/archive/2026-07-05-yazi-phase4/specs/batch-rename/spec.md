## ADDED Requirements

### Requirement: Batch rename via editor
The system SHALL allow users to rename multiple files at once by editing a list of filenames in Neovim.

#### Scenario: Initiate batch rename with r key
- **WHEN** user has multiple files selected (via Space/v) and presses `r`
- **THEN** system SHALL write the selected filenames to a temporary file (one per line)
- **AND** system SHALL open Neovim in the floating terminal with the temporary file

#### Scenario: Batch rename execution
- **WHEN** user saves and closes the Neovim editor
- **THEN** system SHALL parse the new filenames from the temporary file
- **AND** system SHALL rename each file that has a changed name
- **AND** system SHALL refresh the directory listing
- **AND** system SHALL show a toast with the count of renamed files

#### Scenario: Batch rename count mismatch
- **WHEN** the number of lines in the edited file does not match the number of selected files
- **THEN** system SHALL show an error toast and cancel the operation

#### Scenario: Batch rename conflict
- **WHEN** a new filename already exists in the directory
- **THEN** system SHALL skip that file and show a warning toast
- **AND** system SHALL continue renaming the remaining files

#### Scenario: Neovim not available
- **WHEN** Neovim is not installed or not accessible
- **THEN** system SHALL show an error toast "Neovim not available for batch rename"

### Requirement: Unified r key for rename
The system SHALL use `r` for both single-file rename and multi-file batch rename.

#### Scenario: Single file rename
- **WHEN** user has no multi-selection and presses `r` on a file
- **THEN** system SHALL show the InputDialog for renaming (existing behavior)

#### Scenario: Multi-file batch rename
- **WHEN** user has multiple files selected and presses `r`
- **THEN** system SHALL initiate batch rename via Neovim (not InputDialog)
