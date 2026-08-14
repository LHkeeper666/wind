## MODIFIED Requirements

### Requirement: Clipboard paste operation
The system SHALL allow users to paste clipboard contents into the current directory, routing to the appropriate backend based on source and destination path schemes.

#### Scenario: Paste copy operation
- **WHEN** user presses `p` with clipboard operation `copy`
- **THEN** system copies each file from clipboard to the current directory, routing to local copy, ftp_upload, or ftp_download as appropriate

#### Scenario: Paste cut operation
- **WHEN** user presses `p` with clipboard operation `cut`
- **THEN** system moves each file from clipboard to the current directory (copy + delete source), routing through the appropriate backend, and clears the cut visual indicators

#### Scenario: Paste clears clipboard
- **WHEN** paste operation completes (all files processed or aborted)
- **THEN** system clears the clipboard and removes all cut visual indicators

#### Scenario: Paste with empty clipboard
- **WHEN** user presses `p` with empty clipboard
- **THEN** system shows toast "Clipboard empty" and does nothing

#### Scenario: Force paste without conflict prompt
- **WHEN** user presses `P` (Shift+p) with clipboard entries
- **THEN** system pastes all files, overwriting any conflicts without prompting
