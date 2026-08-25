## Why

Markdown ordered-list renumbering currently only runs through the custom insert-mode `Tab` and `Shift+Tab` helpers. Normal-mode Vim indentation such as `<<` uses the Vim plugin's built-in indent operator, so ordered list markers can become stale after outdent.

## What Changes

- Extend Markdown ordered-list renumbering to normal-mode Vim indentation after `<<`.
- Apply the same post-indent renumbering to related Vim indentation paths such as `>>` and visual `<` / `>` when they change Markdown ordered-list structure.
- Keep Vim's existing normal-mode indentation semantics intact; this change only repairs ordered-list numbering after the Vim plugin completes indentation.
- Preserve insert-mode `Tab`, `Shift+Tab`, autocomplete, and `Enter` behavior from the previous change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vim-editor`: Extend Markdown ordered-list renumbering from insert-mode list indentation to normal and visual Vim indentation operations.

## Impact

- Frontend editor behavior in `src/lib/components/PreviewEditor.svelte`.
- Frontend editor behavior in `src/lib/components/FullscreenEditor.svelte`.
- Shared editor Markdown helper behavior in `src/lib/utils/editor-text-keys.ts`.
- No backend, database, file format, or dependency changes expected.
