## 1. Backend: file_ops module

- [x] 1.1 Create `src-tauri/src/file_ops.rs` with `FileOp`, `FileOpRegistry`, `Progress` helper
- [x] 1.2 Implement `copy_file_chunked()` — 64KB chunk loop with cancel check and progress callback
- [x] 1.3 Implement `scan_directory()` — traverse tree, count total files and total bytes, emit `op-scan-complete`
- [x] 1.4 Implement event throttling: min 100ms between `op-progress` emissions
- [x] 1.5 Integrate `file_ops` module into `lib.rs`

## 2. Backend: async commands

- [x] 2.1 Add `copy_file_async` command with spawn_blocking, progress events, cancel support
- [x] 2.2 Add `move_file_async` command — same-drive rename (instant) or cross-drive async copy+delete
- [x] 2.3 Add `delete_file_async` command — async directory deletion with file-count progress
- [x] 2.4 Add `cancel_file_op` command — set AtomicBool flag, clean up partial files
- [x] 2.5 Add `check_copy_conflicts` command — pre-check destination existence, return conflict list
- [x] 2.6 Register all new commands in invoke_handler

## 3. Frontend: FileOpProgress component

- [x] 3.1 Create `src/lib/components/FileOpProgress.svelte` — bottom status bar container
- [x] 3.2 Render progress bars: icon, filename, percentage, bytes/total, cancel button
- [x] 3.3 Handle `op-scan-complete`, `op-progress`, `op-complete`, `op-failed`, `op-cancelled` events
- [x] 3.4 Support multiple concurrent operations (stacked bars)
- [x] 3.5 Auto-dismiss: green "✓ Done" → 3s fadeout; red "✕ Error" → stays until dismissed
- [x] 3.6 Style: dark theme, monospace font, var(--accent) for active, var(--success) for done, var(--warning) for error

## 4. Frontend: integration

- [x] 4.1 Add `FileOpProgress` to `PanelLayout.svelte` layout
- [x] 4.2 Update paste flow: conflict resolution → async batch copy/move with `copy_file_async` / `move_file_async`
- [x] 4.3 Delete flow uses async via `delete_file_async` (directories get progress, single files fast)
- [x] 4.4 Keep existing sync commands available as fallback

## 5. Cleanup & Polish

- [x] 5.1 Run `cargo check` and `npx svelte-check` — fix all errors
- [ ] 5.2 Test: copy 1GB+ file, verify progress bar updates
- [ ] 5.3 Test: cancel mid-operation, verify partial file cleaned up
- [ ] 5.4 Test: conflict pre-check with existing destination files
- [ ] 5.5 Test: two concurrent copy operations
