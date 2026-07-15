## MODIFIED Requirements

### Requirement: Clipboard paste operation
The system SHALL allow users to paste clipboard contents into the current directory, with an option to force overwrite.

#### Scenario: Force paste with P key
- **WHEN** user presses `P` (Shift+p)
- **THEN** system SHALL paste all clipboard entries, automatically overwriting any existing files without prompting

#### Scenario: Force paste preserves clipboard
- **WHEN** force paste completes for a copy operation
- **THEN** the clipboard SHALL remain unchanged

#### Scenario: Force paste clears clipboard for cut
- **WHEN** force paste completes for a cut operation
- **THEN** the clipboard SHALL be cleared
