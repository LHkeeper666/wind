## Why

Archive extraction currently runs as a blocking, fire-and-forget operation with no progress feedback. Users extracting large archives see no indication that the system received their request, have no visibility into extraction progress, and cannot cancel a long-running extraction. The Transfer Manager already provides a robust infrastructure for progress tracking, queue management, and cancellation that file transfers use — extraction should leverage the same system.

## What Changes

- **TransferType::Extract**: Add a new `Extract` variant to the transfer type enum so extraction tasks flow through the existing TransferScheduler with queue, slot, and history support.
- **Archive progress callbacks**: Modify all 5 archive format extractors (zip, tar, tar.gz, 7z, rar) to accept a progress callback and cancellation flag, emitting progress after each file is extracted.
- **New `extract_enqueue` Tauri command**: Replace direct `extract_archive` / `extract_archive_files` invocations with a scheduler-backed command that immediately shows a queued entry in the Transfer Manager.
- **Frontend extraction paths unified**: All three extraction triggers (`e` extract-here, `E`+`p` mark-extract, `y`+`p` yank-paste) go through the same `extract_enqueue` → TransferScheduler pipeline.
- **Remove `x` key extraction**: The `x` key in archive browser mode is removed. Extracting selected archive entries uses `y` → `p` instead.
- **Immediate UI feedback**: When extraction is enqueued, a Transfer Manager entry appears instantly (total_bytes=0, "scanning..."). Once the backend scans archive entries for sizes, the entry updates with actual total_bytes.

## Capabilities

### New Capabilities

- `extract-progress`: Extraction operations integrated into the transfer scheduler with progress tracking, cancellation, and Transfer Manager display.

### Modified Capabilities

- `transfer-manager`: Add `extract` as a new operation type alongside copy/move/delete/ftp-download/ftp-upload. The Transfer Manager displays extraction entries with the same progress bar, speed, ETA, and cancel UI.
- `archive-operations`: Remove `x` key extraction. All extraction paths go through the scheduler. Backend extract functions gain progress callbacks and cancellation support.

## Impact

- **Rust backend**: `transfer/mod.rs`, `transfer/scheduler.rs` — extend with extract support. `archive/mod.rs` and all 5 format files — add callback parameter. `commands/archive_cmd.rs` — new command, old commands may become internal-only.
- **Svelte frontend**: `stores/transfer.ts` — add `'extract'` to opType union. `TransferManager.svelte` — add label/icon for extract. `DirectoryPanel.svelte` — modify `handleExtractHere`, remove `x` key handler. `clipboard-operations.ts` — change paste extract path. `archive-browser.ts` — redirect extract calls.
- **No new dependencies**: Uses existing `zip`, `tar`, `flate2`, `sevenz-rust`, `unrar` crates. No new Rust or npm packages needed.
- **History**: Extraction tasks appear in transfer history alongside file transfers.