## ADDED Requirements

### Requirement: RAR extraction streaming memory bound
The system SHALL extract RAR entries without buffering an entry's full uncompressed content in application memory. The extraction memory footprint MUST NOT scale with the size of an individual archive entry.

#### Scenario: Extract archive containing a multi-gigabyte entry
- **WHEN** the user extracts a RAR archive containing an entry several gigabytes in size
- **THEN** the system writes the entry to disk incrementally
- **AND** the process memory footprint does not grow in proportion to that entry's size

#### Scenario: Entry whose uncompressed size approaches available memory
- **WHEN** a RAR entry's uncompressed size is several gigabytes and available physical memory is limited
- **THEN** the system completes the extraction to disk
- **AND** the process does not terminate from an allocation failure

### Requirement: Serialized RAR library access
The system SHALL ensure that at most one operation at a time executes against the underlying unrar library. A read-only RAR operation requested while another RAR operation is in progress SHALL fail fast with a distinguishable "archive busy" error instead of blocking.

#### Scenario: Directory listing requested during extraction
- **WHEN** an extraction of a RAR archive is in progress
- **AND** the user selects that archive in the file panel, triggering an archive directory listing
- **THEN** the system returns an "archive busy" error rather than blocking until the extraction finishes
- **AND** the UI surfaces a message indicating the archive is being extracted

#### Scenario: Preview of an entry during extraction
- **WHEN** an extraction of a RAR archive is in progress
- **AND** the user requests to preview an entry from that archive
- **THEN** the system returns an "archive busy" error rather than blocking
- **AND** MUST NOT read the archive concurrently with the extraction

#### Scenario: Two RAR extractions dispatched concurrently
- **WHEN** two RAR extraction tasks are dispatched while the scheduler allows them to run
- **THEN** the system executes them one after another against the unrar library
- **AND** both complete without library state corruption

### Requirement: RAR extraction progress reporting
The system SHALL report extraction progress per archive entry while preserving the existing `transfer-progress` event contract (`bytes_done`, `total_bytes`, `status`). Cancellation SHALL take effect between entries.

#### Scenario: Progress reported for each entry
- **WHEN** the user extracts a RAR archive containing multiple entries
- **THEN** the system emits a `transfer-progress` event when each entry begins and when it completes
- **AND** `bytes_done` advances by each completed entry's uncompressed size
- **AND** `total_bytes` remains the archive's total uncompressed size

#### Scenario: Cancellation between entries
- **WHEN** the user cancels an in-progress RAR extraction
- **THEN** the system stops before starting the next entry
- **AND** the task is reported as cancelled

#### Scenario: Progress for selected entries only
- **WHEN** the user extracts selected entries from a RAR archive
- **THEN** `total_bytes` reflects only the selected entries' uncompressed size
- **AND** progress advances per extracted entry
