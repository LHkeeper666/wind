## Why

The current Alt tab mapping reserves `Alt+N` for MRU switching and `Alt+T` for creating a tab, which makes the most common tab-creation action less discoverable and leaves the MRU action inconsistent with the preferred mnemonic. In addition, xterm's editable textarea causes the global shortcut guard to skip Alt tab commands when the terminal has focus, so tab management behaves differently between panels.

This change remaps the shortcuts and makes supported Alt tab commands consistently application-owned whenever focus is in the terminal, without maintaining a separate terminal insert/normal distinction.

## What Changes

- **BREAKING** Change the canonical mapping to `Alt+N` for new tab and `Alt+M` for MRU tab switching.
- **BREAKING** Change the close-tab shortcut from `Alt+C` to `Alt+U`; `Alt+C` is no longer a tab operation.
- Keep `Alt+H`/`Alt+L`, `Alt+R`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9` unchanged.
- Intercept every supported Alt tab command from the terminal input area, including the fullscreen terminal, before xterm forwards the chord to the shell.
- Treat the terminal as one focus context; remove or correct documentation that distinguishes terminal Insert and Normal modes for this shortcut contract.
- Update the help registry, English and Chinese README files, and the affected OpenSpec requirements.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-panel-navigation`: Remap `Alt+M`/`Alt+N` and make all supported Alt tab commands work from terminal and fullscreen-terminal focus without terminal mode distinctions.
- `help-overlay`: Display the revised Alt tab mapping and terminal shortcut semantics.

## Impact

- `src/lib/components/PanelLayout.svelte`: global Alt shortcut dispatch, MRU switcher key handling, and terminal editable-target gating.
- `src/lib/keybindings.ts`, `README.md`, and `README_zh.md`: user-facing shortcut references and terminal mode wording.
- `openspec/specs/terminal-panel-navigation/spec.md` and `openspec/specs/help-overlay/spec.md`: updated behavioral requirements and scenarios.
- No Rust/Tauri command, tab persistence, terminal process, or dependency changes are expected.
