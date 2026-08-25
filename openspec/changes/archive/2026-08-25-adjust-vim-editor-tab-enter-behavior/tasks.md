## 1. Shared Editor Key Handling

- [x] 1.1 Create shared editor text-key helpers for Markdown list parsing, ordered marker incrementing, tab-stop spacing, indentation, dedentation, and list continuation.
- [x] 1.2 Add focused unit-level coverage or equivalent isolated validation for tab-stop spacing and ordered-list marker increment helpers if the existing test setup supports it.

## 2. Preview Editor Integration

- [x] 2.1 Replace PreviewEditor local `Tab` insertion logic with the shared helper.
- [x] 2.2 Configure PreviewEditor autocomplete so `Tab` accepts active completions and `Enter` no longer accepts completions.
- [x] 2.3 Update PreviewEditor Markdown `Enter` handling so ordered list markers increment and empty list items keep existing exit behavior.
- [x] 2.4 Ensure PreviewEditor editor-focused `Tab` events are consumed once and do not double-indent.

## 3. Fullscreen Editor Integration

- [x] 3.1 Replace FullscreenEditor local `Tab` and `Shift+Tab` logic with the shared helper.
- [x] 3.2 Configure FullscreenEditor autocomplete so `Tab` accepts active completions and `Enter` no longer accepts completions.
- [x] 3.3 Add Markdown list continuation behavior to FullscreenEditor so ordered list markers increment consistently with PreviewEditor.

## 4. Verification

- [x] 4.1 Manually verify insert-mode `Tab` accepts completion when autocomplete is active and otherwise indents.
- [x] 4.2 Manually verify ordinary `Tab` advances to the next tab stop on non-list lines.
- [x] 4.3 Manually verify unordered and ordered Markdown list `Tab` indentation keeps markers at line level.
- [x] 4.4 Manually verify ordered Markdown list `Enter` continues with incremented markers and empty list items still exit the list.
- [x] 4.5 Run `npx svelte-check --tsconfig ./tsconfig.json`.
- [x] 4.6 Run `npm run build`.

## 5. Markdown List Tree Movement

- [x] 5.1 Replace selected-list line-prefix indentation with a list-tree movement model that normalizes selections to selected root items.
- [x] 5.2 Preserve descendant relative depth when indenting or outdenting selected Markdown list trees.
- [x] 5.3 Renumber affected ordered-list containers independently after `Tab` and `Shift+Tab`.
- [x] 5.4 Verify mixed-depth selected ordered lists indent without double-moving descendants.
- [x] 5.5 Verify nested ordered list `Shift+Tab` renumbers both source and target containers.
- [x] 5.6 Verify top-level `Shift+Tab` keeps Markdown list markers instead of deleting them.
