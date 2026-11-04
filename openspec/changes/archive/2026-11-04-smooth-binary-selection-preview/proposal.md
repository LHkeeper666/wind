## Why

Selecting a 917 KiB `.torrent` file in the current panel causes severe UI stutter. The extension is treated as text, so preview loading decodes the full file before detecting its NUL bytes and reading it again as binary; hex preview then creates about 4,096 DOM rows synchronously. Binary selection preview should remain responsive, including for other large binary files.

## What Changes

- Classify `.torrent` as binary in the shared file type detector so selection preview takes the binary read path directly.
- Render hex previews incrementally or with viewport virtualization, keeping DOM work bounded while preserving scrolling and readable offsets.
- Keep file size and truncation notices accurate, and ensure stale selection work cannot replace the latest preview.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `file-type-detection`: Recognize `.torrent` as binary consistently.
- `file-preview-loading`: Route `.torrent` through the binary-only read path.
- `selection-preview-overlay`: Keep binary selection preview responsive while scrolling and switching selections.

## Impact

Frontend file classification, file loading, hex rendering, and selection overlay behavior. No backend command or dependency changes are expected.
