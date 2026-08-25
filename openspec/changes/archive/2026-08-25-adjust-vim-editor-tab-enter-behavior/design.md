## Context

The app has two CodeMirror-based Vim editors: `PreviewEditor` for the preview panel and `FullscreenEditor` for the fullscreen overlay. Both currently configure `autocompletion()`, include CodeMirror's `completionKeymap`, and define custom `Tab` indentation behavior locally. `PreviewEditor` also has Markdown list continuation logic on `Enter`; `FullscreenEditor` does not currently share that behavior.

CodeMirror's autocomplete default keymap accepts a selected completion with `Enter`. The requested behavior changes that contract: `Enter` should remain an editing key for line breaks, while `Tab` should accept an active completion before falling back to indentation.

## Goals / Non-Goals

**Goals:**

- Make insert-mode `Tab` accept an active autocomplete suggestion.
- Ensure insert-mode `Enter` always inserts a newline or performs Markdown list continuation, even while autocomplete is visible.
- Make ordinary `Tab` indentation advance to the next tab stop instead of always inserting four spaces.
- Preserve Markdown list indentation as a line-level operation, including the marker.
- Increment ordered Markdown list markers when continuing a list with `Enter`.
- Keep `PreviewEditor` and `FullscreenEditor` behavior aligned.

**Non-Goals:**

- No changes to global command-palette path completion.
- No changes to terminal `Tab` behavior.
- No changes to Vim normal-mode commands, ex command handling, or clipboard/register behavior.
- No new external dependencies.

## Decisions

### Use Custom Completion Key Handling

Configure CodeMirror autocomplete with `defaultKeymap: false`, then install an explicit completion keymap that keeps navigation and close behavior but omits `Enter`. Add a higher-priority `Tab` binding that calls `acceptCompletion(view)` first. If no completion can be accepted, the same binding performs editor indentation.

This avoids fighting CodeMirror's default `Enter` binding and keeps `Enter` available for newline behavior.

### Centralize Editor Text-Key Helpers

Move shared text-key behavior into a small utility module used by both editors. The utility should cover:

- Markdown list marker parsing.
- Ordered-list marker incrementing.
- Tab-stop width calculation.
- Tab indentation and Shift+Tab dedentation.
- Optional Markdown list continuation on `Enter`.

This reduces drift between `PreviewEditor` and `FullscreenEditor`, which already duplicate related logic.

### Define Tab Priority

`Tab` handling in insert mode follows this order:

1. If an autocomplete option is active and accepted by `acceptCompletion(view)`, consume the key.
2. Else, if the cursor or selected lines are Markdown list items, move the selected list tree by one list level.
3. Else, insert enough spaces to advance to the next tab stop for a single cursor, or indent each selected line to its next tab stop.

The default tab size remains four columns.

### Treat Markdown List Indentation as Tree Movement

Markdown list indentation should be modeled as moving list items, not as independently rewriting each selected line prefix. A selected item may own nested child items, and a selection may include both a parent and descendants. The implementation should first identify selected root list items, then move each selected root together with its descendant lines.

For `Tab`, each selected root item moves one list level deeper while preserving the relative indentation of its descendants. For `Shift+Tab`, each selected root item moves one list level shallower while preserving the relative indentation of its descendants. Top-level items should remain list items when outdent is requested; `Shift+Tab` should not delete the marker as part of list-tree movement.

This avoids double-moving descendants in mixed-depth selections:

```text
1. parent
    1. child
    2. child
2. sibling
```

If `parent` and its children are all inside the selection, `parent` is the selected root. The child items move because they belong to `parent`, not because each child line receives another independent indentation change.

### Renumber Ordered Lists by Container

Ordered-list numbering should be computed per ordered-list container after list-tree movement. A single global counter for selected ordered lines is incorrect because different nesting levels and different parent items own independent ordered-list containers.

After `Tab` or `Shift+Tab`, the implementation should identify every ordered-list container whose membership may have changed:

- The source container that lost moved items.
- The target container that received moved items.
- Parent containers whose later sibling numbers may shift after outdent.
- Nested ordered containers carried inside moved trees, if their visible sibling order is affected.

Each affected container should be renumbered independently according to sibling order. A moved ordered item starts at `1.` only when it becomes the first item in a new child ordered-list container; otherwise it receives the next number implied by its sibling position.

### Suggested Internal Model

The first implementation can stay dependency-free by deriving a lightweight line model from the document:

- `lineNumber`, `from`, `to`
- leading indentation in columns
- marker type, marker text, and ordered number
- content start offset
- parent item and descendant range inferred from indentation
- whether the item is selected directly or included as a descendant of a selected root

The transform should then:

1. Parse nearby Markdown list items into this model.
2. Normalize the selection to selected root items.
3. Apply one indent delta to each selected root tree.
4. Rebuild changed prefixes.
5. Renumber affected ordered-list containers independently.

If edge cases around blockquotes, fenced code blocks, lazy continuation lines, or mixed Markdown constructs become significant, the design can move from the lightweight line model to CodeMirror's Markdown syntax tree. CodeMirror's own Markdown list continuation logic is a useful reference because it uses `OrderedList` and `ListItem` structure and renumbers sibling items within a list container.

### Keep Enter as Newline/List Continuation

`Enter` should never accept autocomplete. For Markdown list items, it should continue the list with the appropriate marker:

- Unordered lists reuse the same marker.
- Ordered lists increment the numeric marker.
- Empty list items continue to exit or reduce the list according to existing behavior.

For non-list lines, CodeMirror's normal newline behavior should run.

## Risks / Trade-offs

- [Risk] `Tab` may be handled twice by global capture and CodeMirror keymaps. → Mitigate by clearly routing editor-focused `Tab` to one path and consuming the event once.
- [Risk] Disabling autocomplete's default keymap can accidentally remove useful keys. → Mitigate by explicitly re-adding Escape, arrows, page navigation, and Ctrl-Space.
- [Risk] Fullscreen editor behavior may change more than expected because it currently lacks PreviewEditor's Markdown Enter handling. → Mitigate with specs and manual validation for both editor surfaces.
- [Risk] Tab-stop calculations can be surprising with tabs or wide Unicode characters. → Mitigate by basing the first implementation on plain column count and the existing four-column tab size used by the app.
- [Risk] Regex-only line prefix replacement can misnumber ordered lists in mixed-depth selections or outdent operations. → Mitigate by modeling selected list roots, moving descendant ranges together, and renumbering ordered-list containers independently.
- [Risk] Markdown constructs such as blockquotes, fenced code blocks, and lazy continuation lines may be hard to represent in a lightweight line model. → Mitigate by scoping the first pass to standard indented Markdown lists and considering CodeMirror's Markdown syntax tree if those constructs become required.
