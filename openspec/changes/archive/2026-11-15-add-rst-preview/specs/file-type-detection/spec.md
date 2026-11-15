## ADDED Requirements

### Requirement: RST default preview classification
The unified file type detector SHALL classify `.rst` as text that opens in the preview pane by default, regardless of extension case, while keeping its source editable through the existing editor action.

#### Scenario: Open RST file
- **WHEN** a user opens `guide.rst` or `guide.RST`
- **THEN** the file opens in formatted preview by default
- **AND** the source remains available through the editor action
