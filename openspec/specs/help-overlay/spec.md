## ADDED Requirements

### Requirement: Help overlay displays all keybindings and commands
The system SHALL display a full-screen overlay listing all keybindings and commands, organized by context groups (Global, Directory Panel, Tab, Terminal, Preview/Editor, Commands).

#### Scenario: Help overlay shows grouped keybindings
- **WHEN** user triggers the help overlay
- **THEN** a fixed overlay appears covering the viewport, displaying all keybinding groups with keys and descriptions

### Requirement: F1 key opens help overlay
The system SHALL open the help overlay when the user presses F1 key, unless a modal or command palette is already open.

#### Scenario: F1 opens help from any panel
- **WHEN** user presses F1 while no modal/command palette/file search is open
- **THEN** the help overlay appears

#### Scenario: F1 is blocked when command palette is open
- **WHEN** user presses F1 while the command palette is open
- **THEN** the help overlay does NOT appear

### Requirement: help command opens help overlay
The system SHALL open the help overlay when the user types `help` in the command palette and presses Enter.

#### Scenario: help command triggers overlay
- **WHEN** user types `help` in command palette and presses Enter
- **THEN** the command palette closes and the help overlay appears

### Requirement: Help overlay closes on any keypress
The system SHALL close the help overlay when the user presses any key.

#### Scenario: Any key closes help
- **WHEN** help overlay is visible and user presses any key
- **THEN** the overlay closes and focus returns to the previously focused panel

### Requirement: Help command appears in command palette
The system SHALL include a `Help` entry in the command palette's commands list.

#### Scenario: Help visible in command list
- **WHEN** user opens the command palette
- **THEN** a `Help` command appears in the list

### Requirement: Ratio presets use ratio X:Y:Z format
The system SHALL display ratio preset commands in the format `ratio X:Y:Z` (e.g., `ratio 1:2:2`, `ratio 1:1:1`) instead of `Set Ratio X:Y:Z`.

#### Scenario: Ratio preset format consistency
- **WHEN** user opens the command palette
- **THEN** ratio presets appear as `ratio 1:2:2` and `ratio 1:1:1`

### Requirement: Keybinding registry is declarative
The system SHALL define all keybindings in a single declarative registry file (`src/lib/keybindings.ts`) with structured entries containing key, context, and description.

#### Scenario: Registry structure
- **WHEN** the registry is loaded
- **THEN** it exports an array of `KeybindingGroup` objects, each with a `title` string and `items` array of `Keybinding` objects (with `key`, `context`, `description` fields)
