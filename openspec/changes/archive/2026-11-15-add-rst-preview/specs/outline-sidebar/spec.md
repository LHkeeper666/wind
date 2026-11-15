## ADDED Requirements

### Requirement: RST sections in shared outline
The shared document outline SHALL display RST section headings in document order and hierarchy, using the existing outline navigation and per-tab state behavior. Selecting a heading SHALL navigate to its source line in the preview.

#### Scenario: Navigate RST sections
- **WHEN** an RST preview contains headings with two adornment styles
- **THEN** the outline displays the corresponding parent and child levels
- **AND** selecting a heading navigates to that heading in the preview

#### Scenario: Switch between document types
- **WHEN** a user switches from an RST tab to a Markdown or PDF tab
- **THEN** the outline shows headings or bookmarks for the active tab only
