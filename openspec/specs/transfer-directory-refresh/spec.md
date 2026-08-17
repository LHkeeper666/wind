# transfer-directory-refresh Specification

## Purpose

Define consistent, targeted directory cache invalidation and panel refresh behavior after transfer mutations.

## Requirements

### Requirement: Derive affected directories from transfer terminal events
The system SHALL derive a deduplicated set of affected directory identities from each transfer terminal event using its operation type, source, destination, and outcome. The derivation SHALL conservatively include directories that may contain partial changes after failed or cancelled operations.

#### Scenario: Local copy reaches a terminal state
- **WHEN** a local copy transfer completes, fails, or is cancelled
- **THEN** the destination parent directory is marked affected
- **AND** the source parent directory is not marked affected solely because of the copy

#### Scenario: Local move reaches a terminal state
- **WHEN** a local move transfer completes, fails, or is cancelled
- **THEN** both the source parent directory and destination parent directory are marked affected

#### Scenario: Local delete reaches a terminal state
- **WHEN** a local delete transfer completes, fails, or is cancelled
- **THEN** the source parent directory is marked affected

#### Scenario: FTP download reaches a terminal state
- **WHEN** an FTP download completes, fails, or is cancelled
- **THEN** the local destination parent directory is marked affected
- **AND** the FTP source parent directory is not marked affected solely because of the download

#### Scenario: FTP upload reaches a terminal state
- **WHEN** an FTP upload completes, fails, or is cancelled
- **THEN** the FTP destination parent directory is marked affected
- **AND** the local source parent directory is not marked affected solely because of the upload

#### Scenario: FTP delete reaches a terminal state
- **WHEN** an FTP delete completes, fails, or is cancelled
- **THEN** the FTP source parent directory is marked affected

### Requirement: Normalize directory identities
The system SHALL normalize local and FTP directory locations into stable typed directory identities before comparing paths, invalidating cache entries, or scheduling panel refreshes. Normalization SHALL be lexical and MUST NOT require the directory to exist or perform a network request.

#### Scenario: Equivalent Windows local paths are compared
- **WHEN** two local directory paths differ only by slash direction, drive-letter case, path case, removable trailing separators, or lexically reducible path segments
- **THEN** the system treats them as the same directory identity

#### Scenario: Distinct Windows roots remain distinct
- **WHEN** two normalized local paths refer to different drive roots or different path segments
- **THEN** the system treats them as different directory identities

#### Scenario: Equivalent FTP paths are compared
- **WHEN** two FTP directory locations use the same connection identity and differ only by repeated separators, removable trailing separators, or lexically reducible remote path segments
- **THEN** the system treats them as the same directory identity

#### Scenario: FTP connection identity participates in comparison
- **WHEN** two FTP directory locations have the same normalized remote path but different connection identities
- **THEN** the system treats them as different directory identities

### Requirement: Invalidate affected directory cache precisely
The system SHALL invalidate cached directory data for every affected directory immediately after accepting a directory mutation, regardless of whether that directory is visible. The system MUST NOT invalidate unrelated directory entries.

#### Scenario: Invisible affected directory is invalidated
- **WHEN** a transfer mutates a directory that is only referenced by a background tab or is not currently open
- **THEN** the matching directory cache entry is invalidated
- **AND** no directory read is started solely because the directory is invisible

#### Scenario: Unrelated cache entry remains valid
- **WHEN** a transfer mutation affects directory A while directory B has a cached listing
- **THEN** directory B's cache entry remains valid

#### Scenario: Multiple events affect the same directory
- **WHEN** multiple terminal events normalize to the same directory identity
- **THEN** the system invalidates and tracks that directory as one logical dirty directory

### Requirement: Refresh only matching panels in the active tab
The system SHALL immediately schedule automatic refreshes only for directory panels in the active tab whose displayed directory identity matches an affected directory. Matching SHALL use each panel's actual displayed path at flush time.

#### Scenario: Active current panel matches an affected directory
- **WHEN** the active tab's current panel displays an affected directory
- **THEN** the system schedules that current panel for refresh

#### Scenario: Active left panel matches an affected directory
- **WHEN** the active tab's left panel displays an affected directory
- **THEN** the system schedules that left panel for refresh
- **AND** matching uses the left panel's actual path whether it is an automatic parent panel or a detached manual panel

#### Scenario: Both active panels match affected directories
- **WHEN** the active tab's current and left panels each display an affected directory
- **THEN** the system schedules both matching panels for refresh
- **AND** each panel is refreshed at most once during one flush

#### Scenario: Active tab is unrelated to the transfer
- **WHEN** neither directory panel in the active tab displays an affected directory
- **THEN** the system does not refresh either panel

#### Scenario: User changes tabs or directories before a pending flush
- **WHEN** an affected directory is queued for refresh and the user changes the active tab or displayed panel path before the refresh window flushes
- **THEN** the system evaluates the active tab and actual panel paths at flush time
- **AND** it does not refresh a panel based on a stale captured tab or path

### Requirement: Coalesce repeated directory refreshes
The system SHALL coalesce repeated mutations by normalized directory identity using a trailing debounce and a maximum wait. Continuous transfer activity MUST NOT postpone visible refreshes indefinitely.

#### Scenario: Burst of completions affects one visible directory
- **WHEN** multiple files affecting the same visible directory reach terminal states within the debounce window
- **THEN** the system performs one coalesced refresh for that panel after the burst

#### Scenario: Concurrent mutations affect two visible panels
- **WHEN** terminal events affect the active current and left panel directories during the same aggregation window
- **THEN** the system performs at most one refresh for each affected panel during the flush

#### Scenario: Transfer events continue beyond maximum wait
- **WHEN** matching mutations continue without a debounce pause for at least the configured maximum wait
- **THEN** the system flushes pending refreshes no later than the maximum-wait boundary
- **AND** subsequent mutations begin or join a new aggregation window

#### Scenario: Pending timers are disposed
- **WHEN** the refresh coordinator is destroyed
- **THEN** all debounce and maximum-wait timers are cancelled
- **AND** no delayed panel refresh runs after destruction

### Requirement: Perform final refresh when a batch settles
The system SHALL detect when all transfers in a batch have reached completed, failed, or cancelled states and SHALL immediately flush any pending affected directories for that batch. A batch MUST trigger its final flush at most once.

#### Scenario: Last transfer in a batch completes
- **WHEN** the last queued or running transfer in a batch reaches a terminal state
- **THEN** the system immediately flushes pending refreshes associated with that batch without waiting for the remaining debounce delay

#### Scenario: Batch settles with mixed outcomes
- **WHEN** all transfers in a batch are terminal with a mixture of completed, failed, and cancelled outcomes
- **THEN** the system performs the same final flush using the union of directories affected by those outcomes

#### Scenario: Duplicate terminal notification is observed
- **WHEN** a settled batch receives or exposes a duplicate terminal-state observation
- **THEN** the system does not perform a second batch-final flush

### Requirement: Preserve directory consistency across tabs
The system SHALL maintain a mutation version for each normalized directory identity and an observed version for each tab directory panel. A background panel whose observed version is older than the directory mutation version SHALL synchronize when its tab becomes active.

#### Scenario: Background tab contains an affected directory
- **WHEN** a transfer mutates a directory displayed by a background tab
- **THEN** the background tab performs no immediate directory read
- **AND** its panel remains identifiable as stale through its observed directory version

#### Scenario: User activates a stale background tab
- **WHEN** the user activates a tab whose current or left panel has an observed version older than its displayed directory's mutation version
- **THEN** each stale panel synchronizes with the latest directory listing
- **AND** each panel records the new observed version only after synchronization succeeds

#### Scenario: Multiple tabs display the same affected directory
- **WHEN** multiple tabs display the same directory and that directory is mutated
- **THEN** the active matching panel may refresh immediately
- **AND** refreshing the active panel does not mark background panels as having observed the new directory version
- **AND** each background panel synchronizes when its tab becomes active

#### Scenario: Fresh shared cache is available on tab activation
- **WHEN** a stale background panel is activated after another panel has already loaded the same directory mutation version into the shared cache
- **THEN** the activated panel may reuse that fresh cache entry instead of issuing another directory read
- **AND** it still updates its own rendered listing and observed version

#### Scenario: Background refresh fails after activation
- **WHEN** a stale panel cannot load its directory after its tab becomes active
- **THEN** the system keeps that panel's observed version unchanged
- **AND** a later activation, explicit refresh, or directory mutation can retry synchronization

### Requirement: Accept event-source-neutral directory mutations
The directory refresh coordinator SHALL consume an event-source-neutral mutation contract containing `batchId`, `outcome`, and normalized `affectedDirectories`. Existing transfer terminal events SHALL be converted through an adapter before reaching the coordinator.

#### Scenario: Existing transfer event is adapted
- **WHEN** the frontend receives an existing `transfer-complete`, `transfer-failed`, or `transfer-cancelled` event
- **THEN** the adapter derives a normalized directory mutation
- **AND** the coordinator processes it without parsing the original transfer operation or raw paths

#### Scenario: Structured transfer-layer mutation is supplied
- **WHEN** a future transfer-layer event directly supplies `batchId`, `outcome`, and `affectedDirectories`
- **THEN** the same coordinator can process the mutation without changing cache, aggregation, panel matching, or tab consistency behavior

#### Scenario: Structured batch-settled signal is supplied
- **WHEN** a future transfer layer supplies a batch-settled signal with its final affected-directory union
- **THEN** the signal can trigger the same batch-final flush behavior without changing the refresh coordinator's panel and cache policies
