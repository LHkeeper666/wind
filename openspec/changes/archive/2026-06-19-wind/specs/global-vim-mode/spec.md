## ADDED Requirements

### Requirement: Global vim keybinding layer
The system SHALL implement a global vim-style keybinding layer that works across all panels.

#### Scenario: Panel navigation with vim keys
- **WHEN** user presses Ctrl+w followed by h/j/k/l
- **THEN** focus moves to the corresponding panel (left/down/up/right)

#### Scenario: Escape returns to normal mode
- **WHEN** user presses Escape in any panel
- **THEN** the panel returns to its normal/navigation mode

### Requirement: Command palette
The system SHALL provide a command palette activated by a vim-style command.

#### Scenario: Open command palette
- **WHEN** user presses : in normal mode (outside of Neovim)
- **THEN** a command palette input appears at the bottom of the window

#### Scenario: Execute command
- **WHEN** user types a command and presses Enter in the command palette
- **THEN** the command is executed (e.g., :open, :split, :terminal, :theme)

#### Scenario: Command palette fuzzy search
- **WHEN** user types in the command palette
- **THEN** available commands are filtered with fuzzy matching

### Requirement: Fuzzy file search
The system SHALL provide a fuzzy file search overlay.

#### Scenario: Open fuzzy search
- **WHEN** user presses Ctrl+p (or custom binding)
- **THEN** a fuzzy search overlay appears

#### Scenario: Search files
- **WHEN** user types a query in the fuzzy search
- **THEN** files matching the query are listed, ranked by relevance

#### Scenario: Open file from search
- **WHEN** user selects a file from fuzzy search results and presses Enter
- **THEN** the file opens in the Neovim editor

### Requirement: Status bar
The system SHALL display a status bar showing current mode, file path, and encoding.

#### Scenario: Display vim mode
- **WHEN** the Neovim editor is in normal mode
- **THEN** the status bar shows "NORMAL"

#### Scenario: Display file path
- **WHEN** a file is open in the editor
- **THEN** the status bar shows the file path relative to the project root

#### Scenario: Display encoding
- **WHEN** a file is open
- **THEN** the status bar shows the file encoding (e.g., UTF-8)

### Requirement: Theme system
The system SHALL support light and dark themes.

#### Scenario: Switch to dark theme
- **WHEN** user selects dark theme from settings or command palette
- **THEN** all panels (file tree, editor, preview, terminal) switch to dark theme

#### Scenario: Switch to light theme
- **WHEN** user selects light theme
- **THEN** all panels switch to light theme

#### Scenario: Theme persistence
- **WHEN** user selects a theme and restarts Wind
- **THEN** the selected theme is preserved
