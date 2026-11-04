## 1. Binary classification and loading

- [x] 1.1 Add `torrent` to the shared binary extension set and verify case-insensitive classification.
- [x] 1.2 Verify `.torrent` preview invokes only the binary read command, with partial reads above 1 MiB.

## 2. Bounded hex rendering

- [x] 2.1 Virtualize hex rows in `TextPreviewer`, preserving offsets, ASCII display, scrolling, and the 64 KiB display cap.
- [x] 2.2 Pass source file size into the selection overlay's hex preview and show an accurate size/truncation notice.
- [x] 2.3 Clean up scroll listeners and stale render work when files or selections change.

## 3. Verification

- [x] 3.1 Add focused tests or a reproducible browser check for classification, one-read loading, bounded mounted rows, correct scroll offsets, and rapid selection changes.
- [x] 3.2 Run `npx svelte-check` and the relevant test or validation command; manually inspect the reported `.torrent` selection if the file is available.
