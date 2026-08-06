# Proposal: Fix File Sorting UX

## Why

Directory panel file sorting has two UX issues:

1. **`sC` (sort by created time, reversed) doesn't work.** The sort prefix detection reuses `lastKey === 'KeyS'` which is fragile — a stray double-`s` press silently consumes the prefix as `ss` (sort by size) and the subsequent `Shift+C` does nothing with no feedback.

2. **`sn` sorts lexicographically, not naturally.** Files `1, 10, 11, 2, 21` sort as `1, 10, 11, 2, 21` instead of the expected `1, 2, 10, 11, 21`.

## What Changes

Both fixes in `src/lib/components/DirectoryPanel.svelte`:

- Replace `lastKey === 'KeyS'` + 500ms timeout with a dedicated `sortPrefixPending` state variable (same pattern as `waitingForTabKey` in PanelLayout)
- Add `{ numeric: true }` to `localeCompare` for natural numeric sort on name sorting
