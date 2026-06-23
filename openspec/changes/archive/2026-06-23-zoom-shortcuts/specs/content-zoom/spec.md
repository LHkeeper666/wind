## ADDED Requirements

### Requirement: Zoom in via keyboard shortcut
The system SHALL increase the zoom level of panel content by 10% when the user presses Ctrl+=.

#### Scenario: Zoom in from default
- **WHEN** the user presses Ctrl+= and the current zoom level is 1.0
- **THEN** the zoom level changes to 1.1 and all `.panel` elements scale up accordingly
- **AND** a toast displays "Zoom: 110%"

### Requirement: Zoom out via keyboard shortcut
The system SHALL decrease the zoom level of panel content by 10% when the user presses Ctrl+-.

#### Scenario: Zoom out from default
- **WHEN** the user presses Ctrl+- and the current zoom level is 1.0
- **THEN** the zoom level changes to 0.9 and all `.panel` elements scale down accordingly
- **AND** a toast displays "Zoom: 90%"

### Requirement: Reset zoom via keyboard shortcut
The system SHALL reset the zoom level to 100% when the user presses Ctrl+0.

#### Scenario: Reset zoom from non-default
- **WHEN** the user presses Ctrl+0 and the current zoom level is 1.2
- **THEN** the zoom level resets to 1.0
- **AND** a toast displays "Zoom: 100%"

### Requirement: Zoom level bounds
The system SHALL constrain the zoom level between 0.5 (50%) and 2.0 (200%) inclusive.

#### Scenario: Zoom in at maximum
- **WHEN** the user presses Ctrl+= and the current zoom level is 2.0
- **THEN** the zoom level remains at 2.0

#### Scenario: Zoom out at minimum
- **WHEN** the user presses Ctrl+- and the current zoom level is 0.5
- **THEN** the zoom level remains at 0.5

### Requirement: Zoom does not affect layout
The system SHALL apply zoom only to `.panel` content areas and SHALL NOT affect column widths, resize handles, or the status bar.

#### Scenario: Column widths unchanged after zoom
- **WHEN** the user changes the zoom level
- **THEN** the three-column grid layout maintains its original column width ratios

### Requirement: Zoom via mouse wheel
The system SHALL adjust zoom level when the user scrolls the mouse wheel while holding Ctrl.

#### Scenario: Scroll up zooms in
- **WHEN** the user scrolls up while holding Ctrl and the current zoom level is 1.0
- **THEN** the zoom level changes to 1.1

#### Scenario: Scroll down zooms out
- **WHEN** the user scrolls down while holding Ctrl and the current zoom level is 1.0
- **THEN** the zoom level changes to 0.9

### Requirement: Zoom applies to all panels
The system SHALL apply the same zoom level to parent directory panel, current directory panel, and preview/editor panel simultaneously.

#### Scenario: Multiple panels share zoom level
- **WHEN** the user changes the zoom level
- **THEN** all three panels (.parent-panel, .current-panel, .preview-panel) reflect the same zoom level
