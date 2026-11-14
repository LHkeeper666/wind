## ADDED Requirements

### Requirement: Keyboard entry to a visible outline across layouts

When the preview column has focus in global normal mode, `Ctrl+W l` SHALL move focus to its visible Markdown or PDF outline regardless of whether the workspace has three, two, or one visible columns. The shortcut MUST leave focus unchanged when no outline is visible.

#### Scenario: Three-column Markdown outline
- **WHEN** the preview column has focus in three-column mode and a Markdown outline is visible
- **AND** the user presses `Ctrl+W l`
- **THEN** the Markdown outline receives focus

#### Scenario: Three-column PDF outline
- **WHEN** the preview column has focus in three-column mode and a PDF outline is visible
- **AND** the user presses `Ctrl+W l`
- **THEN** the PDF outline receives focus

#### Scenario: Reduced-column layouts
- **WHEN** the preview column has focus in a two-column or one-column layout and an outline is visible
- **AND** the user presses `Ctrl+W l`
- **THEN** the outline receives focus

#### Scenario: Enter preview before outline
- **WHEN** a directory column has focus and the preview column is visible
- **AND** the user uses `Ctrl+W l` to move right into preview
- **THEN** focus moves to preview content
- **AND** a subsequent `Ctrl+W l` focuses its visible outline

#### Scenario: Outline unavailable
- **WHEN** the preview column has focus and its outline is hidden or absent
- **AND** the user presses `Ctrl+W l`
- **THEN** focus remains in the preview column

#### Scenario: Return to preview content
- **WHEN** an outline has focus
- **AND** the user presses `Ctrl+W h`
- **THEN** focus returns to preview content
