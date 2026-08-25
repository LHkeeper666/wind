## Why

Packaged builds can show a blank CodeMirror viewport when a Markdown file is edited from preview mode with `e`. The file content is already loaded, but the editor can be created and scrolled before its container has a stable visible size, so CodeMirror paints an empty or partially invalid viewport until later input causes a redraw.

## What Changes

- Stabilize the Markdown preview-to-editor transition so CodeMirror is measured after the editor container is visible and sized.
- Re-measure active editor sessions after mode changes and container/layout resizes before restoring scroll to the target Markdown line.
- Keep existing file loading, Vim key handling, and Markdown preview rendering behavior unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vim-editor`: Add a requirement that entering editor mode from Markdown preview must render the loaded document immediately without blank or partially painted viewport regions.

## Impact

- Frontend only: `src/lib/components/PreviewEditor.svelte`.
- No backend command, storage, database, dependency, or keyboard contract changes.
