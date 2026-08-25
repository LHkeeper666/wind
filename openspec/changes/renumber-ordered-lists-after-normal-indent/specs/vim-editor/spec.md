## ADDED Requirements

### Requirement: Normal Vim indentation renumbers Markdown ordered-list containers

When normal or visual Vim indentation changes Markdown ordered-list nesting, the system SHALL renumber affected ordered-list containers after the Vim indentation operation completes.

#### Scenario: Normal-mode outdent renumbers source and target containers

- **WHEN** the editor is in Vim normal mode
- **AND** the cursor is on a nested ordered Markdown list item
- **AND** the user presses `<<`
- **THEN** the item is outdented by Vim's normal indentation behavior
- **AND** the original nested ordered-list container is renumbered consecutively
- **AND** the target parent ordered-list container is renumbered consecutively

#### Scenario: Normal-mode indent renumbers affected ordered containers

- **WHEN** the editor is in Vim normal mode
- **AND** the cursor is on an ordered Markdown list item
- **AND** the user presses `>>`
- **THEN** the item is indented by Vim's normal indentation behavior
- **AND** every affected ordered-list container is renumbered consecutively

#### Scenario: Visual indentation renumbers selected ordered list structure

- **WHEN** the editor is in Vim visual mode
- **AND** the selection includes one or more ordered Markdown list items
- **AND** the user presses `<` or `>`
- **THEN** Vim applies its existing visual indentation behavior
- **AND** every affected ordered-list container is renumbered consecutively

#### Scenario: Non-Markdown indentation does not receive extra renumbering

- **WHEN** the editor is in Vim normal or visual mode
- **AND** the Vim indentation operation changes ordinary non-list text
- **THEN** the indentation behavior remains the Vim plugin's existing behavior
- **AND** the system does not perform unrelated Markdown numbering edits

#### Scenario: Preview and fullscreen editors behave consistently

- **WHEN** the same Markdown ordered-list document is edited in `PreviewEditor` and `FullscreenEditor`
- **AND** the user performs the same normal or visual Vim indentation operation
- **THEN** both editor surfaces produce the same ordered-list renumbering result
