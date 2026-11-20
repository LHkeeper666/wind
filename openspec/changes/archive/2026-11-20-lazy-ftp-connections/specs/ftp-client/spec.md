## MODIFIED Requirements

### Requirement: FTP connection management
The system SHALL allow users to save named FTP sites, connect to them, disconnect active sessions, and remove saved sites as distinct operations.

#### Scenario: Save a new site offline
- **WHEN** user types `:ftp add myserver 192.168.1.5 --user admin --pass secret` while the server is offline
- **THEN** the configuration is saved and `[FTP] myserver` appears in virtual root `\`
- **AND** no network connection is attempted

#### Scenario: Connect to FTP server
- **WHEN** user types `:ftp connect myserver 192.168.1.5 --user admin --pass secret`
- **THEN** the system saves the site, attempts an immediate connection to 192.168.1.5:21 with the given credentials, and navigates to its FTP root on success
- **AND** connection failure is reported while the saved site remains available

#### Scenario: Connect with custom port
- **WHEN** user types `:ftp connect myserver 192.168.1.5 --port 2121`
- **THEN** the system uses port 2121 instead of default 21

#### Scenario: Connect with anonymous login
- **WHEN** user types `:ftp connect public-srv ftp.example.com` without `--user`
- **THEN** the system uses anonymous/anonymous credentials

#### Scenario: Connect a saved site
- **WHEN** user types `:ftp connect myserver` for a saved site
- **THEN** the system reuses a valid session or establishes one and navigates to `ftp://myserver/`

#### Scenario: Disconnect current session
- **WHEN** user types `:ftp disconnect myserver`
- **THEN** the current main session is closed or removed and `[FTP] myserver` remains in virtual root

#### Scenario: Remove saved site
- **WHEN** user types `:ftp remove myserver`
- **THEN** the saved configuration and current main session are removed and the site disappears from virtual root

#### Scenario: List all connections
- **WHEN** user types `:ftp list`
- **THEN** the system lists all saved sites with host, port, and user regardless of session state

#### Scenario: Duplicate connection name
- **WHEN** user tries to add or connect with a new host under an already saved name
- **THEN** the system reports that the site already exists and does not overwrite it

### Requirement: FTP connection persistence
The system SHALL persist named FTP site metadata (name, host, port, user, password) to the existing local JSON configuration file and restore sites without connecting on startup.

#### Scenario: Save site before explicit connection
- **WHEN** a new site is added or `:ftp connect <name> <host>` is issued
- **THEN** the configuration is saved to `%APPDATA%/wind/ftp-connections.json` before any requested network connection

#### Scenario: Restore sites on startup
- **WHEN** Wind starts
- **THEN** saved sites appear in virtual root `\` without a network connection

#### Scenario: Keep site on disconnect
- **WHEN** the user disconnects a current session
- **THEN** its configuration remains in the persisted file

#### Scenario: Remove site explicitly
- **WHEN** the user removes a saved site
- **THEN** its entry is removed from the persisted file

#### Scenario: Auto-connect on first use
- **WHEN** the user navigates to `ftp://myserver/` or starts an FTP operation for a saved site without a main session
- **THEN** the system establishes a session using the saved credentials before performing the operation

## ADDED Requirements

### Requirement: FTP session reuse and bounded recovery
The system SHALL reuse a valid main session for interactive FTP operations and SHALL reconnect only when the session is absent or confirmed invalid.

#### Scenario: Reuse an active session
- **WHEN** two directory visits or FTP operations use the same saved site while its main session remains valid
- **THEN** they use that session without establishing another connection

#### Scenario: Recover from a dropped session
- **WHEN** a directory listing encounters a confirmed closed or broken session
- **THEN** the system reconnects using saved settings and retries the listing at most once

#### Scenario: Directory permission error
- **WHEN** directory listing fails for lack of permission while the session remains valid
- **THEN** the original error is returned without reconnecting or retrying the listing

#### Scenario: Ambiguous write failure
- **WHEN** an FTP write operation fails after the server may have applied it
- **THEN** the system reports the error without automatically replaying the write

### Requirement: Saved FTP site editing remains offline
The system SHALL list and edit saved sites without network access; only Save and Reconnect SHALL explicitly establish a new main session.

#### Scenario: Virtual root and editor
- **WHEN** the server is offline and the user opens virtual root or presses `i` on its saved site
- **THEN** the site is listed and its editor opens without a connection attempt

#### Scenario: Ordinary save
- **WHEN** the user chooses Save in the site editor
- **THEN** the updated configuration persists without a connection attempt and any current main session remains until replaced or invalidated

#### Scenario: Save and Reconnect succeeds
- **WHEN** the user chooses Save and Reconnect
- **THEN** the configuration persists and a new session is attempted with updated settings
- **AND** success replaces the old main session

#### Scenario: Save and Reconnect fails
- **WHEN** the server is offline after Save and Reconnect persists the update
- **THEN** the connection error is shown, updated settings remain saved, and an existing old main session remains available

#### Scenario: Configuration changes during connection
- **WHEN** an in-flight connection uses an older configuration and the site is edited or removed before installation
- **THEN** that connection is not installed as the current main session
