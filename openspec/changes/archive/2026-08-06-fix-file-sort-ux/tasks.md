# Tasks: Fix File Sorting UX

## Implementation

- [x] **1. Add `sortPrefixPending` state and timeout** (lines 77-78)
- [x] **2. Refactor sort prefix detection in `handleKeydown`** (lines 745-772)
- [x] **3. Add `{ numeric: true }` to name sort `localeCompare`** (line 112)
- [x] **4. Verify with type checking** — `npx svelte-check`: 0 errors, 49 pre-existing warnings

## Verification

- [x] **5. Manual smoke test** — all sort modes verified, toast notifications confirmed
