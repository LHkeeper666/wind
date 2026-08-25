## 1. Global shortcut implementation

- [x] 1.1 Update `PanelLayout.svelte` to map `Alt+N` to new tab, `Alt+M` to MRU switching, and `Alt+U` to close tab, including repeated `M` cycling while Alt is held.
- [x] 1.2 Ignore key-repeat events for one-shot Alt tab commands so holding `Alt+N` cannot create multiple tabs accidentally.
- [x] 1.3 Add a narrow terminal xterm-target exception to the editable-input guard for recognized supported Alt tab chords, preserving `preventDefault`, propagation blocking, fullscreen behavior, and focus-loss cleanup.

## 2. Shortcut and terminal documentation

- [x] 2.1 Update `src/lib/keybindings.ts` with the revised `Alt+M`/`Alt+N` mapping and remove terminal Insert/Normal mode entries.
- [x] 2.2 Update `README.md` and `README_zh.md` with the revised mapping, terminal interception behavior, and single terminal focus-context wording; remove stale `t`-prefix and terminal mode descriptions.

## 3. Verification and specification alignment

- [x] 3.1 Run `npx svelte-check`, `openspec validate "remap-mru-and-intercept-terminal-alt-shortcuts" --strict`, and `git diff --check`; resolve only issues caused by this change.
- [x] 3.2 Manually verify all supported Alt tab actions from directory, preview, editor Normal, embedded terminal, and fullscreen terminal focus, including MRU cycling and Alt-release commit.
- [x] 3.3 Manually verify supported Alt chords do not reach the shell, while generic editable inputs, unsupported Alt, bare Alt, and Ctrl+Alt/AltGr remain unaffected.
