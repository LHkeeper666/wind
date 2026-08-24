## Context

Tab shortcuts are currently implemented as a one-second `t` prefix in the capture-phase global handler and as separate state machines in preview and editor modes. The global handler also uses the physical `t` key to keep the MRU tab switcher open until keyup. This duplicates command routing, makes the runtime mapping drift from the help and README tables, and consumes Vim's `t` motion in editor Normal mode.

The replacement must work in the Windows Tauri WebView, where Alt is also used by system and keyboard-layout interactions. The existing global keydown listener runs in the capture phase, so it can safely own only an explicit set of application shortcuts before terminals, editors, or panels consume them.

## Goals / Non-Goals

**Goals:**
- Provide immediate global tab actions through `Alt+<key>` chords in non-editable contexts.
- Preserve a hold-to-cycle MRU/physical-order tab switcher using Alt release as its commit signal.
- Restore unmodified `t` to Vim and remove all tab-prefix timeout state.
- Keep the help registry and both READMEs aligned with the runtime mapping.

**Non-Goals:**
- Add configurable keybindings or change tab persistence, tab-bar mouse controls, or Tauri/Rust APIs.
- Intercept arbitrary Alt shortcuts, bare Alt, system shortcuts, or `Ctrl+Alt` (AltGr) input.
- Change terminal or editor insert-mode semantics.

## Decisions

### Decision 1: Use direct Alt chords, not an Alt latch

The canonical mapping is `Alt+T` (new), `Alt+C` (close), `Alt+R` (rename hint), `Alt+N` (MRU switcher), `Alt+H`/`Alt+L` (display-order switcher backward/forward), `Alt+,`/`Alt+.` (swap), `Alt+D` (toggle left-panel detach), and `Alt+1`–`Alt+9` (select by index).

Pressing and releasing Alt before the second key is not supported: Windows uses bare Alt for system interaction and it would require another timeout state machine. Direct chords are immediate, accessible in keybinding displays, and match conventional modifier semantics.

### Decision 2: Centralize dispatch in `PanelLayout`

`PanelLayout`'s capture-phase handler becomes the sole dispatcher for tab commands. It checks the existing modal/fullscreen/editable-context gate, requires `altKey && !ctrlKey && !metaKey`, matches only the canonical codes, then prevents default and stops propagation. `PreviewEditor` no longer recognizes tab-operation keys locally; its unmodified `t` key continues to flow to CodeMirror Vim.

Keeping component-local handlers would retain divergent behavior between preview, editor, terminal, and fullscreen states. Moving the handler to individual panels would make the global shortcut contract regress when new panels are added.

### Decision 3: Let Alt hold the switcher open

`Alt+N` initializes the MRU switcher. `Alt+L` and `Alt+H` initialize the display-order switcher in forward and backward directions respectively. While Alt remains held, further `N`, `L`, or `H` keydowns advance the corresponding order and direction; the `AltLeft` or `AltRight` keyup commits the preview selection. The `tHeld` state and prefix timeout are replaced by an Alt-held state, and switcher events are consumed only while that state is active.

This preserves the existing preview-before-commit behavior while making a tap of `Alt+N` commit naturally when Alt is released. A timeout-based prefix would be unnecessary and would make a held modifier unreliable.

### Decision 4: Treat the registry as the displayed mapping source of truth

The Tab group in `src/lib/keybindings.ts` lists every supported Alt binding with its exact runtime semantics. The help overlay renders this registry unchanged; README files copy the same mapping. The implementation and docs must be reconciled rather than preserving the stale `t n`/`t x`/`t [` tables.

## Risks / Trade-offs

- [Risk] Alt can trigger WebView or Windows defaults → Mitigation: handle only known chords in capture phase and call `preventDefault()`; leave bare Alt, `Alt+F4`, and `Alt+Space` untouched.
- [Risk] AltGr reports as Ctrl+Alt and could corrupt non-US keyboard input → Mitigation: reject all shortcuts when `ctrlKey` is set.
- [Risk] A lost Alt keyup can leave the switcher active → Mitigation: clear or commit switcher state on window focus loss and retain the existing defensive cleanup paths.
- [Risk] Alt chords are useful editor bindings → Mitigation: do not intercept them in editor insert mode, command inputs, search fields, or other editable controls. The terminal currently has no separate input-mode state, so its focused view keeps the established global tab-shortcut behavior.

## Migration Plan

1. Deploy the shortcut replacement and documentation updates together; no persisted state or backend migration is required.
2. Remove `t`-prefix behavior without compatibility aliases so unmodified `t` is immediately available to Vim and ordinary input.
3. If a regression is found, revert the frontend shortcut change as one unit; tab data and terminal state are unaffected.

## Open Questions

None. The confirmed mapping uses direct `Alt+<key>` chords.
