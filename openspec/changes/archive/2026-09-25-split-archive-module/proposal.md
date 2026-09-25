## Why

`src-tauri/src/archive/mod.rs` is 1663 lines — the largest Rust module in the project. It mixes 7 unrelated concerns (password management, encoding detection, ZIP/TAR/TAR.GZ/7z operations, shared helpers) in a single file. This makes it hard to navigate, test individual formats, and add new archive formats without conflicts.

## What Changes

- Extract password cache/management into `archive/password.rs`
- Extract encoding detection into `archive/encoding.rs`
- Extract shared path utilities into `archive/shared.rs`
- Extract ZIP operations (including central directory parsing) into `archive/zip.rs`
- Extract TAR + TAR.GZ operations into `archive/tar.rs` (merged, ~95% code overlap)
- Extract 7z operations into `archive/seven_z.rs`
- Slim down `mod.rs` to ~120 lines: `ArchiveFormat` enum + public API dispatch only

## Capabilities

### New Capabilities

_None — this is a pure structural refactoring. No new behavior._

### Modified Capabilities

_None — all existing specs (`archive-browsing`, `archive-encoding-detection`, `archive-operations`) remain unchanged. Public API signatures are preserved._

## Impact

- **Code**: `src-tauri/src/archive/` — from 1 file to 7 files
- **APIs**: No changes. `mod.rs` retains all `pub` functions with identical signatures
- **External callers**: Zero changes required (`crate::archive::*` paths unchanged)
- **Dependencies**: No new crate dependencies. Internal module dependencies are all one-way (no cycles)
