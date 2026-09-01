## Why

Some archives cannot be opened or previewed without a password, but the current archive flow has no password entry path. Users hit a dead end when they open encrypted `zip` or `7z` archives, especially when extracting with `e` / `E` or previewing files inside an archive.

## What Changes

- Add password prompting for encrypted `zip` and `7z` archives during archive open, extraction, and file preview.
- Show the password input automatically when the user explicitly enters an encrypted archive or starts an encrypted extraction.
- Show a preview-panel hint for selected encrypted archives instead of prompting immediately.
- Keep the password dialog open after an incorrect password and show a clear error message.
- Cache the entered password in memory for the lifetime of the application session.
- Leave `tar` and `tar.gz` behavior unchanged; they do not support password-based encryption in this change.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `archive-browsing`: encrypted `zip` and `7z` archives require password-aware open/preview flows, including retry behavior after password failure.
- `archive-operations`: extraction of encrypted `zip` and `7z` archives requires password-aware execution for `e` / `E` paths.

## Impact

Frontend archive entry points, archive preview loading, extraction commands, and the Rust archive backend command signatures will change to carry optional passwords and remember them in memory during the session.
