## Why

On Windows, renaming a local file or directory only by letter case reports that the destination already exists. The destination check treats the source's case-insensitive alias as a conflicting entry, blocking a valid rename in both single and batch operations.

## What Changes

- Allow local files and directories to be renamed when only their name's letter case changes.
- Keep rejecting a destination that belongs to another directory entry, including in case-sensitive directories.
- Apply the same collision rules to single and batch rename, and verify the requested casing is actually stored.
- Add focused regression coverage for case-only renames and genuine collisions.

## Capabilities

### New Capabilities

- `local-file-rename`: Defines local single and batch rename behavior for case-only changes, no-op names, and destination collisions.

### Modified Capabilities

None.

## Impact

The local rename commands in `src-tauri/src/commands/file_ops.rs` and their tests are affected. The existing frontend invocation contract and FTP/archive rename behavior remain unchanged. A file-identity helper dependency may be needed if the existing Windows APIs are not used directly.
