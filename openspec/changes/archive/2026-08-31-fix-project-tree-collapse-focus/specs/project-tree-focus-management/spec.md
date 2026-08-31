## ADDED Requirements

### Requirement: Collapsed directory keeps selection visible
Project mode file tree SHALL ensure that the selected item remains visible after a directory is collapsed.

#### Scenario: Collapse parent directory of selected file
- **WHEN** the current selection is a file inside a directory and the user collapses that directory
- **THEN** the system selects the collapsed directory
- **AND** the system moves file tree focus to the collapsed directory
- **AND** the preview target is updated to the collapsed directory

#### Scenario: Collapse ancestor directory of selected file
- **WHEN** the current selection is a file inside a nested directory and the user collapses any visible ancestor directory
- **THEN** the system selects the ancestor directory that was collapsed
- **AND** the system moves file tree focus to that collapsed directory
- **AND** the preview target is updated to that collapsed directory

### Requirement: Unrelated directory collapse preserves current context
Project mode file tree SHALL preserve the current selection, focus, and preview when the collapsed directory does not contain the current selection.

#### Scenario: Collapse unrelated directory
- **WHEN** the current selection is outside the directory being collapsed
- **THEN** the current selection remains unchanged
- **AND** the current file tree focus remains unchanged
- **AND** the preview target remains unchanged

### Requirement: Directory self-collapse keeps directory selected
Project mode file tree SHALL keep a selected directory as the active context when that same directory is collapsed.

#### Scenario: Collapse selected directory
- **WHEN** the current selection is a directory and the user collapses that same directory
- **THEN** the selected directory remains selected
- **AND** the system keeps file tree focus on that directory
- **AND** the preview target remains that directory

### Requirement: Collapse handling uses shared state correction
Project mode file tree SHALL use one shared collapse visibility correction path for all directory collapse entry points.

#### Scenario: Mouse collapse uses shared correction
- **WHEN** the user collapses a directory with the mouse
- **THEN** the system applies the shared collapse visibility correction behavior

#### Scenario: Keyboard collapse uses shared correction
- **WHEN** the user collapses a directory with a keyboard interaction
- **THEN** the system applies the shared collapse visibility correction behavior

#### Scenario: Programmatic collapse uses shared correction
- **WHEN** project mode collapses a directory through a programmatic action
- **THEN** the system applies the shared collapse visibility correction behavior
