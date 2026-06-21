## ADDED Requirements

### Requirement: Default panel layout
The system SHALL display a default layout with file tree on the left, main content area in the center, and preview panel on the right, with terminal at the bottom.

#### Scenario: Application start layout
- **WHEN** Wind starts
- **THEN** the layout shows: file tree (left), editor/preview (center), preview (right), terminal (bottom)

### Requirement: Panel focus management
The system SHALL support vim-style panel switching.

#### Scenario: Cycle focus with Ctrl+h/j/k/l
- **WHEN** user presses Ctrl+h
- **THEN** focus moves to the panel on the left

#### Scenario: Cycle focus with Ctrl+w
- **WHEN** user presses Ctrl+w followed by h/j/k/l
- **THEN** focus moves to the corresponding panel

#### Scenario: Visual focus indicator
- **WHEN** a panel receives focus
- **THEN** the panel border highlights to indicate active focus

### Requirement: Panel resize
The system SHALL allow panels to be resized via keyboard and mouse.

#### Scenario: Resize with keyboard
- **WHEN** user presses Ctrl+w followed by +/- while focused on a panel
- **THEN** the panel size increases/decreases

#### Scenario: Resize with mouse drag
- **WHEN** user drags the border between two panels
- **THEN** the adjacent panels resize accordingly

### Requirement: Path synchronization
The system SHALL synchronize the current directory across panels.

#### Scenario: File tree to terminal sync
- **WHEN** user navigates to a directory in the file tree and activates sync
- **THEN** the terminal's working directory changes to that directory

#### Scenario: Editor to file tree sync
- **WHEN** user opens a file in Neovim (BufEnter)
- **THEN** the file tree highlights and scrolls to the corresponding file

### Requirement: Editor-preview live sync
The system SHALL synchronize the editor and preview panel for supported file types.

#### Scenario: Markdown live preview
- **WHEN** user edits a Markdown file in Neovim
- **THEN** the preview panel updates in real-time to reflect the changes

#### Scenario: Preview scroll sync
- **WHEN** user scrolls in the Neovim editor
- **THEN** the preview panel scrolls to the corresponding position

### Requirement: Terminal panel toggle
The system SHALL allow the terminal panel to be shown/hidden.

#### Scenario: Toggle terminal visibility
- **WHEN** user presses the terminal toggle shortcut (e.g., Ctrl+`)
- **THEN** the terminal panel shows or hides while preserving its state
