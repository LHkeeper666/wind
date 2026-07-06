## Why

Current file operations (copy, move, delete) are synchronous blocking calls. Copying a 2GB file or a large directory like `node_modules` freezes the entire UI for seconds. The user gets no progress feedback — the app appears hung. Moving to async I/O with progress reporting brings the file manager experience in line with yazi and Windows Explorer.

## What Changes

- **Copy file**: New async command with chunked I/O, byte-level progress events, and cancel support
- **Copy directory**: Recursive async copy with per-file progress and total byte tracking
- **Move file**: Async, same-drive `fs::rename` (instant), cross-drive fallback to async copy+delete
- **Delete file/directory**: Async, with file-count progress for large directories
- **Pre-check**: Before executing, run a conflict check and let the user resolve conflicts one by one
- **Phase 1 scan**: Async directory tree scan to compute total bytes/files before execution starts
- **Progress UI**: Bottom status bar component showing per-operation progress bars with cancel buttons, supports multiple concurrent operations, completed operations auto-dismiss after 3 seconds
- **Backend**: New `file_ops` module with `FileOp` state management, `cancel_flag` via `AtomicBool`, progress emission via Tauri events

## Capabilities

### New Capabilities

- `async-file-operations`: Core async I/O — chunked copy, cancel support, progress events, pre-check conflict detection, directory scan, multi-operation parallelism

### Modified Capabilities

None — all existing `copy_file`, `move_file`, `delete_file` commands remain unchanged; new async variants are added alongside them.

## Impact

- **Rust**: New `src-tauri/src/file_ops.rs` module, ~6 new async commands in `lib.rs`, no new dependencies required
- **Frontend**: New `FileOpProgress.svelte` status bar component, modifications to `DirectoryPanel.svelte` (paste → pre-check → async), `PanelLayout.svelte` (event listeners)
- **Backward compatible**: Existing sync commands preserved; frontend switches to async variants
