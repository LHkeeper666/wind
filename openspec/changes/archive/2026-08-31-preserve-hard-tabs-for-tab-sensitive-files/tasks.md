## 1. Indentation Policy

- [x] 1.1 Add a shared helper that classifies file paths into ordinary, Makefile-style, and TSV-style indentation policies.
- [x] 1.2 Cover Makefile names `Makefile`, `makefile`, `GNUmakefile`, `BSDmakefile`, plus `.mk` and `.mak` extensions.
- [x] 1.3 Cover TSV-style `.tsv` and `.tab` extensions as literal-tab insertion targets.

## 2. Editor Text-Key Behavior

- [x] 2.1 Extend `handleInsertModeTab` to accept indentation policy while preserving autocomplete confirmation first.
- [x] 2.2 Insert literal `\t` for single-cursor Tab in Makefile-style and TSV-style files.
- [x] 2.3 Insert one leading literal `\t` per selected line for Makefile-style multi-line indentation.
- [x] 2.4 Extend `handleInsertModeShiftTab` so Makefile-style lines remove one leading literal `\t` before falling back to existing space dedent behavior.
- [x] 2.5 Extend Enter handling so Makefile-style automatic indentation preserves leading literal tabs from the current line.
- [x] 2.6 Keep Markdown list indentation, tree movement, and ordered-list renumbering behavior unchanged for Markdown files.

## 3. Editor Integration

- [x] 3.1 Pass the active file's indentation policy from `PreviewEditor` into shared text-key handlers.
- [x] 3.2 Pass the active file's indentation policy from `FullscreenEditor` into shared text-key handlers.
- [x] 3.3 Configure CodeMirror `indentUnit` from the same policy in both editor surfaces.
- [x] 3.4 Ensure Tab-sensitive behavior remains identical between preview and fullscreen editing.

## 4. Verification

- [x] 4.1 Add focused tests for ordinary file Tab spacing behavior.
- [x] 4.2 Add focused tests for Makefile single-cursor Tab, selected-line Tab, Shift+Tab, and Enter continuation preserving literal `\t`.
- [x] 4.3 Add focused tests for TSV-style Tab inserting a literal field separator.
- [x] 4.4 Run `npx svelte-check` and the relevant frontend test command if one exists.
