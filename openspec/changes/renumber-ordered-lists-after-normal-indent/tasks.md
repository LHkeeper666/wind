## 1. Shared Renumber Helper

- [ ] 1.1 Export a helper from `editor-text-keys.ts` that renumbers Markdown ordered-list containers without applying list-tree movement.
- [ ] 1.2 Ensure the helper returns whether it dispatched numbering changes so callers can skip no-op edits.

## 2. Normal/Visual Vim Integration

- [ ] 2.1 Add PreviewEditor normal-mode overlay handling that detects completed Vim indentation operations and runs ordered-list renumbering after document changes.
- [ ] 2.2 Add FullscreenEditor normal-mode overlay handling with the same behavior.
- [ ] 2.3 Keep non-list indentation and non-indentation Vim keys unchanged.

## 3. Verification

- [ ] 3.1 Verify normal-mode `<<` on a nested ordered Markdown item renumbers both source and target containers.
- [ ] 3.2 Verify normal-mode `>>` on an ordered Markdown item renumbers affected containers.
- [ ] 3.3 Verify visual `<` and `>` on ordered Markdown list selections renumber affected containers.
- [ ] 3.4 Run `npm run check`.
- [ ] 3.5 Run `npm run build`.
