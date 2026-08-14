# ftp-client Delta Specification

## ADDED Requirements

### Requirement: FTP folder transfer trigger
The system SHALL trigger folder upload/download when a directory entry is pasted across FTP/local panel boundaries.

#### Scenario: Paste directory from FTP panel to local panel
- **WHEN** the clipboard contains a directory entry from an FTP panel (is_dir=true) and user triggers paste in a local panel
- **THEN** the system calls `ftp_download_folder` instead of `ftp_download` for single files
- **AND** a single batch containing all files in the folder tree is enqueued via TransferScheduler

#### Scenario: Paste local directory to FTP panel
- **WHEN** the clipboard contains a local directory entry (is_dir=true) and user triggers paste in an FTP panel
- **THEN** the system calls `ftp_upload_folder` instead of `ftp_upload` for single files
- **AND** a single batch containing all files in the folder tree is enqueued via TransferScheduler

#### Scenario: Paste single file unchanged
- **WHEN** the clipboard contains a single file entry (is_dir=false)
- **THEN** the system continues to use the existing single-file `ftp_download`/`ftp_upload` path
