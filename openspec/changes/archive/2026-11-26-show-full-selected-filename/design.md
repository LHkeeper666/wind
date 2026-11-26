## Context

`FileListPanel` and `ProjectTreePanel` render compact rows with ellipsis and already receive `selectedIndex`. The selected row can use that existing state to reveal its full name without adding a separate panel area.

## Goals / Non-Goals

**Goals:**
- Reveal the complete name within the currently selected row while navigating with the keyboard or mouse.
- Reveal complete names on hover and keep unselected rows compact.

**Non-Goals:**
- Automatically resize columns, add a selected-name footer, or change the file-info dialog.
- Add a new keyboard shortcut or store a display preference.

## Decisions

1. Apply `white-space: normal` and `overflow-wrap: anywhere` only to `.file-item.selected .file-name` in both list components. Unselected names retain ellipsis. The selected row grows to fit its text, and the existing scroll-to-selection behavior keeps it reachable. A separate full-name footer was considered but rejected because it would duplicate the selected name and consume permanent panel space.
2. Keep the name flex child shrinkable with `min-width: 0`, especially in project tree rows with indent and toggle controls. This allows long uninterrupted names to wrap inside narrow columns without changing column ratios.
3. Add a native `title` containing the full filename to list and tree name elements. The browser tooltip works without new state or timers. A custom tooltip was considered, but adds positioning and focus complexity for little benefit.

## Risks / Trade-offs

- [Selected row height changes during navigation] → Expand only the selected row and verify the existing scroll-to-selection behavior.
- [Deep tree indentation leaves little name width] → Allow breaks within long words while preserving the current column width.

## Migration Plan

Frontend-only change; no migration. Revert the component changes to restore the prior display.

## Open Questions

None required for implementation.
