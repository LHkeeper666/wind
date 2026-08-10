## MODIFIED Requirements

### Requirement: FTP file upload
The system SHALL allow users to upload files from the local filesystem to an FTP server with streaming progress reporting and cancellation support.

#### Scenario: Upload via cross-panel paste (copy)
- **WHEN** user yanks (y) a local file, switches to an FTP panel, and presses p
- **THEN** system enqueues the upload task via TransferScheduler with progress displayed in TransferManager panel

#### Scenario: Upload via cross-panel paste (cut)
- **WHEN** user cuts (x) a local file, switches to an FTP panel, and presses p
- **THEN** system enqueues the upload task, and after successful upload, deletes the local source file

#### Scenario: Upload streaming progress
- **WHEN** an FTP upload is in progress
- **THEN** the system emits `transfer-progress` events every 100ms with bytes uploaded and calculated speed
- **AND** TransferManager displays a real-time progress bar with percentage and speed

#### Scenario: Cancel upload
- **WHEN** user cancels an FTP upload via TransferManager
- **THEN** the partial file on the remote server is deleted
- **AND** the transfer status changes to "cancelled"

### Requirement: FTP file download
The system SHALL allow users to download files from an FTP server to the local filesystem with streaming progress reporting and cancellation support.

#### Scenario: Download via cross-panel paste (copy)
- **WHEN** user yanks (y) a file from an FTP panel, switches to a local panel, and presses p
- **THEN** system enqueues the download task via TransferScheduler with progress displayed in TransferManager panel

#### Scenario: Download via cross-panel paste (cut)
- **WHEN** user cuts (x) a file from an FTP panel, switches to a local panel, and presses p
- **THEN** system enqueues the download task, and after successful download, deletes the remote source file

#### Scenario: Download streaming progress
- **WHEN** an FTP download is in progress
- **THEN** the system emits `transfer-progress` events every 100ms with bytes downloaded and calculated speed
- **AND** the download uses a 64KB chunked read loop instead of `tokio::io::copy` for intermediate progress reporting

#### Scenario: Cancel download
- **WHEN** user cancels an FTP download via TransferManager
- **THEN** the partial local file is deleted
- **AND** the transfer status changes to "cancelled"
