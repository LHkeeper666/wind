## ADDED Requirements

### Requirement: Torrent files are classified as binary
The unified file type detector SHALL classify `.torrent` files as binary regardless of extension case, and `TextPreviewer` SHALL use that same classification.

#### Scenario: Torrent extension
- **WHEN** a file path ends in `.torrent` or `.TORRENT`
- **THEN** `isTextFile()` returns `false`
- **AND** `TextPreviewer` treats its content as binary
