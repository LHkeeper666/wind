## Context

Archive extraction in Wind currently runs as a single blocking `spawn_blocking` call with no intermediate feedback. The user presses `e`, the UI freezes until extraction completes, and only then does a toast appear. For large archives (hundreds of MB, thousands of files), this creates a poor experience: no progress, no cancel, no confirmation the request was received.

The file transfer system (`src-tauri/src/transfer/`) already solves this problem for copy/move/delete/FTP operations. It has a `TransferScheduler` with queue management, slot-based concurrency, cooperative cancellation via `AtomicBool`, 100ms-throttled progress events, and a `TransferManager.svelte` UI with progress bars, speed, ETA, and keyboard navigation. The natural approach is to integrate extraction into this existing system rather than building parallel infrastructure.

## Goals / Non-Goals

**Goals:**
- All extraction operations (`e`, `E`+`p`, `y`+`p`) show progress in the Transfer Manager
- Extraction tasks are queued, cancellable, and persisted in transfer history
- Users see an immediate Transfer Manager entry when extraction is requested (before scanning begins)
- Extraction concurrency is independently controlled (CPU-bound vs IO-bound transfers)

**Non-Goals:**
- Compression progress (separate concern, not user-requested)
- Progress for individual file extraction within an archive (progress is per-file granularity, not per-byte within a file)
- Changing the conflict detection flow (remains frontend-driven, pre-extraction)
- Modifying the archive browsing experience beyond removing the `x` key

## Decisions

### Decision 1: Extend TransferScheduler with Extract variant

**Choice**: Add `TransferType::Extract` to the existing `TransferScheduler` rather than creating a separate `ExtractScheduler`.

**Rationale**:
- Maximizes code reuse: queue management, batch IDs, history persistence, event emission, cancel logic — all shared
- The `TransferScheduler` already has a clean slot abstraction (`can_take_slot` / `take_slot` / `free_slot_direct`) that extends naturally
- Frontend `TransferManager.svelte` already renders any `opType` — just needs `'extract'` added to the TypeScript union and label/icon mappings
- A separate scheduler would duplicate ~300 lines of queue/history/dispatch logic

**Alternative considered**: Separate `ExtractScheduler` with its own queue. Rejected because it doubles maintenance surface for no architectural benefit — the schedulers would share 90% of their code.

**Slot strategy**: Add `extract_slots_used` / `extract_max_slots` (default 2) alongside the existing `local_slots_used` / `ftp_conn_slots`. Extraction is CPU-bound (decompression), transfers are IO-bound (disk/network), so independent slot pools prevent mutual blocking.

```rust
// scheduler.rs additions
extract_slots_used: usize,
extract_max_slots: usize,  // default 2, configurable via transfer_set_extract_slots
```

### Decision 2: Progress callback injection into archive extractors

**Choice**: Add an `on_progress: &ExtractProgress` callback parameter to every format's `extract_all` and `extract_files` function.

**Type signature**:
```rust
pub type ExtractProgress = dyn Fn(u64) -> bool + Send + Sync;
//                          bytes_extracted  continue?
// Returns false to signal cancellation
```

**Rationale**:
- All 5 formats (zip, tar, tar.gz, 7z, rar) iterate entries in a loop — injecting a callback after each file's write is minimal code change
- The callback returns `bool` (continue?) rather than `Result` to keep the cancellation check simple: `if !on_progress(bytes) { return Err("Cancelled".into()); }`
- `Arc<dyn Fn>` makes the callback `Send + Sync` so it works across `spawn_blocking` boundaries
- 7z's `for_each_entries` closure captures the `Arc` by clone — returning `Ok(false)` from the closure stops iteration

**Alternative considered**: Wrapping `std::io::Write` in a progress-reporting writer for per-byte progress. Rejected because: (1) most time in compressed formats is spent on decompression, not writes, so write-tracking gives misleading progress; (2) adds complexity to every `File::create` call; (3) per-file granularity is sufficient UX.

### Decision 3: Two-phase enqueue (immediate entry + async scan)

**Choice**: Emit `transfer-progress` with `status: "queued"` and `total_bytes: 0` immediately on enqueue, then spawn a blocking task to scan archive entries for sizes, updating `total_bytes` when ready.

**Flow**:
```
T+0ms    invoke('extract_enqueue') → emit queued (total_bytes=0)
T+1ms    spawn_blocking: open archive, sum entry sizes
T+???ms  emit queued (total_bytes=actual) → slot available → dispatch
```

**Rationale**:
- User requirement: "application should acknowledge the request immediately"
- Transfer Manager shows "scanning..." when `total_bytes: 0` and status is queued
- Scanning is fast for all formats (headers are at the start of the file, sizes are in central directory for ZIP, entry headers for TAR/7z/RAR)
- Frontend doesn't need to compute sizes — the backend does it after receiving the command

### Decision 4: Frontend conflict detection stays as-is

**Choice**: Keep the existing frontend-driven conflict detection in `handleExtractHere` (collects archive paths, collects existing paths, prompts per conflict). Only the final extraction call changes from `extractArchive()` to `invoke('extract_enqueue')`.

**Rationale**:
- Conflict detection requires user interaction (prompt dialogs) — this is inherently frontend
- The existing streaming conflict flow works well and is already spec'd
- Backend doesn't need to know about conflicts — `skip_paths` is passed as a parameter

### Decision 5: Remove `x` key, keep `y`→`p`

**Choice**: Remove `handleArchiveExtract()` and the `x` key binding from `DirectoryPanel.svelte`. Archive entry extraction uses `y` (yank) → navigate → `p` (paste).

**Rationale**:
- User decision: simplify to one extraction flow for selected entries
- `y`→`p` is consistent with the general copy-paste metaphor
- Reduces keybinding surface area in archive mode

## Risks / Trade-offs

**[Risk] 7z/RAR extraction reads entire file into memory per entry**
→ Mitigation: These formats already do this in the current implementation. Progress callback doesn't change memory behavior. For very large individual files (>1GB), this is an existing limitation, not introduced by this change.

**[Risk] Archive scanning adds latency before extraction starts**
→ Mitigation: Scanning reads only headers (no decompression), typically <100ms even for large archives. The "scanning..." state in Transfer Manager provides clear feedback during this phase.

**[Risk] Cancellation leaves partial files on disk**
→ Mitigation: This is the explicit user decision ("keep partial output"). Consistent with how transfer cancellation works (doesn't delete already-copied files). Documented in Transfer Manager's cancel behavior.

**[Trade-off] Per-file progress granularity vs per-byte**
→ Per-file is simpler and sufficient. A 1000-file archive shows smooth progress. A single 2GB file shows a jump — acceptable for v1.

**[Trade-off] Transfer history includes extraction tasks**
→ Extraction records mix with transfer records in history. Acceptable since they share the same UI and lifecycle. Could add a filter in the future if needed.

## Migration Plan

1. Backend changes first (archive callbacks, scheduler extension, new command)
2. Frontend changes second (store, UI, extraction paths)
3. Remove old `extract_archive` / `extract_archive_files` Tauri commands if no longer used externally, or keep as internal-only helpers
4. No data migration needed — `transfer-history.json` format is backward-compatible (new `extract` type is just a new enum variant)

## Open Questions

- Should `extract_enqueue` return before or after the scanning phase? Currently designed to return immediately (task ID), scanning happens async. This means the frontend gets the ID before `total_bytes` is known — acceptable since the entry already shows in the UI.
- Should the `extract_archive` command be removed entirely, or kept as a simpler non-scheduler path for programmatic use? Recommendation: remove, consolidate on `extract_enqueue`.