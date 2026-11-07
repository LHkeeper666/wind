## ADDED Requirements

### Requirement: SVG default preview classification
The unified file type detector SHALL classify `.svg` as an image for default preview routing while preserving its text-editable classification, regardless of extension case.

#### Scenario: Open uppercase SVG extension
- **WHEN** a user opens a file ending in `.SVG`
- **THEN** the file opens in graphical preview by default
- **AND** its source remains available through the existing editor action
