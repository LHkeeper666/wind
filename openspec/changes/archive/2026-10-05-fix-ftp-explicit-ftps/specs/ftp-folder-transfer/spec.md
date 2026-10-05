## MODIFIED Requirements

### Requirement: FTP folder upload
The system SHALL allow users to upload entire directories from the local filesystem to an FTP server, expanding the directory tree into individual file transfer tasks within a single batch. Remote directory creation SHALL distinguish between "already exists" and "permission denied" errors.

#### Scenario: Upload folder via cross-panel paste (copy)
- **WHEN** user yanks (y) a local directory, switches to an FTP panel, and presses p
- **THEN** the system recursively scans the local directory to build a file list with sizes
- **AND** creates the remote directory structure on the FTP server using MKD commands
- **AND** enqueues all files as individual FtpUpload tasks in a single batch via TransferScheduler

#### Scenario: Upload folder via cross-panel paste (cut)
- **WHEN** user cuts (x) a local directory, switches to an FTP panel, and presses p
- **THEN** after all files in the batch upload successfully, the local source directory is deleted
- **AND** if any file fails, the local source is preserved

#### Scenario: Pre-create remote directories before upload
- **WHEN** a folder upload is initiated
- **THEN** all remote directories in the tree are created on the FTP server using MKD before the first file transfer starts
- **AND** MKD errors for missing parent directories are handled by creating intermediate directories as needed
- **AND** MKD errors indicating "permission denied" (e.g., 550 with "permissions" in the message) SHALL be reported as errors and abort the folder upload
- **AND** MKD errors indicating "directory already exists" (e.g., 521 or "exist" in the message) SHALL be silently ignored

#### Scenario: Cancel folder upload
- **WHEN** user cancels all transfers in a folder upload batch
- **THEN** all running uploads are cancelled and partial remote files are cleaned up via RM on the FTP server

#### Scenario: Permission error aborts folder upload
- **WHEN** MKD fails with a permission error (550 with "permissions" in the error message) during remote directory creation
- **THEN** the folder upload is aborted before any file transfers begin
- **AND** an error message is returned indicating the specific directory that failed and the permission error

#### Scenario: Independent session cleanup
- **WHEN** a folder upload completes (success or failure)
- **THEN** all independent FTP sessions created for the upload are properly closed with QUIT commands
- **AND** no orphaned connections remain on the FTP server