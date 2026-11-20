## ADDED Requirements

### Requirement: FTP transfer sessions are independent and on demand
The system SHALL create an independent FTP session for each background task that needs FTP access, using a snapshot of the saved configuration at task start without holding the FTP manager lock during network waits.

#### Scenario: Transfer starts without main session
- **WHEN** an FTP transfer starts for a saved site with no main session
- **THEN** it establishes its own session and runs without first creating a main session

#### Scenario: Transfer and configuration edit overlap
- **WHEN** an FTP task waits for network connection while the user saves site settings
- **THEN** the save can complete without waiting for that network operation
- **AND** the running task uses its start-time configuration while later tasks use the updated configuration

#### Scenario: Transfer and site removal overlap
- **WHEN** a site is removed after a transfer has started
- **THEN** the running task continues on its independent session, while a task starting afterward cannot obtain the removed configuration
