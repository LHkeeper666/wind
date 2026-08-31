## 1. Locate State Ownership

- [x] 1.1 Identify the project mode file tree component, composable, or store that owns expanded directory paths, selected path, focused path, and preview target.
- [x] 1.2 Trace the current mouse collapse flow and any keyboard or programmatic collapse entry points.
- [x] 1.3 Identify existing selection and preview update methods that should be reused by the collapse correction logic.

## 2. Shared Collapse Correction

- [x] 2.1 Add a shared helper or action, such as `ensureSelectionVisibleAfterCollapse(collapsedDirPath)`, in the tree state layer.
- [x] 2.2 Implement structured descendant-path detection so similarly prefixed sibling paths are not treated as children.
- [x] 2.3 When the current selection is inside the collapsed directory, update selection, tree focus, and preview target to the collapsed directory through the existing selection flow.
- [x] 2.4 When the current selection is outside the collapsed directory, preserve selection, focus, and preview target.
- [x] 2.5 When the selected item is the directory being collapsed, keep that directory selected and focused while refreshing or preserving the directory preview.

## 3. Collapse Entry Points

- [x] 3.1 Update mouse-triggered directory collapse to call the shared correction method.
- [x] 3.2 Update keyboard-triggered directory collapse, if present, to call the shared correction method.
- [x] 3.3 Update programmatic or batch directory collapse, if present, to call the shared correction method or document why it is unaffected.
- [x] 3.4 Ensure DOM focus is restored to the corrected visible directory node after render when the implementation has no separate focus state.

## 4. Validation

- [x] 4.1 Add or update unit tests for collapsing the parent directory of the selected file.
- [x] 4.2 Add or update unit tests for collapsing an ancestor directory of the selected file.
- [x] 4.3 Add or update unit tests for collapsing an unrelated directory without changing selection, focus, or preview.
- [x] 4.4 Add or update unit tests for collapsing the selected directory itself.
- [x] 4.5 Run the relevant frontend typecheck and test commands for the affected package.
- [ ] 4.6 Manually verify in project mode that mouse collapse moves focus to the collapsed parent directory and the preview updates immediately.
