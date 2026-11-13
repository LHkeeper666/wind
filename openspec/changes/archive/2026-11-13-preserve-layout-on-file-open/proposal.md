## Why

Opening a file with `l` or `L` currently expands the preview and replaces the user's column ratios with `[0,1,4]`. File opening should preserve the workspace layout, including layouts that intentionally hide the preview column.

## What Changes

- Opening a file in the current or a new preview slot preserves the active column ratios and the saved three-column cycle ratios.
- When the preview column is visible, the opened file receives preview focus as before. When it has zero width, the file opens in its slot while focus stays on a visible directory column.
- Reusing an existing slot and completing the unsaved-changes confirmation follow the same rule.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace-preview-slots`: Clarify `l` and `L` opening behavior when the preview column is hidden.
- `ratio-presets`: Require file opening to preserve user-selected ratios and layout-cycle state.

## Impact

Frontend file activation and preview-slot focus logic in `DirectoryPanel.svelte` and `PanelLayout.svelte`; layout state in `layout.ts` is consulted but need not change. No backend, API, or dependency changes.
