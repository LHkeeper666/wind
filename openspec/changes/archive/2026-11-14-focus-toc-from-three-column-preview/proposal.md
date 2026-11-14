## Why

In three-column mode, `Ctrl+W l` cannot move focus from preview content to a visible Markdown or PDF outline. The navigation handler gates outline focus on `previewExpanded`, which is false whenever all three columns are visible, even though outline availability does not depend on that layout flag.

## What Changes

- Allow `Ctrl+W l` to focus a visible outline from the preview column in three-column mode.
- Keep the same behavior in two-column and one-column modes, for both Markdown and PDF outlines.
- Preserve the existing navigation sequence from a directory column into preview content, and `Ctrl+W h` from outline back to preview content.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `outline-sidebar`: Define layout-independent keyboard focus entry for a visible Markdown or PDF outline.

## Impact

Frontend window-navigation handling in `src/lib/composables/keyboard-shortcuts.svelte.ts` and panel navigation callbacks in `src/lib/components/PanelLayout.svelte`. No backend, API, or dependency changes.
