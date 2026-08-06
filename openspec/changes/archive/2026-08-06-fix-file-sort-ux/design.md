# Design: File Sorting UX Fix

## Architecture Overview

Both fixes are localized to `src/lib/components/DirectoryPanel.svelte`. No backend changes required.

## Fix 1: Robust Sort Prefix Detection

### Current approach (fragile)

```
       ┌─────────┐     lastKey === 'KeyS'?     ┌──────────────┐
 s ───▶│ lastKey │────── && < 500ms? ──────────▶│ setSort(...) │
       │ = KeyS  │                              │ lastKey=''   │
       └─────────┘                              └──────────────┘
```

Problem: `lastKey` is a global "last key pressed" tracker used for multiple purposes (double-g, sort prefix). Any stray key press can corrupt the prefix state.

### New approach (state-machine)

```
                    ┌──────────────────────────────────────┐
                    │                                      │
    s ─────────────▶│  sortPrefixPending = true            │
                    │  start 1000ms timeout                │
                    │                                      │
                    │       ┌────────────────────┐         │
    n/s/e/m/c/t ───▶│──────▶│ setSort/toggle     │────────▶│ clear pending
                    │       └────────────────────┘         │
                    │                                      │
    (timeout) ─────▶│  sortPrefixPending = false           │
                    │                                      │
    (other key) ───▶│  handled normally,                   │
                    │  sortPrefixPending = false           │
                    └──────────────────────────────────────┘
```

Implementation follows the `waitingForTabKey` pattern in `PanelLayout.svelte`:

```typescript
// State
let sortPrefixPending = $state(false);
let sortPrefixTimeout: ReturnType<typeof setTimeout> | null = null;

// In handleKeydown, when 's' is pressed:
if (event.code === 'KeyS' && !event.ctrlKey && !event.altKey) {
  if (!sortPrefixPending) {
    // First 's': start prefix
    sortPrefixPending = true;
    if (sortPrefixTimeout) clearTimeout(sortPrefixTimeout);
    sortPrefixTimeout = setTimeout(() => { sortPrefixPending = false; }, 1000);
  }
  // If already pending, let it fall through (second 's' → sort by size)
}

// In handleKeydown, when ANY key is pressed and sortPrefixPending is true:
if (sortPrefixPending && codeMap[event.code]) {
  sortPrefixPending = false;
  if (sortPrefixTimeout) { clearTimeout(sortPrefixTimeout); sortPrefixTimeout = null; }
  // ... process sort command
}
```

The existing `codeMap` and `setSort` call logic remains unchanged.

## Fix 2: Natural Sort for Names

One-line change. `String.prototype.localeCompare` supports a `numeric` option:

```typescript
// Before
cmp = a.name.localeCompare(b.name);

// After
cmp = a.name.localeCompare(b.name, undefined, { numeric: true });
```

> **Note on the `undefined` parameter**: `localeCompare` signature is `localeCompare(compareString, locales?, options?)`. Passing `undefined` for locales uses the browser's default locale while still applying the `numeric` option.

### Behavior comparison

| Input | Before (lexicographic) | After (numeric) |
|-------|----------------------|-----------------|
| `1, 2, 10, 11, 21` | `1, 10, 11, 2, 21` | `1, 2, 10, 11, 21` |
| `file1.txt, file2.txt, file10.txt` | `file1, file10, file2` | `file1, file2, file10` |
| `a1b, a2b, a10b` | `a10b, a1b, a2b` | `a1b, a2b, a10b` |
| `abc, def` (no numbers) | `abc, def` | `abc, def` (same) |

`{ numeric: true }` only affects embedded numeric sequences, leaving purely alphabetic strings sorted as before.
