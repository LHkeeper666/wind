## MODIFIED Requirements

### Requirement: FTP folder transfer trigger
The system SHALL trigger folder upload/download when a directory entry is pasted across FTP/local panel boundaries. Individual files SHALL be submitted through `transfer_enqueue`; folder-transfer entry points SHALL remain available for directory task expansion.

#### Scenario: Paste directory from FTP panel to local panel
- **WHEN** the clipboard contains a directory entry from an FTP panel (is_dir=true) and user triggers paste in a local panel
- **THEN** the system calls `ftp_download_folder`
- **AND** a single batch containing all files in the folder tree is enqueued via TransferScheduler

#### Scenario: Paste local directory to FTP panel
- **WHEN** the clipboard contains a local directory entry (is_dir=true) and user triggers paste in an FTP panel
- **THEN** the system calls `ftp_upload_folder`
- **AND** a single batch containing all files in the folder tree is enqueued via TransferScheduler

#### Scenario: Paste single file unchanged
- **WHEN** the clipboard contains a single file entry (is_dir=false) and user pastes across FTP/local panel boundaries
- **THEN** the system enqueues an FTP upload or download task through `transfer_enqueue`
- **AND** the scheduler executes the single-file transfer without calling the retired `ftp_download` or `ftp_upload` commands
