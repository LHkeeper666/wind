## Why

The file information panel displays creation, modification, and access times as UTC clock values without a timezone label. Users comparing them with Windows file properties see a consistent offset (eight hours in China), making the metadata appear incorrect.

## What Changes

- Display the three file timestamps in the user's local timezone when opening file information with `i`.
- Transfer timestamps as unambiguous instants from the Rust command to the frontend, leaving locale formatting to the UI.
- Handle unavailable timestamps without showing a fabricated date.

## Capabilities

### New Capabilities

- `file-info-local-time`: File information timestamps reflect the user's local timezone and remain accurate across date boundaries.

### Modified Capabilities

None.

## Impact

- Affects `get_file_info` in `src-tauri/src/commands/file_info.rs`, its response type, and `src/lib/components/FileInfoPanel.svelte`.
- Changes the internal Tauri command response shape for the three timestamp fields; callers and tests using those fields must be updated together.
- No new dependency is required.
