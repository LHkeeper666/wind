## ADDED Requirements

### Requirement: Accurate cut completion feedback
The system SHALL distinguish queued, completed, failed, and partially completed cut operations using actual transfer outcomes.

#### Scenario: Cut item queued
- **WHEN** a cut operation starts asynchronous transfer work
- **THEN** the system reports the work as queued rather than moved

#### Scenario: Some cut entries fail
- **WHEN** one or more clipboard entries fail to move
- **THEN** the system reports the successful and failed counts accurately and retains failed entries for retry

#### Scenario: Source cleanup fails
- **WHEN** destination transfer succeeds but source deletion fails
- **THEN** the system reports a partial move and retains the affected entry for retry

### Requirement: Safe single-file cross-backend cut
The system SHALL delete a source file in an FTP-to-local or local-to-FTP cut only after its destination transfer succeeds and is verified.

#### Scenario: FTP download fails
- **WHEN** a cut FTP file fails to download or is cancelled
- **THEN** the system keeps the remote source file and its cut clipboard entry

#### Scenario: Local upload fails
- **WHEN** a cut local file fails to upload or is cancelled
- **THEN** the system keeps the local source file and its cut clipboard entry
