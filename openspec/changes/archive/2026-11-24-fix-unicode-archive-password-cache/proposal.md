## Why

Previewing an archive whose path contains non-ASCII characters can panic before the archive is opened. RAR suffix detection in the shared password-cache path slices a UTF-8 string at a byte offset that may fall inside a character; the observed 7z preview failure is one example.

## What Changes

- Make archive password-cache key generation safe for Unicode paths across supported archive formats.
- Use consistent, suffix-anchored recognition of RAR volumes (`.rar`, `.rXX`, `.partN.rar`) for format detection and cache normalization.
- Preserve password sharing between volumes of one RAR archive without mistaking unrelated filenames for volumes.
- Add focused regression coverage for Unicode paths, suffix edge cases, and archive preview and extraction entry points.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `archive-browsing`: Archive listing and preview must handle Unicode archive paths without a backend panic, including before password resolution.
- `archive-operations`: Password-assisted archive operations must handle Unicode paths and share cached passwords across valid RAR volumes.

## Impact

Rust archive format and password-cache logic in `src-tauri/src/archive/mod.rs` and `password.rs`, plus related Rust tests. No API shape or dependency change is intended.
