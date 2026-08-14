## ADDED: IpynbPreviewer Class

### Requirement: .ipynb File Detection
The system SHALL detect `.ipynb` files by extension and route them to `IpynbPreviewer` instead of the raw JSON editor.

#### Scenario: Selecting a .ipynb file
- **WHEN** the user navigates to a file ending in `.ipynb`
- **THEN** the preview panel renders a notebook view showing formatted cells

### Requirement: Markdown Cell Rendering
The system SHALL render `cell_type: "markdown"` cells as formatted markdown.

#### Scenario: Notebook contains markdown cells
- **WHEN** a .ipynb file contains markdown cells
- **THEN** each markdown cell's source is rendered as HTML using markdown-it (headings, lists, links, code blocks, emphasis)

### Requirement: Code Cell Rendering
The system SHALL render `cell_type: "code"` cells with syntax-highlighted source code.

#### Scenario: Notebook contains Python code cells
- **WHEN** a .ipynb file contains code cells
- **THEN** the source code is syntax-highlighted using Shiki
- **AND** the kernel language (from metadata.kernelspec.language) is used to select the highlighting language

#### Scenario: Code cell has an execution count
- **WHEN** a code cell has `execution_count` set to a number
- **THEN** "In [N]:" label is displayed; if null, "In [ ]:" is shown

### Requirement: Code Cell Output Display
The system SHALL display `text/plain` and `image/png` outputs below each code cell.

#### Scenario: Code cell has text output
- **WHEN** a code cell has outputs containing `text/plain` data
- **THEN** the text output is displayed in a styled output area

#### Scenario: Code cell has image output
- **WHEN** a code cell has outputs containing `image/png` data
- **THEN** the image is rendered using a base64 data URI

#### Scenario: Code cell has multiple outputs
- **WHEN** a code cell has multiple output entries
- **THEN** each output is displayed in sequence with an "Out [N]:" label

### Requirement: Raw Cell Rendering
The system SHALL render `cell_type: "raw"` cells as preformatted plain text.

#### Scenario: Notebook contains raw cells
- **WHEN** a .ipynb file contains raw cells
- **THEN** the raw content is displayed in a `<pre>` block

### Requirement: Cell Type Badges
The system SHALL display a cell type badge (Markdown/Code/Raw) at the top of each cell for visual distinction.

#### Scenario: Cell type identification
- **WHEN** a notebook preview is rendered
- **THEN** each cell has a small badge indicating its cell_type

### Requirement: Error Handling
The system SHALL handle invalid .ipynb JSON gracefully.

#### Scenario: File is not valid JSON
- **WHEN** a file with `.ipynb` extension cannot be parsed as JSON
- **THEN** an error message is displayed in the preview panel with the raw content shown as plain text

#### Scenario: File JSON does not conform to notebook schema
- **WHEN** parsed JSON lacks a `cells` array
- **THEN** a warning is displayed and the content is shown as plain text

### Requirement: Source String Array Handling
The system SHALL correctly join source string arrays into single strings.

#### Scenario: source field is array of strings
- **WHEN** a cell's `source` field is `["line 1\n", "line 2\n"]`
- **THEN** it is joined as `"line 1\nline 2\n"` before rendering

### Requirement: Editor Mode for Raw JSON
The system SHALL allow editing the raw JSON of a .ipynb file by pressing `e`.

#### Scenario: Press e on notebook preview
- **WHEN** the user presses `e` while previewing a .ipynb file
- **THEN** the editor opens showing the raw JSON content
- **AND** the file is editable like any other text file

### Requirement: Fullscreen Editor Support
The system SHALL allow opening .ipynb in fullscreen editor by pressing `E`.

#### Scenario: Press E on notebook preview
- **WHEN** the user presses `E` (shift+e) while previewing a .ipynb file
- **THEN** the fullscreen editor opens with the raw JSON content

### Requirement: Keyboard Navigation
The system SHALL support j/k scrolling in notebook preview.

#### Scenario: Scroll notebook preview
- **WHEN** the user presses `j` or `k` while focused on the notebook preview
- **THEN** the preview scrolls down or up by 40px
