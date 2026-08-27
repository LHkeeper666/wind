# md-frontmatter-rendering

## Purpose

Parse YAML frontmatter from markdown files and render it as a compact, styled metadata block above the main content, replacing the default raw-text rendering.

## Requirements

### Requirement: Frontmatter parsed and rendered as metadata block
The system SHALL parse YAML frontmatter from markdown files and render it as a compact metadata block above the main content.

#### Scenario: File with frontmatter
- **WHEN** a markdown file contains `---` delimited YAML frontmatter at the top
- **THEN** the frontmatter is parsed and rendered as a styled metadata block with key-value pairs in small muted text
- **AND** the `---` delimiters are not rendered as horizontal rules
- **AND** the metadata block is visually separated from the main content by a dashed bottom border

#### Scenario: File without frontmatter
- **WHEN** a markdown file does not contain YAML frontmatter
- **THEN** no metadata block is rendered
- **AND** the file content is rendered normally without any modification

#### Scenario: Frontmatter with nested values
- **WHEN** frontmatter contains nested objects or arrays (e.g., `tags: [a, b]`, `author: { name: "x" }`)
- **THEN** nested values are flattened to a JSON-like string representation
- **AND** the rendering does not crash or show errors

#### Scenario: Invalid frontmatter
- **WHEN** the file starts with `---` but the content between delimiters is not valid YAML
- **THEN** the system falls back to rendering the content as normal markdown
- **AND** no error is visible to the user

### Requirement: Frontmatter metadata styling
The system SHALL style the frontmatter metadata block with small, muted text to distinguish it from the main content.

#### Scenario: Visual appearance
- **WHEN** frontmatter metadata is rendered
- **THEN** the font size is smaller than body text (approximately 0.82em)
- **AND** the text opacity is reduced to visually de-emphasize it
- **AND** keys are displayed in accent color and values in secondary text color

### Requirement: Heading line numbers account for frontmatter
The system SHALL adjust heading line numbers in the table of contents to account for removed frontmatter lines.

#### Scenario: TOC line numbers
- **WHEN** a file with N lines of frontmatter is parsed for headings
- **THEN** heading line numbers are offset by N to match the visible content lines