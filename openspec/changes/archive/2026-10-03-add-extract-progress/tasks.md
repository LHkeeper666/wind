## 1. Archive module: progress callbacks and cancellation

- [x] 1.1 Define `ExtractProgress` type in `archive/mod.rs` (`Arc<dyn Fn(u64) -> bool + Send + Sync>`)
- [x] 1.2 Add `on_progress` parameter to `zip::extract_all` and `zip::extract_files` — call callback after each file write, check return value for cancellation
- [x] 1.3 Add `on_progress` parameter to `tar::extract_all_impl` and `tar::extract_files_impl` — same pattern
- [x] 1.4 Add `on_progress` parameter to `seven_z::extract_all` and `seven_z::extract_files` — capture `Arc` in `for_each_entries` closure, return `Ok(false)` on cancel
- [x] 1.5 Add `on_progress` parameter to `rar::extract_all` and `rar::extract_files` — check cancel in header loop
- [x] 1.6 Update `archive/mod.rs` dispatcher functions (`extract_all`, `extract_files`) to accept and forward `on_progress`
- [x] 1.7 Verify: `cargo check` passes with all archive module changes

## 2. Transfer scheduler: Extract variant and slot management

- [x] 2.1 Add `Extract` variant to `TransferType` enum in `transfer/mod.rs` with `#[serde(rename_all = "kebab-case")]`
- [x] 2.2 Add `extract_slots_used` and `extract_max_slots` fields to `TransferScheduler`
- [x] 2.3 Implement `can_take_slot`, `take_slot`, `free_slot_direct` for `TransferType::Extract`
- [x] 2.4 Add `set_extract_max_slots` and update `get_slot_config` to include extract slots
- [x] 2.5 Verify: `cargo check` passes

## 3. Transfer execution: extract task handler

- [x] 3.1 Create `transfer/extract.rs` with `execute_extract` function that: opens archive, calls `extract_all`/`extract_files` with progress callback that emits `transfer-progress` events (100ms throttle) and checks cancel flag
- [x] 3.2 Add `Extract` arm to `execute_transfer` dispatcher in `transfer/mod.rs`
- [x] 3.3 Add `enqueue_extract` method to `TransferScheduler`: accepts archive_path, dest_dir, password, optional internal_paths, optional skip_paths; immediately emits queued event (total_bytes=0); spawns blocking scan task to compute total_bytes; dispatches when slot available
- [x] 3.4 Verify: `cargo check` passes

## 4. Tauri command: extract_enqueue

- [x] 4.1 Add `extract_enqueue` command in `commands/archive_cmd.rs` that calls into TransferScheduler
- [x] 4.2 Register `extract_enqueue` in `lib.rs` command list
- [x] 4.3 Add `transfer_set_extract_slots` command for slot configuration
- [x] 4.4 Verify: `cargo check` passes

## 5. Frontend: store and type updates

- [x] 5.1 Add `'extract'` to `TransferEntry.opType` union type in `stores/transfer.ts`
- [x] 5.2 Add `case 'extract': return 'Extract'` to `opTypeLabel` function
- [x] 5.3 Update `getStatusIcon` to handle extract (e.g., running icon for extract)
- [x] 5.4 Add `enqueueExtract` function to the transfer store that invokes `extract_enqueue`
- [x] 5.5 Verify: `npx svelte-check` passes

## 6. Frontend: TransferManager UI

- [x] 6.1 Verify "scanning..." display when `totalBytes === 0` and `status === 'queued'` works with existing template logic (or add if needed)
- [x] 6.2 Test that extract entries render correctly with progress bar, speed, ETA, cancel button

## 7. Frontend: wire extraction paths to scheduler

- [x] 7.1 Update `handleExtractHere` in `DirectoryPanel.svelte`: after conflict detection, call `transfer.enqueueExtract(...)` instead of `extractArchive(...)`
- [x] 7.2 Update `handlePaste` in `clipboard-operations.ts`: extract mark path (`E`+`p`) calls `transfer.enqueueExtract(...)` instead of `invokeArchiveWithOptionalPassword('extract_archive', ...)`
- [x] 7.3 Update `handlePaste` in `clipboard-operations.ts`: archive yank path (`y`+`p`) calls `transfer.enqueueExtract(...)` with `internal_paths` instead of `invokeArchiveWithOptionalPassword('extract_archive_files', ...)`
- [x] 7.4 Open Transfer Manager automatically when extraction is enqueued (call `onOpenTransfer`)

## 8. Remove x key extraction

- [x] 8.1 Remove `handleArchiveExtract` function from `DirectoryPanel.svelte`
- [x] 8.2 Remove `case 'x'` keybinding from archive mode keydown handler in `DirectoryPanel.svelte`
- [x] 8.3 Remove `extractArchiveFiles` usage from `archive-browser.ts` if no longer used elsewhere

## 9. Cleanup and verification

- [x] 9.1 Run `cargo check` to verify all Rust changes compile
- [x] 9.2 Run `npx svelte-check` to verify all TypeScript/Svelte changes pass
- [x] 9.3 Test `e` key extraction: entry appears in Transfer Manager, progress updates, completes
- [x] 9.4 Test `E`+`p` extraction: same flow with mark-and-paste
- [x] 9.5 Test `y`+`p` extraction: yank from archive browser, paste extracts with progress
- [x] 9.6 Test cancellation: cancel running extraction, verify partial files preserved
- [x] 9.7 Test concurrent extractions: queue multiple, verify slot limits
- [x] 9.8 Test transfer history: extraction records appear after completion