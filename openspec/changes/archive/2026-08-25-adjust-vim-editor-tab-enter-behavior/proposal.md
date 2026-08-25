## Why

The Vim editor currently lets CodeMirror autocomplete accept suggestions with `Enter`, which conflicts with the expected editing flow where `Enter` creates a new line. `Tab` indentation also uses a fixed four-space insertion for ordinary text, and ordered Markdown lists keep the same number when continuing a list.

## What Changes

- Change insert-mode autocomplete confirmation from `Enter` to `Tab`.
- Make `Enter` create a new line even when the autocomplete tooltip is open.
- Make ordinary `Tab` indentation advance to the next tab stop instead of always inserting four spaces.
- Preserve existing Markdown list indentation behavior where `Tab` indents the whole list item line, including the marker.
- Increment ordered Markdown list markers when continuing a list with `Enter`.
- Keep preview editor and fullscreen editor Vim behavior consistent.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vim-editor`: Adjust insert-mode `Tab`/`Enter` behavior for autocomplete, indentation, and Markdown ordered list continuation.

## Impact

- Frontend editor behavior in `src/lib/components/PreviewEditor.svelte`.
- Fullscreen editor behavior in `src/lib/components/FullscreenEditor.svelte`.
- Likely shared editor keybinding/Markdown helpers under `src/lib/utils/`.
- No backend, database, file format, or dependency changes expected.
