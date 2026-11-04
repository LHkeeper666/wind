## ADDED Requirements

### Requirement: Torrent preview uses one binary read
For a `.torrent` path, `loadTextOrBinary()` SHALL invoke a binary read directly and SHALL NOT invoke a text read or charset detection first.

#### Scenario: Torrent file within the preview size limit
- **WHEN** a `.torrent` file is selected for preview and its size is at most 1 MiB
- **THEN** the loader invokes `read_binary_file` once and does not invoke `read_file`

#### Scenario: Torrent file above the preview size limit
- **WHEN** a `.torrent` file larger than 1 MiB is selected for preview
- **THEN** the loader invokes `read_binary_file_partial` and does not invoke `read_file_partial`
