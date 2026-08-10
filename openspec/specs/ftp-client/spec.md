# ftp-client Specification

## Purpose
TBD - created by archiving change add-ftp-client. Update Purpose after archive.
## Requirements
### Requirement: FTP connection management
The system SHALL allow users to register, connect to, and disconnect from FTP servers using named connections.

#### Scenario: Connect to FTP server
- **WHEN** user types `:ftp connect myserver 192.168.1.5 --user admin --pass secret`
- **THEN** system connects to the FTP server at 192.168.1.5:21 with credentials admin/secret and registers it as "myserver"
- **AND** connection appears in virtual root `\` as `[FTP] myserver`

#### Scenario: Connect with custom port
- **WHEN** user types `:ftp connect myserver 192.168.1.5 --port 2121`
- **THEN** system connects on port 2121 instead of the default 21

#### Scenario: Connect with anonymous login
- **WHEN** user types `:ftp connect public-srv ftp.example.com` (no --user)
- **THEN** system connects with anonymous/anonymous credentials

#### Scenario: Disconnect from server
- **WHEN** user types `:ftp disconnect myserver`
- **THEN** system closes the FTP connection, removes it from virtual root, and shows toast "Disconnected from myserver"

#### Scenario: List all connections
- **WHEN** user types `:ftp list`
- **THEN** system shows a list of all registered connections with their host, port, and user

#### Scenario: Duplicate connection name
- **WHEN** user tries to connect with a name that is already registered
- **THEN** system shows toast "Connection 'myserver' already exists" and does nothing

### Requirement: FTP connection persistence
The system SHALL persist FTP connection metadata (name, host, port, user, password) to a local JSON configuration file, and restore connections on startup.

#### Scenario: Save connection on connect
- **WHEN** user successfully connects to an FTP server
- **THEN** system saves the connection metadata to `%APPDATA%/wind/ftp-connections.json`

#### Scenario: Restore connections on startup
- **WHEN** Wind starts
- **THEN** system reads `%APPDATA%/wind/ftp-connections.json` and populates the FTP connection list
- **AND** connections appear in virtual root `\` without requiring re-connection

#### Scenario: Remove connection on disconnect
- **WHEN** user disconnects from an FTP server
- **THEN** system removes the connection entry from the persisted configuration file

#### Scenario: Auto-reconnect on use
- **WHEN** user navigates to `ftp://myserver/` and the connection to "myserver" is stored but not currently active
- **THEN** system automatically reconnects using the stored credentials

### Requirement: FTP path scheme
The system SHALL use the URI format `ftp://<connection-name>/<remote-path>` to represent FTP paths throughout the application.

#### Scenario: Navigate to FTP directory via command palette
- **WHEN** user types `:cd ftp://myserver/var/www`
- **THEN** current panel displays the contents of `/var/www` on the FTP server "myserver"

#### Scenario: FTP path in status bar
- **WHEN** current panel is browsing `ftp://myserver/var/www`
- **THEN** status bar displays `ftp://myserver/var/www`

### Requirement: FTP directory browsing
The system SHALL allow users to browse directories on connected FTP servers using the same DirectoryPanel interface as local directories.

#### Scenario: List FTP root directory
- **WHEN** current panel path is `ftp://myserver/`
- **THEN** system lists files and directories from the FTP server's root using MLSD (or LIST as fallback)

#### Scenario: Navigate into FTP subdirectory
- **WHEN** user presses `Enter` on a directory entry in an FTP listing
- **THEN** current panel navigates into that directory and displays its contents

#### Scenario: Navigate up from FTP directory
- **WHEN** user presses `h` while parent panel is focused or presses `Enter` on `..` entry in an FTP subdirectory
- **THEN** current panel navigates to the parent directory on the FTP server

#### Scenario: FTP directory refresh
- **WHEN** user presses `R` on an FTP panel
- **THEN** system re-fetches the directory listing from the FTP server

#### Scenario: FTP connection error during browsing
- **WHEN** the FTP connection drops while listing a directory
- **THEN** DirectoryPanel displays error message "Failed to load: connection lost"

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

### Requirement: FTP file delete
The system SHALL allow users to delete files and directories on an FTP server.

#### Scenario: Delete single FTP file to trash
- **WHEN** user presses `d` on a file in an FTP panel
- **THEN** system deletes the file from the FTP server and shows toast "Deleting 1 file..."

#### Scenario: Delete FTP directory recursively
- **WHEN** user presses `D` on a directory in an FTP panel
- **THEN** system permanently deletes the directory and all its contents from the FTP server

### Requirement: FTP file rename
The system SHALL allow users to rename files on an FTP server.

#### Scenario: Rename FTP file
- **WHEN** user presses `r` on a file in an FTP panel, enters a new name, and confirms
- **THEN** system renames the file on the FTP server

### Requirement: FTP listing maps to FileEntry
FTP directory listings SHALL be converted to the existing `FileEntry` struct so that DirectoryPanel renders them unchanged.

#### Scenario: FTP file entry fields
- **WHEN** the FTP server returns a file entry
- **THEN** the FileEntry contains: name (filename), path (full `ftp://` URI), is_dir (false), size (file size in bytes), is_hidden (based on dot-prefix), modified (file mtime as unix timestamp)

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

