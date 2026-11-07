## ADDED Requirements

### Requirement: SVG defaults to graphical preview
The system SHALL display a selected local `.svg` file as a scaled image in the preview pane and SHALL preserve an entry to edit its XML source.

#### Scenario: Open SVG file
- **WHEN** a user opens a valid local `.svg` file
- **THEN** the preview pane displays the rendered graphic
- **AND** the user can enter source editing through the existing edit action

#### Scenario: Select SVG file
- **WHEN** a user selects a valid local `.svg` file without opening it
- **THEN** the selection preview displays the rendered graphic

#### Scenario: Invalid SVG file
- **WHEN** a selected `.svg` file cannot be decoded as an image
- **THEN** the preview pane displays a clear load failure state

### Requirement: SVG preview uses image isolation
The system SHALL render SVG file bytes as an image resource and SHALL NOT insert untrusted SVG source directly into the preview DOM.

#### Scenario: SVG contains active markup
- **WHEN** an SVG source contains script or event attributes
- **THEN** preview loading treats it as an image resource rather than executable inline DOM markup

### Requirement: SVG preview resources are released
The system SHALL release temporary image URLs when their owning preview is no longer used.

#### Scenario: Replace an SVG preview
- **WHEN** a user replaces or closes an SVG preview
- **THEN** its unused temporary URL is revoked after the displayed image no longer needs it
