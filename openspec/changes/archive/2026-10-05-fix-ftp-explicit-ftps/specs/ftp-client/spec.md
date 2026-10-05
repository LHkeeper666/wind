## MODIFIED Requirements

### Requirement: FTP MKD command compatibility
The system SHALL successfully create directories on FTP servers that return non-standard status codes for MKD, and SHALL use CWD + relative path MKD as fallback when absolute path MKD is rejected.

#### Scenario: MKD with non-standard 250 response
- **WHEN** the server returns `250 Directory created` for a MKD command
- **THEN** the system SHALL treat this as success (not error)

#### Scenario: MKD with 550 on absolute path
- **WHEN** MKD with absolute path (e.g., `/Android/obb/xxx`) returns 550
- **THEN** the system SHALL CWD to the parent directory and retry MKD with relative path
- **AND** if the relative MKD also fails, probe with CWD to check if directory already exists
- **AND** if CWD succeeds, treat as success (directory already exists)

#### Scenario: Directory existence detection via CWD
- **WHEN** a MKD command fails with any error
- **THEN** the system SHALL use CWD to probe whether the directory already exists
- **AND** CWD success means the directory exists (continue)
- **AND** CWD failure means the directory does not exist (proceed with creation)

### Requirement: FTP session cleanup
The system SHALL properly close independent FTP sessions after transfer operations complete.

#### Scenario: Upload session cleanup
- **WHEN** an FTP upload completes (success or failure)
- **THEN** the system SHALL send QUIT command to close the session

#### Scenario: Download session cleanup
- **WHEN** an FTP download completes (success or failure)
- **THEN** the system SHALL send QUIT command to close the session

#### Scenario: Delete session cleanup
- **WHEN** an FTP delete completes (success or failure)
- **THEN** the system SHALL send QUIT command to close the session