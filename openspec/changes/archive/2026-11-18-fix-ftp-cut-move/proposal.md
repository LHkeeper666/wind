## Why

On 2026-11-18 around 10:46, moving an FTP directory failed three times: the server rejected rename with 550, then the file-only fallback attempted RETR on a directory and never reached upload or source cleanup. The current cut flows can also delete sources before queued transfers succeed and report success for failed items, creating data-loss and misleading-feedback risks.

## What Changes

- Support same-connection FTP file and directory moves when server-side rename fails, with recursive directory transfer and progress/cancellation visibility.
- Delete source content only after the destination has been fully transferred and verified; preserve the source on failure or cancellation across FTP/local cut transfers.
- Handle empty folders, destination conflicts, partial failure, and stale FTP sessions explicitly.
- Report actual per-item outcomes; keep failed cut entries available for retry instead of reporting all items moved.

## Capabilities

### New Capabilities

- `ftp-move`: Safe same-connection FTP file and directory moves, including rename fallback and source cleanup.

### Modified Capabilities

- `ftp-folder-transfer`: Ensure cut-mode cleanup occurs only after complete successful transfer, including empty folders.
- `file-clipboard`: Retain failed cut entries and show accurate results for queued and completed moves.

## Impact

Affected areas include `src-tauri/src/commands/ftp_cmd.rs`, FTP transfer scheduling and completion handling, and `src/lib/utils/clipboard-operations.ts` plus transfer UI state. No new external dependency or database change is expected.
