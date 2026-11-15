## ADDED Requirements

### Requirement: Common RST document rendering
The system SHALL render `.rst` files as formatted documents in the preview pane. It SHALL support section headings, paragraphs, emphasis, strong emphasis, inline literals, bullet and enumerated lists, hyperlinks, literal blocks, `code` and `code-block` directives, simple and grid tables, images, and note, warning, tip, and important admonitions.

#### Scenario: Common document structures
- **WHEN** a user opens an `.rst` document containing the supported structures
- **THEN** the preview displays their formatted equivalents in source order
- **AND** the user can enter the existing editor action to change the source

#### Scenario: Case-insensitive extension
- **WHEN** a user opens a file ending in `.RST`
- **THEN** the file receives the same RST preview behavior

### Requirement: Safe and resilient RST preview
The system MUST treat document content as untrusted data. It SHALL escape source text, reject active URL schemes, and SHALL NOT execute directives or embedded code. Unsupported or malformed constructs SHALL remain readable without preventing supported content from rendering.

#### Scenario: Unsupported directive
- **WHEN** a document contains a Sphinx-only or otherwise unsupported directive
- **THEN** the preview shows its source or a clear unsupported-content placeholder
- **AND** subsequent supported paragraphs still render

#### Scenario: Active content in a link
- **WHEN** a link target uses an active scheme such as `javascript:`
- **THEN** the preview does not create an actionable link to that target

#### Scenario: Malformed document
- **WHEN** table borders or directive indentation are malformed
- **THEN** the preview does not execute content or fail as a whole
- **AND** the affected text remains readable

#### Scenario: Comment versus directive
- **WHEN** a document contains a plain `..` comment and a supported `.. code-block::` directive
- **THEN** the comment does not appear in the preview
- **AND** the directive renders a code block with syntax highlighting for a supported language

#### Scenario: Relative indentation in code blocks
- **WHEN** the first source line of a literal or code directive block is indented more than a later line
- **THEN** the preview preserves that relative indentation on the first line

### Requirement: Local images and preview lifecycle
The system SHALL resolve relative image paths against the RST file location and SHALL keep rendering scoped to the current file and tab.

#### Scenario: Relative image
- **WHEN** an RST document references a local image with a relative path
- **THEN** the preview displays that image using the existing local asset path behavior

#### Scenario: Rapid file switch
- **WHEN** the user switches files before an RST preview finishes rendering
- **THEN** the previous file's result does not replace the current preview
