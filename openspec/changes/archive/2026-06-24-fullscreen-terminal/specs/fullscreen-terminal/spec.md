## ADDED Requirements

### Requirement: Ctrl+Shift+` toggles fullscreen terminal
The system SHALL toggle terminal fullscreen mode when the user presses Ctrl+Shift+`.

#### Scenario: Enter fullscreen mode
- **WHEN** the user presses Ctrl+Shift+` and the terminal is not in fullscreen
- **THEN** the terminal enters fullscreen mode, covering 100% of the window

#### Scenario: Exit fullscreen mode
- **WHEN** the user presses Ctrl+Shift+` and the terminal is in fullscreen
- **THEN** the terminal exits fullscreen mode, restoring the previous layout and height

#### Scenario: Open terminal in fullscreen when hidden
- **WHEN** the terminal is hidden and the user presses Ctrl+Shift+`
- **THEN** the terminal shows and enters fullscreen mode directly

### Requirement: Ctrl+` closes terminal in fullscreen mode
The system SHALL close the terminal when the user presses Ctrl+` while in fullscreen mode.

#### Scenario: Close terminal from fullscreen
- **WHEN** the user presses Ctrl+` while in fullscreen terminal
- **THEN** the terminal exits fullscreen and hides, focus returns to the current panel

### Requirement: Panel switching disabled in fullscreen
The system SHALL disable Ctrl+W panel switching when the terminal is in fullscreen mode.

#### Scenario: Panel switch attempt in fullscreen
- **WHEN** the user presses Ctrl+W followed by h/l/j/k while in fullscreen terminal
- **THEN** no panel switching operation is executed

### Requirement: Mode preserved during fullscreen toggle
The system SHALL preserve the terminal's insert/normal mode state when toggling fullscreen.

#### Scenario: Insert mode preserved
- **WHEN** the terminal is in insert mode and enters fullscreen
- **THEN** it remains in insert mode after entering fullscreen

#### Scenario: Normal mode preserved
- **WHEN** the terminal is in normal mode and enters fullscreen
- **THEN** it remains in normal mode after entering fullscreen

### Requirement: Terminal functionality preserved in fullscreen
The system SHALL preserve all terminal functionality in fullscreen mode.

#### Scenario: All features work in fullscreen
- **WHEN** the terminal is in fullscreen mode
- **THEN** shell selector (Bash/Pwsh/CMD) remains usable
- **THEN** insert/normal mode switching (Escape/i) remains functional
- **THEN** terminal input and output work normally
