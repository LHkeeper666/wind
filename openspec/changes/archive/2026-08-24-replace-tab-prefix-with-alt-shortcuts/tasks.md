## 1. OpenSpec coordination

- [x] 1.1 Reconcile the active `fix-fullscreen-terminal-tab-focus` change so its remaining tab-shortcut requirements and manual checks use Alt bindings, or explicitly supersede it before either change is archived.

## 2. Global tab shortcut handling

- [x] 2.1 In `PanelLayout.svelte`, replace `waitingForTabKey`, its timeout, and `tHeld` with an Alt-held lifecycle for the tab switcher.
- [x] 2.2 Dispatch the canonical `Alt+T/C/R/N/H/L/,/./D/1-9` mapping only from the capture-phase global handler, preserving existing tab actions and preventing default only for supported chords.
- [x] 2.3 Commit an active switcher on `AltLeft`/`AltRight` keyup and clear its state on focus loss; allow repeated `N`, `H`, or `L` while Alt remains held.
- [x] 2.4 Exclude editable inputs, editor insert mode, modal overlays, unsupported Alt shortcuts, and `Ctrl+Alt`/AltGr combinations from tab-command interception; keep terminal behavior global because it has no separate input-mode state.

## 3. Remove prefix-specific paths

- [x] 3.1 Remove the local `t`-prefix state machines and timeout handling from `PreviewEditor.svelte`, allowing its unmodified `t` key to reach CodeMirror Vim.
- [x] 3.2 Remove obsolete tab-command props, helper branches, and status-prefix updates that are no longer reachable after centralizing dispatch.

## 4. Update shortcut references

- [x] 4.1 Update `src/lib/keybindings.ts` so the Tab group is titled and populated with the canonical Alt shortcuts, including detach and rename-hint behavior.
- [x] 4.2 Update `README.md` and `README_zh.md` to match the runtime mapping and remove stale `t n`/`t x`/`t [` documentation.

## 5. Verify behavior

- [x] 5.1 Run `npx svelte-check` and resolve only issues caused by this change.
- [x] 5.2 Manually verify every direct Alt action in directory, preview, editor Normal, terminal, and fullscreen-terminal contexts.
- [x] 5.3 Manually verify MRU and `Alt+H/L` display-order switching when tapping and holding Alt, including repeated cycling and commit on Alt release.
- [x] 5.4 Manually verify unmodified Vim `t`, editor insert input, command/search dialogs, bare Alt, unsupported Alt chords, and AltGr remain unaffected.
