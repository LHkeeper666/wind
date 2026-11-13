## Context

`DirectoryPanel` routes `l` and `L` on files to `PanelLayout` activation handlers. Those handlers open or select a preview slot, then may call `layout.expandPreview()`, which replaces the workspace ratios with `[0,1,4]`. They also focus preview unconditionally. A zero-width preview can therefore receive focus even when the layout is preserved.

## Goals / Non-Goals

**Goals:**
- Preserve the current ratios and saved layout-cycle snapshot through file activation.
- Focus preview only when its ratio is positive; otherwise retain the initiating visible directory panel's focus.
- Apply the same rule to new slots, existing slots, and activation after unsaved-change confirmation.

**Non-Goals:**
- Change explicit ratio commands, the layout cycle, or the preview toggle.
- Change directory navigation or archive entry behavior.

## Decisions

- Remove automatic preview expansion from the file activation paths in `PanelLayout`. Keep layout changes behind explicit user controls. Altering `expandPreview()` globally would affect unrelated commands.
- Choose the focus target from the current preview ratio at activation time. If preview is hidden, keep the initiating directory panel active; if visible, retain existing preview focus and selection-overlay behavior. This follows the existing zero-width focus rule.
- Route opening an already-open file through the same visibility-aware focus decision. The current early-return paths otherwise bypass the new rule.
- Keep slot creation and replacement independent of column visibility. A hidden preview still receives the file so the user can reveal it later without reopening.

## Risks / Trade-offs

- [Hidden preview gives no immediate visual confirmation of the opened file] → Keep slot state and selected-file state updated; the file appears when the user reveals preview.
- [Async focus restoration may move focus into a hidden preview] → Check visibility before both immediate and deferred focus changes, including existing-slot activation.

## Migration Plan

No data migration. Implement in frontend activation and focus paths, then run Svelte checks and targeted behavioral verification. Reverting these frontend changes restores prior behavior.

## Open Questions

None.
