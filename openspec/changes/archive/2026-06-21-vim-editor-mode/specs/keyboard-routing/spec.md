## ADDED Requirements

### Requirement: Global Escape handler removal
The PanelLayout global keyboard handler SHALL NOT intercept the Escape key. Escape handling SHALL be delegated entirely to the focused component.

#### Scenario: Escape with no modal open
- **WHEN** no command palette or file search modal is open
- **AND** the user presses Escape
- **THEN** the global handler SHALL NOT process the event
- **AND** the event SHALL propagate to the focused component

#### Scenario: Escape closes command palette
- **WHEN** the command palette is open
- **AND** the user presses Escape
- **THEN** the command palette's own keyboard handler closes it

#### Scenario: Escape closes file search
- **WHEN** the file search modal is open
- **AND** the user presses Escape
- **THEN** the file search modal's own keyboard handler closes it

### Requirement: Mode-aware colon key handling
The `:` key SHALL open the command palette only when the active column is not `preview`, or when the preview column is in `global-normal` mode (not in `editor-normal` or `editor-insert`).

#### Scenario: Colon in directory panel
- **WHEN** the active column is `current` or `parent`
- **AND** the user presses `:`
- **THEN** the command palette opens

#### Scenario: Colon in editor normal mode
- **WHEN** the PreviewEditor is in `editor-normal` mode
- **AND** the user presses `:`
- **THEN** the vim command line activates (NOT the command palette)

#### Scenario: Colon in global normal mode
- **WHEN** the PreviewEditor is in `global-normal` mode
- **AND** the active column is `preview`
- **AND** the user presses `:`
- **THEN** the command palette opens

### Requirement: Non-text file edit attempt feedback
When the user presses `e` or `E` on a non-text file (image, binary), the system SHALL display a toast notification indicating the file cannot be edited.

#### Scenario: e on image file
- **WHEN** the user presses `e` on a `.png` file in `global-normal` mode
- **THEN** a toast message "此文件类型不支持编辑" is displayed
- **AND** the mode remains `global-normal`

#### Scenario: E on image file
- **WHEN** the user presses `E` on a `.png` file in `global-normal` mode
- **THEN** a toast message "此文件类型不支持编辑" is displayed
- **AND** the fullscreen editor does not open

### Requirement: E key opens fullscreen editor from directory panel
The user SHALL be able to press `E` in the current directory panel to open the fullscreen editor for the selected file.

#### Scenario: E in current directory panel
- **WHEN** the active column is `current`
- **AND** a text file is selected
- **AND** the user presses `E`
- **THEN** the fullscreen editor opens for the selected file

#### Scenario: E in current directory on non-text file
- **WHEN** the active column is `current`
- **AND** a non-text file is selected
- **AND** the user presses `E`
- **THEN** a toast message "此文件类型不支持编辑" is displayed

## REMOVED Requirements

### Requirement: Global Escape closes modals
**Reason**: Escape handling is now delegated to each component's own handler. The global capture-phase Escape handler in PanelLayout is removed to prevent conflicts with editor Escape handling.
**Migration**: Command palette and file search modals already have their own keyboard handlers that close on Escape. No functional change.

### Requirement: Enter key enters edit mode
**Reason**: Replaced by `e` key for entering Editor Normal mode. Enter in PreviewEditor is no longer used for mode switching.
**Migration**: Press `e` instead of Enter to enter the editor.

### Requirement: e key opens fullscreen editor
**Reason**: Replaced by `E` key. Lowercase `e` now enters Editor Normal mode.
**Migration**: Press `E` instead of `e` for fullscreen editor.
