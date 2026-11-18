## 1. Move orchestration

- [x] 1.1 Add a move group state model that records source, destination, child task IDs, verification, cleanup, and terminal outcome.
- [x] 1.2 Add completion and cancellation barriers so no source deletion can run while a child transfer is queued, running, failed, or cancelled.
- [x] 1.3 Implement destination verification and source-change checks before cleanup; report a distinct partial-move result if cleanup fails.

## 2. Same-connection FTP moves

- [x] 2.1 Keep the rename fast path, validate same-connection paths and directory ancestry, and detect source type before fallback.
- [x] 2.2 Replace file-only `ftp_copy_move_fallback` with tracked file transfer that finalizes data streams, bounds waits, and cleans temporary files.
- [x] 2.3 Implement recursive directory fallback with independent FTP sessions, target directory creation, child file transfers, empty-folder handling, and bottom-up source cleanup.
- [x] 2.4 Apply existing paste conflict choices before rename or fallback; make partial destination retry explicit.

## 3. Cross-backend cut safety

- [x] 3.1 Route FTP-to-local single-file and folder cuts through the move completion barrier; remove immediate remote deletion after enqueue.
- [x] 3.2 Route local-to-FTP single-file and folder cuts through the same barrier; remove immediate local deletion after enqueue.
- [x] 3.3 Ensure failed, cancelled, and empty-folder transfers preserve or clean sources according to the specs.

## 4. Frontend outcomes and verification

- [x] 4.1 Show queued status while work is running and count only completed moves as moved.
- [x] 4.2 Retain failed and partially moved cut entries for retry, refresh both listings at terminal outcome, and show actionable errors.
- [x] 4.3 Add Rust tests for rename failure, directory RETR prevention, empty and nested folders, partial transfer, cancellation, cleanup failure, and source changes.
- [x] 4.4 Add frontend tests for mixed success/failure, conflict choices, queued status, and clipboard retention.
- [x] 4.5 Run `cargo check`, targeted Rust tests, `npx svelte-check`, and targeted frontend tests; inspect logs for no directory RETR and no premature source deletion. Manual FTP move verification passed on 2026-11-18.
