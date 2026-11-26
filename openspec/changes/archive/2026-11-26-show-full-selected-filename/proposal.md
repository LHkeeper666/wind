## Why

Long filenames are truncated in the narrow directory columns, so users must resize the layout or open file information to identify the selected entry. The selected row should reveal its complete name in place.

## What Changes

- Wrap the selected entry's name within its row in ordinary directory lists and project tree mode.
- Show the complete name on mouse hover over a truncated list or tree entry.
- Keep unselected rows compact and single-line, without changing the column layout.

## Capabilities

### New Capabilities

- `full-filename-visibility`: Complete filename display in the selected row and on mouse hover in list and project tree views.

### Modified Capabilities

None.

## Impact

- Frontend directory panel, file list, and project tree components and their styling.
- No Rust commands, data model changes, or new dependencies.
