## Context

Wind is a Tauri 2 desktop application. Currently all file operations (`copy_file`, `move_file`, `delete_file`, `permanent_delete`) run synchronously in the Tauri command handler on the Rust thread pool. Large operations (e.g., copying a 2GB file or `node_modules`) block the frontend completely — no interaction, no feedback, the app appears hung.

Tauri 2 supports async commands which can use `tokio::task::spawn_blocking` to offload heavy work. The frontend uses Svelte 5 runes for reactive components. Backend→frontend communication uses `app_handle.emit()` for events (the terminal module already follows this pattern).

## Goals / Non-Goals

**Goals:**
- Make copy/move/delete non-blocking via async Tauri commands
- Report byte-level progress for single file copies, file-count progress for directories
- Support cancellation via `AtomicBool` flag per operation
- Pre-check: detect destination conflicts before execution, let user resolve
- Phase 1 scan: compute total size before copying to show accurate progress
- Multiple concurrent operations (parallel via thread pool)
- Bottom status bar showing per-operation progress bar + cancel button
- Completed operations auto-dismiss after 3 seconds

**Non-Goals:**
- Application-level file locking (same as yazi/Explorer — user is responsible)
- Drag-and-drop file operations (existing mechanism preserved)
- Operation queuing / priority (all ops run immediately in parallel)

## Decisions

### 1. Async pattern: `tokio::task::spawn_blocking`

The existing `search_files` command already uses this pattern. We follow it: the command is `async fn`, inside we call `spawn_blocking(move || { ... })` with the actual I/O work, emitting progress events via the cloned `AppHandle`.

**Alternative considered**: A dedicated background thread with a channel (mpsc). Rejected — `spawn_blocking` is simpler and Tauri already manages the thread pool.

### 2. Chunked I/O for single-file copy

Instead of `fs::copy()`, we implement a custom `copy_file_chunked()` that reads 64KB chunks and emits progress after each chunk. This gives byte-level granularity.

```rust
fn copy_file_chunked(
    src: &Path, dst: &Path, progress: &Progress,
) -> io::Result<u64> {
    let mut reader = File::open(src)?;
    let src_size = reader.metadata()?.len();
    let mut writer = File::create(dst)?;
    let mut buf = [0u8; 65536];
    let mut written: u64 = 0;
    loop {
        if progress.cancelled() { return Err(Cancelled); }
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        writer.write_all(&buf[..n])?;
        written += n as u64;
        progress.report(written, src_size);
    }
    Ok(written)
}
```

### 3. Three-phase flow

```
Pre-check → Scan → Execute
```

- **Pre-check**: Synchronous metadata scan to detect filename conflicts. Retained as blocking (it's fast — just path existence checks).
- **Scan**: `spawn_blocking` that traverses the directory tree, counts total files and total bytes. Emits `op-scan` event when done so the frontend knows the 100% target.
- **Execute**: `spawn_blocking` that does the actual chunked copy/move/delete, emitting `op-progress` per chunk.

### 4. State management: `FileOpRegistry`

A global `HashMap<u64, FileOp>` behind a `Mutex`:

```rust
struct FileOp {
    id: u64,
    op_type: OpType,       // Copy, Move, Delete
    status: OpStatus,      // Scanning, Running, Done, Cancelled, Failed
    bytes_done: u64,
    total_bytes: u64,
    files_done: u32,
    total_files: u32,
    current_file: String,
    cancel_flag: Arc<AtomicBool>,
}

static FILE_OPS: Lazy<Mutex<HashMap<u64, FileOp>>> = Lazy::new(|| ...);
static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);
```

### 5. Event protocol

Each Tauri event has a `type` discriminator:

- `op-scan-complete`: `{ id, total_bytes, total_files }` — scan phase done, frontend now knows total
- `op-progress`: `{ id, bytes_done, total_bytes, files_done, total_files, current_file }` — per-chunk/per-file update
- `op-complete`: `{ id }` — operation finished successfully
- `op-failed`: `{ id, error }` — operation errored
- `op-cancelled`: `{ id }` — user cancelled

### 6. Cancellation model

Each operation has an `Arc<AtomicBool>`. The `copy_file_chunked` loop checks it every 64KB. A new command `cancel_file_op(id)` sets the flag. After cancellation, we clean up partial destination files.

### 7. Frontend: `FileOpProgress.svelte`

A bottom status bar component that listens to `op-*` events:

- Renders one progress bar per active operation
- Shows: operation icon, file name, progress bar, percentage, current/total bytes
- Cancel button (✕) on each bar
- Completed: green "✓ Done" → 3s → fade out
- Failed: red "✕ Error" — stays until manually dismissed
- Multiple bars stack vertically

### 8. DirectoryPanel paste flow change

Current flow: `p` → `invoke('copy_file')` → blocking → toast

New flow: `p` → check conflicts → show conflict modal → `invoke('copy_file_async')` → returns immediately → progress bar appears → done toast

## Risks / Trade-offs

- **Chunked copy slower than fs::copy**: 64KB chunks add some overhead vs. OS-level copy. Mitigation: same-drive copy within the same filesystem uses `fs::copy` for speed (bytes reported as one chunk at the end).
- **Memory for directory scan**: Scanning a directory with 100K files builds a vec of entries in memory. Mitigation: stream entries, don't collect all. Cap scan depth at 20 levels.
- **Event flooding**: Emitting an event every 64KB for a 100GB file would send 1.6M events. Mitigation: throttle to one event every 100ms minimum.
- **Race condition on cancel**: The operation might complete between the user clicking cancel and the flag being checked. Mitigation: the final `op-complete` event overwrites `op-cancelled` — frontend handles both.
