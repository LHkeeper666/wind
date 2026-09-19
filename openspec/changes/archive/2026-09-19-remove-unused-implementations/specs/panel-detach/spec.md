## MODIFIED Requirements

### Requirement: Cross-panel paste routing
The system SHALL route paste operations to the correct backend based on the source and destination path schemes.

#### Scenario: Local to local paste
- **WHEN** clipboard contains local files and the active paste target is a local directory
- **THEN** system uses existing local copy/move operations (unchanged behavior)

#### Scenario: Local to FTP paste (upload)
- **WHEN** clipboard contains local files and the active paste target is an FTP directory
- **THEN** system enqueues individual file uploads through `transfer_enqueue` and directory uploads through `ftp_upload_folder`
- **AND** the system does not invoke the retired `ftp_upload` command

#### Scenario: FTP to local paste (download)
- **WHEN** clipboard contains FTP files and the active paste target is a local directory
- **THEN** system enqueues individual file downloads through `transfer_enqueue` and directory downloads through `ftp_download_folder`
- **AND** the system does not invoke the retired `ftp_download` command

#### Scenario: FTP to FTP paste (same server)
- **WHEN** clipboard contains FTP files from server "A" and the active paste target is another directory on server "A"
- **THEN** system uses server-side rename/move (RNFR/RNTO) for efficiency

#### Scenario: FTP to FTP paste (different servers)
- **WHEN** clipboard contains FTP files from server "A" and the active paste target is a directory on server "B"
- **THEN** system downloads from A then uploads to B (relay transfer)
