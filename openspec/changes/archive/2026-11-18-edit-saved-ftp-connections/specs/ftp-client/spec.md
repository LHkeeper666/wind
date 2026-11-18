## ADDED Requirements

### Requirement: Edit saved FTP connection from virtual root
The system SHALL open an FTP connection editor when the user presses `i` on a saved FTP connection entry in the virtual root. The editor SHALL prefill host, port, and user, keep the connection name read-only, and SHALL NOT return the saved password value to the frontend.

#### Scenario: Open saved connection editor
- **WHEN** the virtual root is focused, `[FTP] myserver` is selected, and the user presses `i`
- **THEN** the system opens an editor for `myserver` with its saved host, port, and user
- **AND** the password input is empty and indicates that leaving it empty preserves the saved password

#### Scenario: Preserve file information shortcut
- **WHEN** a non-connection entry is selected and the user presses `i`
- **THEN** the existing file information behavior runs

#### Scenario: Do not edit connection through remote file
- **WHEN** a file or directory inside `ftp://myserver/` is selected and the user presses `i`
- **THEN** the FTP connection editor does not open

### Requirement: Save edited FTP connection
The system SHALL validate and persist updates to an existing saved FTP connection without changing its name. A blank password input SHALL retain the saved password; a nonblank password SHALL replace it.

#### Scenario: Save without reconnecting
- **WHEN** the user changes the host or credentials and chooses Save
- **THEN** the saved configuration is updated and the editor closes
- **AND** any active main session remains active until reconnect or failure
- **AND** newly created sessions use the updated configuration

#### Scenario: Save and reconnect
- **WHEN** the user changes the connection and chooses Save and Reconnect
- **THEN** the configuration is persisted and the system attempts a new main session with the updated settings
- **AND** a successful new session replaces the old main session

#### Scenario: Reconnect fails after save
- **WHEN** Save and Reconnect persists the configuration but the new connection fails
- **THEN** the system reports the connection error
- **AND** the updated configuration remains saved and the old main session remains available if it was active

#### Scenario: Invalid or missing connection
- **WHEN** the user submits an empty host or user, an invalid port, or a name that is no longer saved
- **THEN** the system rejects the update with a clear error and does not alter the saved configuration

#### Scenario: Persistence fails
- **WHEN** the configuration file cannot be written
- **THEN** the system reports the failure and does not update its in-memory configuration

#### Scenario: Existing background transfer
- **WHEN** an FTP transfer already has an independent session and the saved connection is edited
- **THEN** that transfer continues on its established session without interruption
