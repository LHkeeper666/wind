## ADDED Requirements

### Requirement: Markdown preview enters editor with stable visible content

When a Markdown file is already loaded in preview mode and the user enters Vim editor mode with `e`, the system SHALL initialize or activate CodeMirror only against the loaded Markdown content and SHALL ensure the visible editor viewport is measured after the editor container has a stable non-zero size.

#### Scenario: Edit Markdown from preview in packaged build

- **WHEN** the user previews a loaded `readme.md`
- **AND** presses `e` in the preview panel
- **THEN** the editor switches to `editor-normal`
- **AND** CodeMirror displays the Markdown document content immediately
- **AND** the editor viewport does not show blank or partially painted regions

#### Scenario: Edit Markdown after layout changes

- **WHEN** the user changes layout state by switching tabs, expanding/collapsing preview, toggling/resizing the terminal, or returning focus to the app
- **AND** previews a loaded Markdown file
- **AND** presses `e`
- **THEN** the editor measures its visible container before applying initial scroll restoration
- **AND** the Markdown content is visible without requiring insert-mode input to repaint

#### Scenario: Preserve target line from preview

- **WHEN** the user scrolls a Markdown preview so a later section is visible
- **AND** presses `e`
- **THEN** the editor opens near the corresponding Markdown source line after layout measurement completes
- **AND** no stale preview or old editor session content is displayed

### Requirement: Markdown list Tab indentation preserves markers

When a Markdown list item is edited in insert mode, pressing `Tab` SHALL indent the whole list item line, including the `-`, `*`, `+`, `1.`, or task-list marker, rather than inserting spaces only into the list item's content.

#### Scenario: Tab on unordered Markdown list item content

- **WHEN** the cursor is inside the content of a line beginning with `- item`
- **AND** the editor is in insert mode
- **AND** the user presses `Tab`
- **THEN** the line becomes `    - item`
- **AND** spaces are not inserted between the marker and `item`

#### Scenario: Tab on ordered Markdown list item content

- **WHEN** the cursor is inside the content of a line beginning with `1. item`
- **AND** the editor is in insert mode
- **AND** the user presses `Tab`
- **THEN** the line becomes `    1. item`
- **AND** spaces are not inserted between the marker and `item`
