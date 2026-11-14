## Context

The global `Ctrl+W` handler and directory-panel navigation callbacks each implement rightward panel switching. Both enter the outline from the preview column only when `previewExpanded` is true. Layout state defines that flag as fewer than three visible columns, while `PreviewEditor.isTocVisible()` already checks whether a Markdown or PDF outline is available in normal preview mode.

## Goals / Non-Goals

**Goals:**
- Make a visible outline reachable from preview content with `Ctrl+W l` in any layout with a visible preview column.
- Keep directory-to-preview navigation and outline-to-content navigation consistent.

**Non-Goals:**
- Change outline visibility, column ratios, or editing-mode key handling.
- Auto-open a closed outline or add a new shortcut.

## Decisions

- Remove the `previewExpanded` gate in both rightward navigation implementations. Retain the existing active-column, `isTocVisible()`, and `isTocFocused()` checks. Outline visibility is the relevant condition; layout width is not.
- Keep the current two-step navigation from a directory column: first `Ctrl+W l` enters preview, then another enters its outline. This preserves panel traversal semantics.
- Use the existing `PreviewEditor.focusToc()` path for Markdown and PDF. It already delegates to their respective focus implementations and handles delayed outline mounting.

## Risks / Trade-offs

- A caller could attempt focus while an outline is closing. The existing visibility check and focus retry guards prevent focus from landing in an unavailable outline.
- The duplicated navigation branches can drift again. Update both in the same change and verify both keyboard and directory-panel navigation paths.
