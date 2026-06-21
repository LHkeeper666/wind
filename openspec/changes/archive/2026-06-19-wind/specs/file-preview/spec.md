## ADDED Requirements

### Requirement: Previewer plugin interface
The system SHALL define a unified Previewer interface that all file previewers implement.

#### Scenario: Previewer registration
- **WHEN** Wind starts
- **THEN** all built-in previewers are registered with the PreviewRouter

#### Scenario: Previewer matching
- **WHEN** a file is selected for preview
- **THEN** the PreviewRouter finds the first registered previewer whose match() returns true

#### Scenario: Fallback preview
- **WHEN** no previewer matches the file type
- **THEN** the file is displayed as plain text or hex dump

### Requirement: Text and code syntax highlighting
The system SHALL provide syntax highlighting for text and code files using Shiki.

#### Scenario: Highlight a JavaScript file
- **WHEN** user selects a .js file in the file tree
- **THEN** the preview displays the file with JavaScript syntax highlighting

#### Scenario: Highlight with VSCode theme
- **WHEN** a code file is previewed
- **THEN** the syntax highlighting uses the active Shiki theme (e.g., One Dark Pro, GitHub Dark)

#### Scenario: Large file handling
- **WHEN** user previews a text file larger than 1MB
- **THEN** only the first N lines are highlighted, with a virtual scroll mechanism for the rest

### Requirement: Markdown preview
The system SHALL render Markdown files with full formatting support.

#### Scenario: Render Markdown with formatting
- **WHEN** user selects a .md file
- **THEN** the preview displays rendered HTML with headings, bold, italic, lists, tables, and code blocks

#### Scenario: Render math formulas
- **WHEN** a Markdown file contains LaTeX math formulas (e.g., $E=mc^2$)
- **THEN** the formulas are rendered using KaTeX

#### Scenario: Render Mermaid diagrams
- **WHEN** a Markdown file contains Mermaid diagram code blocks
- **THEN** the diagrams are rendered visually using Mermaid

#### Scenario: Code block highlighting
- **WHEN** a Markdown file contains fenced code blocks with language specifiers
- **THEN** the code blocks are syntax-highlighted using Shiki

### Requirement: PDF preview
The system SHALL render PDF files using pdf.js.

#### Scenario: Render PDF pages
- **WHEN** user selects a .pdf file
- **THEN** the PDF pages are rendered as canvas elements with proper scaling

#### Scenario: PDF page navigation
- **WHEN** user scrolls through a PDF
- **THEN** pages are rendered on-demand (visible pages + buffer pages)

#### Scenario: PDF zoom
- **WHEN** user uses mouse wheel or keyboard to zoom in a PDF
- **THEN** the PDF pages are re-rendered at the new zoom level

### Requirement: Image preview
The system SHALL display image files with zoom and pan capabilities.

#### Scenario: Display an image
- **WHEN** user selects a .png, .jpg, .gif, .svg, or .webp file
- **THEN** the image is displayed in the preview panel

#### Scenario: Image zoom
- **WHEN** user scrolls the mouse wheel over an image preview
- **THEN** the image zooms in/out

#### Scenario: Image pan
- **WHEN** user drags the mouse on a zoomed image
- **THEN** the image pans to show different regions

#### Scenario: Image info display
- **WHEN** an image is previewed
- **THEN** the image dimensions, format, and file size are displayed

### Requirement: JSON preview
The system SHALL display JSON files with both raw and tree views.

#### Scenario: JSON tree view
- **WHEN** user selects a .json file
- **THEN** the preview displays a collapsible tree view of the JSON structure

#### Scenario: JSON raw view
- **WHEN** user switches to raw view in JSON preview
- **THEN** the JSON is displayed as syntax-highlighted text

### Requirement: CSV preview
The system SHALL display CSV files as a table.

#### Scenario: CSV table view
- **WHEN** user selects a .csv file
- **THEN** the preview displays the data as a table with headers and rows

#### Scenario: CSV pagination
- **WHEN** a CSV file has more than 1000 rows
- **THEN** the preview shows the first 1000 rows with pagination controls
