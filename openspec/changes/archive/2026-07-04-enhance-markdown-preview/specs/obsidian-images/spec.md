## ADDED Requirements

### Requirement: Basic wikilink image
The system SHALL parse Obsidian-style wikilink image syntax `![[filename]]` and render it as an image element.

#### Scenario: Simple image reference
- **WHEN** markdown contains `![[photo.png]]`
- **THEN** the preview renders an img element with the src resolved relative to the current file's directory

### Requirement: Wikilink image with width
The system SHALL parse `![[filename|width]]` syntax and set the image width attribute.

#### Scenario: Image with width constraint
- **WHEN** markdown contains `![[photo.png|200]]`
- **THEN** the preview renders an img element with `width="200"`

### Requirement: Wikilink image with dimensions
The system SHALL parse `![[filename|widthxheight]]` syntax and set both width and height attributes.

#### Scenario: Image with width and height
- **WHEN** markdown contains `![[photo.png|200x100]]`
- **THEN** the preview renders an img element with `width="200"` and `height="100"`

### Requirement: Path resolution
The system SHALL resolve wikilink image paths relative to the directory of the current markdown file.

#### Scenario: Image in same directory
- **WHEN** the current file is `/notes/readme.md` and markdown contains `![[screenshot.png]]`
- **THEN** the image src resolves to the path relative to `/notes/`

### Requirement: Case-insensitive matching
The system SHALL match image filenames case-insensitively when resolving paths.

#### Scenario: Case mismatch
- **WHEN** markdown contains `![[Photo.PNG]]` and the actual file is `photo.png`
- **THEN** the image still loads correctly
