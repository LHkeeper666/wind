## ADDED Requirements

### Requirement: Insert mode autocomplete uses Tab confirmation

When the Vim editor is in insert mode and an autocomplete suggestion is active, the system SHALL confirm the selected completion with `Tab` and SHALL NOT confirm autocomplete with `Enter`.

#### Scenario: Tab accepts active completion

- **WHEN** the editor is in insert mode
- **AND** the autocomplete tooltip is open with a selected suggestion
- **AND** the user presses `Tab`
- **THEN** the selected suggestion is inserted
- **AND** the key is not also processed as indentation

#### Scenario: Enter does not accept active completion

- **WHEN** the editor is in insert mode
- **AND** the autocomplete tooltip is open with a selected suggestion
- **AND** the user presses `Enter`
- **THEN** the editor inserts a newline or performs Markdown list continuation
- **AND** the selected completion is not inserted

#### Scenario: Completion navigation remains available

- **WHEN** the autocomplete tooltip is open in insert mode
- **THEN** the user can still navigate suggestions with arrow and page keys
- **AND** the user can close completion with `Escape`
- **AND** the user can trigger completion manually with `Ctrl+Space`

### Requirement: Tab indentation advances to the next tab stop

When the Vim editor is in insert mode and `Tab` is used for ordinary indentation outside Markdown list indentation, the system SHALL insert only the number of spaces needed to advance the cursor or line indentation to the next tab stop.

#### Scenario: Single cursor at column one advances to column four

- **WHEN** the editor is in insert mode
- **AND** the cursor is on a non-list line at column 1
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** the editor inserts 3 spaces
- **AND** the cursor advances to column 4

#### Scenario: Single cursor at column four advances to column eight

- **WHEN** the editor is in insert mode
- **AND** the cursor is on a non-list line at column 4
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** the editor inserts 4 spaces
- **AND** the cursor advances to column 8

#### Scenario: Selected non-list lines indent to their next tab stop

- **WHEN** the editor is in insert mode
- **AND** a selection spans one or more non-list lines
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** each selected line receives the number of leading spaces needed to advance its indentation to the next tab stop
- **AND** each selected line is not blindly prefixed with exactly 4 spaces unless it is already on a tab stop

### Requirement: Markdown list indentation keeps line-level semantics

When the Vim editor is in insert mode and `Tab` is used on Markdown list items without accepting autocomplete, the system SHALL indent the whole list item line as a list-level operation.

#### Scenario: Unordered Markdown list item indents with marker

- **WHEN** the cursor is inside the content of a line beginning with `- item`
- **AND** the editor is in insert mode
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** the line becomes `    - item`
- **AND** spaces are not inserted between the marker and `item`

#### Scenario: Ordered Markdown list item indents with marker

- **WHEN** the cursor is inside the content of a line beginning with `1. item`
- **AND** the editor is in insert mode
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** the line becomes `    1. item`
- **AND** spaces are not inserted between the marker and `item`

### Requirement: Markdown list indentation moves list trees

When the Vim editor is in insert mode and `Tab` or `Shift+Tab` is used on selected Markdown list items, the system SHALL treat each selected root list item and its nested descendants as a list tree, preserving internal child structure while moving the selected roots by one list level.

#### Scenario: Selected parent item indents with its children

- **WHEN** the editor is in insert mode
- **AND** a selection includes an ordered list item with nested child items
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** the selected parent item is indented by one list level
- **AND** its nested child items remain nested under that parent item with the same relative depth
- **AND** child items are not indented a second time merely because their lines are also inside the selection

#### Scenario: Mixed-depth selection indents selected roots once

- **WHEN** the editor is in insert mode
- **AND** a selection spans Markdown list items at different nesting levels
- **AND** some selected items are descendants of other selected items
- **AND** no autocomplete suggestion is accepted
- **AND** the user presses `Tab`
- **THEN** only the selected root list items are moved by one list level
- **AND** descendant list items move only as part of their selected root tree
- **AND** the relative structure within each moved tree is preserved

#### Scenario: Selected list tree outdents one level

- **WHEN** the editor is in insert mode
- **AND** a selection includes a nested Markdown list item with descendants
- **AND** the user presses `Shift+Tab`
- **THEN** the selected list item is outdented by one list level
- **AND** its descendants remain nested under that item with the same relative depth
- **AND** no selected descendant is outdented a second time

#### Scenario: Top-level list item does not outdent past document margin

- **WHEN** the editor is in insert mode
- **AND** the selected Markdown list item is already at the top list level
- **AND** the user presses `Shift+Tab`
- **THEN** the item remains a Markdown list item at the top list level
- **AND** the marker is not removed as part of list-tree outdent

### Requirement: Markdown ordered list indentation renumbers affected containers

When Markdown ordered list items are moved by `Tab` or `Shift+Tab`, the system SHALL renumber every affected ordered-list container according to the item's position among siblings in that container.

#### Scenario: Ordered list item starts at one when indented into a new child container

- **WHEN** the editor is in insert mode
- **AND** an ordered list item is selected
- **AND** no ordered child list already exists at the target position
- **AND** the user presses `Tab`
- **THEN** the moved item is numbered `1.` in the new child ordered-list container
- **AND** later sibling items in the original ordered-list container are renumbered consecutively

#### Scenario: Ordered list item continues numbering when indented into an existing child container

- **WHEN** the editor is in insert mode
- **AND** an ordered list item is selected
- **AND** an ordered child list already exists at the target position
- **AND** the user presses `Tab`
- **THEN** the moved item is numbered according to its position in that child ordered-list container
- **AND** all sibling items in that child ordered-list container are renumbered consecutively
- **AND** later sibling items in the original ordered-list container are renumbered consecutively

#### Scenario: Ordered list item outdent renumbers source and target containers

- **WHEN** the editor is in insert mode
- **AND** a nested ordered list item is selected
- **AND** the user presses `Shift+Tab`
- **THEN** the moved item is numbered according to its new position in the parent ordered-list container
- **AND** remaining sibling items in the original nested ordered-list container are renumbered consecutively
- **AND** later sibling items in the parent ordered-list container are renumbered consecutively

#### Scenario: Mixed-depth ordered selection renumbers each affected container independently

- **WHEN** the editor is in insert mode
- **AND** a selection spans ordered list items from multiple nesting levels
- **AND** the user presses `Tab` or `Shift+Tab`
- **THEN** each affected ordered-list container is renumbered independently from its first visible sibling item
- **AND** numbering in one ordered-list container does not reuse a global counter from another container

### Requirement: Markdown ordered list continuation increments markers

When the Vim editor is in insert mode and `Enter` continues a Markdown ordered list item, the system SHALL increment the numeric list marker for the new item.

#### Scenario: Ordered list marker increments on Enter

- **WHEN** the cursor is at the end of `1. item`
- **AND** the editor is in insert mode
- **AND** the user presses `Enter`
- **THEN** the editor inserts a new line beginning with `2. `

#### Scenario: Multi-digit ordered list marker increments on Enter

- **WHEN** the cursor is at the end of `9. item`
- **AND** the editor is in insert mode
- **AND** the user presses `Enter`
- **THEN** the editor inserts a new line beginning with `10. `

#### Scenario: Nested ordered list marker preserves indentation

- **WHEN** the cursor is at the end of `    1. nested`
- **AND** the editor is in insert mode
- **AND** the user presses `Enter`
- **THEN** the editor inserts a new line beginning with `    2. `

#### Scenario: Empty ordered list item still exits list

- **WHEN** the cursor is on an empty ordered list item such as `2. `
- **AND** the editor is in insert mode
- **AND** the user presses `Enter`
- **THEN** the editor exits or reduces the current list item using the existing empty-list behavior
- **AND** it does not insert `3. `

### Requirement: Vim editor text-key behavior is consistent across editor surfaces

The system SHALL apply the same insert-mode `Tab`, `Shift+Tab`, autocomplete confirmation, and Markdown list continuation behavior in both the preview panel editor and the fullscreen editor.

#### Scenario: Preview editor and fullscreen editor handle Tab consistently

- **WHEN** the same document content and cursor position are edited in `PreviewEditor` and `FullscreenEditor`
- **AND** the editor is in insert mode
- **AND** the user presses `Tab`
- **THEN** both editor surfaces produce the same document change

#### Scenario: Preview editor and fullscreen editor handle ordered list Enter consistently

- **WHEN** the same Markdown ordered list item is edited in `PreviewEditor` and `FullscreenEditor`
- **AND** the editor is in insert mode
- **AND** the user presses `Enter`
- **THEN** both editor surfaces continue the list with the same incremented marker
