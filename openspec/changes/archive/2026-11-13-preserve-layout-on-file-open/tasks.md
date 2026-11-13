## 1. File activation

- [x] 1.1 Remove automatic `expandPreview()` calls from current-slot and new-slot file activation while preserving slot and selected-file updates.
- [x] 1.2 Make new-file and already-open-file activation focus preview only when its column ratio is positive; otherwise retain the initiating directory panel and DOM focus.
- [x] 1.3 Verify the save/discard confirmation path uses the same layout and focus behavior after replacing an edited file.

## 2. Verification

- [x] 2.1 Cover `l` and `L` in three-column, two-column, and hidden-preview layouts, including reuse of an existing slot and later revealing the preview.
- [x] 2.2 Verify current ratios and the three-column cycle snapshot remain unchanged after each file-open path, and directory navigation plus explicit layout controls still work.
- [x] 2.3 Run `npx svelte-check` and any focused tests added for activation and focus behavior.
