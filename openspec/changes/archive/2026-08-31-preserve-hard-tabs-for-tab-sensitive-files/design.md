## Context

Wind's editor uses CodeMirror 6 with `@replit/codemirror-vim`. Insert-mode `Tab`, `Shift+Tab`, and `Enter` are routed through shared frontend helpers so Markdown list behavior and autocomplete remain consistent between `PreviewEditor` and `FullscreenEditor`.

Today that shared helper always expands Tab to spaces, and both editor surfaces configure `indentUnit.of('    ')`. This is acceptable for most code and Markdown files, but it breaks formats where the tab byte is meaningful. Makefile recipes require a literal tab by default, and TSV-style files use tabs as field separators.

## Goals / Non-Goals

**Goals:**

- Preserve literal `\t` characters for Makefile-style files and TSV-style tab-delimited files.
- Keep existing space-based indentation for ordinary files.
- Keep existing Markdown list indentation and renumbering behavior unchanged.
- Apply the same policy in preview editor and fullscreen editor.
- Keep the change local to editor behavior without touching file persistence or backend commands.

**Non-Goals:**

- Add a full project-wide formatter or whitespace normalization system.
- Implement per-language formatting preferences beyond the tab-sensitive file policy.
- Add `.editorconfig` support in this change.
- Change Makefile behavior when the file explicitly uses `.RECIPEPREFIX`; the editor will still preserve hard tabs by default for Makefile-style files.

## Decisions

1. Introduce a small per-file indentation policy.

   The editor should derive a policy from the current file path:
   - `space-indent` for ordinary files.
   - `hard-tab-indent` for Makefile-style files such as `Makefile`, `makefile`, `GNUmakefile`, `BSDmakefile`, `*.mk`, and `*.mak`.
   - `tab-delimited` for TSV-style files such as `*.tsv` and `*.tab`.

   Alternative considered: make every file insert literal tabs. That would fix Makefiles, but it would regress Markdown lists and common language style for JavaScript, TypeScript, Svelte, Rust, Java, Python, JSON, and YAML.

2. Keep Markdown list behavior ahead of generic indentation.

   Markdown list handling is already specialized: it moves list items as trees and renumbers ordered-list containers. That behavior should continue to run before generic Tab insertion for Markdown files. The hard-tab policy applies only where file type detection chooses it.

   Alternative considered: route all Tab handling through CodeMirror's default indentation commands. That would reduce custom code, but it would lose existing list-tree behavior and autocomplete semantics.

3. Pass indentation policy into shared text-key helpers.

   `handleInsertModeTab`, `handleInsertModeShiftTab`, and Enter handling should receive the active file's policy from each editor surface. This keeps PreviewEditor and FullscreenEditor behavior aligned while avoiding duplicate logic.

   Alternative considered: duplicate Makefile handling in each Svelte component. That would be quicker, but it increases the chance the two editor surfaces drift.

4. Configure CodeMirror indentation extensions from the same policy.

   Editor initialization should use `indentUnit.of('\t')` for hard-tab files and `indentUnit.of('    ')` for ordinary files, with `EditorState.tabSize.of(4)` retained for display and column math. The explicit key handlers remain the source of truth for insert-mode Tab behavior.

## Risks / Trade-offs

- Makefile detection may miss uncommon filenames -> keep the detector small and covered by tests; users can still manually type a literal tab via paste until broader configuration exists.
- `.RECIPEPREFIX` allows non-tab recipe prefixes -> this change intentionally preserves the safe default for Makefile-style files and does not parse file contents for custom prefixes.
- TSV editing treats Tab as data rather than focus traversal -> this is expected only while the editor is active; global focus switching remains outside editor insert mode.
- Mixed tabs and spaces in existing Makefiles may remain mixed -> the change prevents new accidental space-only recipe prefixes but does not rewrite existing file content.

## Migration Plan

No data migration is required. Existing files remain unchanged until edited. Rollback is limited to reverting the frontend editor policy and helper changes.
