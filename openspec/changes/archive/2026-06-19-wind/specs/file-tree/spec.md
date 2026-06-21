## ADDED Requirements

### Requirement: File tree displays directory structure
The system SHALL display a hierarchical file tree view of the opened directory, showing files and folders with proper indentation.

#### Scenario: Open a directory
- **WHEN** user opens a directory in Wind
- **THEN** the file tree displays all files and folders in the root directory, with folders expandable

#### Scenario: Expand a folder
- **WHEN** user presses Enter or clicks on a folder in the file tree
- **THEN** the folder expands to show its children, indented one level deeper

#### Scenario: Collapse a folder
- **WHEN** user presses h or clicks on an expanded folder
- **THEN** the folder collapses, hiding its children

### Requirement: File tree vim key navigation
The system SHALL support vim-style keyboard navigation in the file tree.

#### Scenario: Navigate down
- **WHEN** user presses j in the file tree
- **THEN** the selection moves to the next file/folder

#### Scenario: Navigate up
- **WHEN** user presses k in the file tree
- **THEN** the selection moves to the previous file/folder

#### Scenario: Jump to top
- **WHEN** user presses gg in the file tree
- **THEN** the selection moves to the first item

#### Scenario: Jump to bottom
- **WHEN** user presses G in the file tree
- **THEN** the selection moves to the last item

#### Scenario: Open file in editor
- **WHEN** user presses Enter on a file in the file tree
- **THEN** the file opens in the Neovim editor

#### Scenario: Open directory in terminal
- **WHEN** user presses Enter on a folder in the file tree
- **THEN** the terminal's working directory changes to that folder

### Requirement: File tree mouse support
The system SHALL support mouse interactions in the file tree.

#### Scenario: Click to select
- **WHEN** user clicks on a file/folder in the file tree
- **THEN** that item becomes selected

#### Scenario: Double-click to open
- **WHEN** user double-clicks a file in the file tree
- **THEN** the file opens in the Neovim editor

#### Scenario: Scroll with mouse wheel
- **WHEN** user scrolls the mouse wheel over the file tree
- **THEN** the file tree scrolls vertically

### Requirement: File tree search and filter
The system SHALL provide file search and filtering capabilities.

#### Scenario: Activate search
- **WHEN** user presses / in the file tree
- **THEN** a search input appears at the bottom of the file tree

#### Scenario: Filter files
- **WHEN** user types a search query in the file tree search
- **THEN** only files/folders matching the query are shown

#### Scenario: Clear search
- **WHEN** user presses Escape while in the file tree search
- **THEN** the search is cleared and all files are shown again

### Requirement: File operations
The system SHALL support basic file operations through the file tree.

#### Scenario: Delete a file
- **WHEN** user presses dd on a file in the file tree
- **THEN** the system prompts for confirmation, then deletes the file

#### Scenario: Rename a file
- **WHEN** user presses r on a file in the file tree
- **THEN** the system enters rename mode, allowing the user to type a new name

#### Scenario: Create a new file
- **WHEN** user presses o in the file tree
- **THEN** the system prompts for a filename and creates the file in the current directory

#### Scenario: Copy a file
- **WHEN** user presses yy on a file in the file tree
- **THEN** the file path is copied to the internal clipboard

#### Scenario: Paste a file
- **WHEN** user presses p in the file tree
- **THEN** the copied file is pasted into the current directory
