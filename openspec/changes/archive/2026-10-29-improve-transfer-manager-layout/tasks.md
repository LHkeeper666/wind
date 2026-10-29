## 1. Responsive transfer rows

- [x] 1.1 Replace fixed source, destination, and progress widths with flexible regions that use available panel width.
- [x] 1.2 Reflow transfer details at narrow widths while keeping progress and cancel/dismiss actions visible.
- [x] 1.3 Preserve readable details for queued, completed, failed, cancelled, and indeterminate extraction entries; expose full truncated paths on hover.
- [x] 1.4 Align progress, transfer size, speed, and time across transfer states using shared columns.

## 2. Shared zoom and readability

- [x] 2.1 Raise default Transfer Manager primary and secondary text sizes and adjust row and control dimensions accordingly.
- [x] 2.2 Connect Transfer Manager text and supporting dimensions to the existing `--zoom-level` value without changing panel height or workspace layout.
- [x] 2.3 Match the Transfer Manager header and single-line row density to the current directory panel while retaining readable text and narrow-width reflow.

## 3. Verification

- [x] 3.1 Run `npx svelte-check` and resolve any diagnostics introduced by the change.
- [x] 3.2 Verify wide and narrow panel layouts at 50%, 100%, and 200% zoom across running, queued, completed, failed, and cancelled entries; confirm panel resizing and actions still work. (User confirmed visual verification.)
