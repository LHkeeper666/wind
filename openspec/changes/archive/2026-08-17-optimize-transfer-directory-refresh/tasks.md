## 1. Directory Identity and Mutation Adapter

- [x] 1.1 Define the typed `DirectoryKey` representation and lexical normalization rules for local paths and FTP connection/path locations.
- [x] 1.2 Implement parent-directory derivation for local and FTP file locations without filesystem or network lookups.
- [x] 1.3 Implement the legacy transfer-event adapter that maps `transfer-complete`, `transfer-failed`, and `transfer-cancelled` payloads to deduplicated `DirectoryMutation` values.
- [x] 1.4 Cover copy, move, delete, FTP upload, FTP download, and FTP delete outcomes, including conservative affected-directory handling for failed and cancelled transfers.
- [x] 1.5 Define the event-source-neutral mutation input boundary so future structured `affectedDirectories`, `outcome`, and `batchId` events can bypass legacy payload parsing.

## 2. Directory Cache and Version Tracking

- [x] 2.1 Add precise cache invalidation by normalized `DirectoryKey` without clearing unrelated directory entries.
- [x] 2.2 Add a monotonic mutation version registry for each normalized directory identity.
- [x] 2.3 Add per-panel observed directory version tracking and update it only after a directory load or fresh-cache synchronization succeeds.
- [x] 2.4 Preserve stale versions after failed synchronization so later tab activation, manual refresh, or a new mutation can retry.

## 3. Refresh Coordinator

- [x] 3.1 Implement the dirty-directory set and deduplicate mutations by normalized directory identity.
- [x] 3.2 Implement trailing debounce and maximum-wait scheduling with configurable defaults, and cancel all timers during coordinator disposal.
- [x] 3.3 Match affected directories against the active tab's actual current and left panel paths at flush time.
- [x] 3.4 Ensure one flush refreshes each matching panel at most once and does not refresh unrelated active panels.
- [x] 3.5 Invalidate affected cache entries immediately while deferring directory reads for invisible or background directories.
- [x] 3.6 Detect the first settled state for each batch from existing transfer state and perform a one-time final flush for the batch's affected-directory union.
- [x] 3.7 Add support for a future structured `transfer-batch-settled` signal without changing panel matching, cache invalidation, or tab version policies.

## 4. Panel and Tab Integration

- [x] 4.1 Replace the unconditional transfer listeners in `PanelLayout.svelte` with the mutation adapter and refresh coordinator lifecycle.
- [x] 4.2 Wire current and left `DirectoryPanel` instances, including automatic-parent and detached-left-panel modes, to report displayed directory identity and observed version.
- [x] 4.3 Check directory versions during tab activation and synchronize stale current/left panels without starting reads for inactive tabs.
- [x] 4.4 Ensure multiple tabs showing the same directory track independent observed versions while sharing the latest valid cache entry.
- [x] 4.5 Preserve existing manual refresh behavior and request-generation protection when coordinator-triggered refreshes race with navigation.

## 5. Verification

- [ ] 5.1 Add deterministic coverage for local/FTP path normalization, parent-directory derivation, operation-to-directory mapping, and failed/cancelled partial-result rules.
- [ ] 5.2 Add coverage for dirty-directory deduplication, debounce, maximum wait, batch-final flush, duplicate settled notifications, and coordinator disposal.
- [ ] 5.3 Add coverage for active current/left panel matching, unrelated-panel suppression, tab changes before flush, background-tab deferral, stale-version activation, and same-directory multi-tab behavior.
- [ ] 5.4 Run `npm run check` and manually verify local multi-file copy/move/delete plus FTP upload/download/delete across active, unrelated, background, and same-directory tabs.
