## Context

Insert-mode Markdown list indentation is handled by shared helpers in `src/lib/utils/editor-text-keys.ts`, so it can move list trees and renumber ordered-list containers. Normal-mode Vim indentation follows a different path: `PreviewEditor` and `FullscreenEditor` pass overlay key events to `Vim.multiSelectHandleKey()`, and the `@replit/codemirror-vim` indent operator delegates to CodeMirror's `indentMore()` or `indentLess()`.

CodeMirror marks those indentation transactions with `input.indent` or `delete.dedent`, but reacting to every such transaction would affect more than Vim normal/visual `<<`, `>>`, `<`, and `>` usage. The narrower integration point is the existing normal-mode overlay key path.

## Goals / Non-Goals

**Goals:**

- Renumber Markdown ordered-list containers after normal-mode `<<` and `>>`.
- Renumber Markdown ordered-list containers after visual-mode `<` and `>`.
- Keep the Vim plugin's existing indentation mechanics, cursor movement, repeat count handling, and visual behavior intact.
- Share the renumbering helper between `PreviewEditor` and `FullscreenEditor`.

**Non-Goals:**

- Do not replace the Vim plugin's built-in `indent` operator.
- Do not change insert-mode `Tab`, `Shift+Tab`, autocomplete, or `Enter` behavior.
- Do not introduce a new Markdown parser or external dependency.
- Do not change ordinary code/text indentation semantics outside Markdown ordered-list renumbering.

## Decisions

### Use Post-Indent Renumbering

The implementation should detect candidate Vim indentation keys in the normal-mode overlay path, capture the document before calling `Vim.multiSelectHandleKey()`, then inspect the document afterward. If the key completed a Vim indentation operation and the document changed, the editor should run a shared Markdown ordered-list renumber helper.

This keeps `@replit/codemirror-vim` responsible for interpreting `<<`, `>>`, visual `<` / `>`, repeat counts, and cursor motion. The new behavior only repairs ordered-list numbering after the existing edit.

### Keep Detection Narrow

The post-processing should only run for keys that can complete Vim indentation:

- `<` and `>` in normal/operator-pending/visual contexts.
- The second key in `<<` or `>>`.
- Visual `<` or `>`.

The helper should also check whether the document actually changed before dispatching renumbering edits. This avoids extra work for partial operator input, unsupported contexts, or non-editing key presses.

### Export a Renumber Helper

`editor-text-keys.ts` should expose a small helper that renumbers ordered-list containers in an existing CodeMirror document without performing list-tree movement. It should reuse the same container-level numbering logic used by insert-mode list transforms so both editor surfaces stay consistent.

The helper should return `false` when no Markdown ordered-list numbering changes are needed, allowing callers to avoid unnecessary dispatches.

## Risks / Trade-offs

- [Risk] Running renumbering as a second transaction may create a separate undo step. -> Mitigate by using a narrow user event and testing undo behavior; if needed later, replace the Vim indent operator for one-transaction behavior.
- [Risk] Detecting indentation completion from key presses may miss unusual remaps. -> Mitigate by scoping this change to the built-in `<<`, `>>`, visual `<`, and visual `>` paths first.
- [Risk] Renumbering the full document after every qualifying indent may do unnecessary work on large files. -> Mitigate by dispatching only when ordered marker text actually changes; optimize to affected ranges later if needed.
- [Risk] Non-Markdown ordered-looking text could be renumbered. -> Mitigate by reusing the existing Markdown list prefix parser and limiting execution to indentation edits.
