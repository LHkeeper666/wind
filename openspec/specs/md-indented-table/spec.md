# md-indented-table

## Purpose

Recognize markdown tables with leading whitespace (spaces or tabs) and render them as proper tables while preserving their visual indentation level.

## Requirements

### Requirement: Indented tables recognized and rendered
The system SHALL recognize markdown tables with leading whitespace (spaces or tabs) and render them as proper tables.

#### Scenario: Table indented with spaces
- **WHEN** a markdown file contains a table block where each line is indented with 2 or more spaces
- **THEN** the block is recognized as a table and rendered with proper table HTML elements
- **AND** the rendered table maintains its visual indentation via margin-left

#### Scenario: Table indented with tabs
- **WHEN** a markdown file contains a table block where each line is indented with one or more tab characters
- **THEN** the block is recognized as a table and rendered with proper table HTML elements
- **AND** the rendered table maintains its visual indentation (1 tab = 4ch)

#### Scenario: Table indented with 2 tabs
- **WHEN** a markdown file contains a table block where each line is indented with two tab characters
- **THEN** the block is recognized as a table and rendered with proper table HTML elements
- **AND** the rendered table has margin-left of 8ch

#### Scenario: Non-table indented code is not affected
- **WHEN** a markdown file contains indented lines that do not match the table pattern (no `|` separators)
- **THEN** those lines are rendered as normal paragraphs or code blocks (standard markdown behavior)

#### Scenario: Single vertical bar line not mistaken for table
- **WHEN** a markdown file contains a line with leading whitespace and a single `|` character (e.g., `  | x | = x > 0`)
- **THEN** the line is NOT treated as a table row
- **AND** the line is rendered as normal text

#### Scenario: Table with consistent indentation
- **WHEN** a table block has consistent indentation across all rows
- **THEN** only that common indentation is stripped
- **AND** the table renders correctly with all rows as part of the same table

### Requirement: Indented table visual indentation preserved
The system SHALL preserve the visual indentation of tables after rendering.

#### Scenario: Visual indentation matches original
- **WHEN** a table with N characters of leading whitespace is rendered
- **THEN** the rendered table is wrapped in a container with `margin-left: Nch`
- **AND** the table content aligns with the indentation level of the source markdown