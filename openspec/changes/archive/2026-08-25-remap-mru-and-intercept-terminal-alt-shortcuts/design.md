## Context

Tab shortcuts are dispatched by the capture-phase global keyboard handler in `PanelLayout.svelte`. The current mapping uses `Alt+T` for new tab and `Alt+N` for the MRU switcher; the switcher also consumes repeated `N` while Alt is held. The same handler intentionally skips editable targets, but xterm.js receives keyboard input through a textarea inside `.terminal-containers`, so supported Alt tab commands currently bypass the application when the terminal is focused.

The terminal has no reliable application-level Insert/Normal mode state in the current implementation. The desired contract is therefore one terminal focus context: supported Alt tab chords belong to Wind whether the terminal is embedded or fullscreen, while ordinary unsupported Alt input remains available to the shell.

## Goals / Non-Goals

**Goals:**

- Remap `Alt+N` to new tab and `Alt+M` to MRU switching.
- Preserve display-order switching on `Alt+H`/`Alt+L` and all other supported Alt tab actions.
- Intercept the complete supported Alt tab set from terminal xterm input, including fullscreen terminal focus.
- Remove terminal Insert/Normal distinctions from the shortcut registry and user documentation.
- Preserve modal, generic editable-input, unsupported Alt, and AltGr behavior.

**Non-Goals:**

- Do not add a terminal mode state machine or change shell input/output handling.
- Do not intercept arbitrary Alt combinations or `Ctrl+Alt` keyboard-layout input.
- Do not change tab persistence, tab ordering algorithms, Tauri commands, or terminal process management.

## Decisions

### Decision 1: Keep shortcut ownership centralized in `PanelLayout`

The existing capture-phase global handler remains the only tab shortcut dispatcher. This keeps directory, preview, editor Normal, terminal, and fullscreen-terminal behavior consistent and avoids adding competing xterm event handlers.

### Decision 2: Use the revised canonical mapping

The canonical mapping becomes `Alt+N` new tab, `Alt+M` MRU switcher, `Alt+H`/`Alt+L` display-order backward/forward, `Alt+U` close, `Alt+R` rename hint, `Alt+,`/`Alt+.` swap, `Alt+D` detach toggle, and `Alt+1`–`Alt+9` indexed selection. While the MRU switcher is active, repeated `M` advances MRU selection; repeated `H`/`L` continues to move in display order. One-shot actions ignore key-repeat events so holding `Alt+N` does not create multiple tabs.

### Decision 3: Add a narrow terminal exception to the editable-target gate

The global handler continues to reject generic `INPUT`, `TEXTAREA`, contenteditable, command, search, and dialog targets. It makes an exception only when the target is inside the terminal container and the event is a recognized supported Alt tab chord with no Ctrl or Meta modifier. The handler then prevents default and stops propagation before xterm forwards the chord to the shell. Unsupported Alt input and AltGr remain untouched.

### Decision 4: Treat terminal as one documented focus context

The keybinding registry and README files no longer advertise terminal Escape/`i` Insert/Normal mode controls. The terminal section instead states that supported Alt tab shortcuts are intercepted while terminal focus is active; shell input remains responsible for all other keys.

## Risks / Trade-offs

- [Risk] A terminal user may expect a shell-specific supported Alt chord → Mitigation: reserve only the explicit tab mapping; leave all unsupported Alt chords to xterm.
- [Risk] A terminal textarea is not reliably identified by `activeColumn` alone → Mitigation: check the event target's terminal-container ancestry and the recognized chord set.
- [Risk] Alt keyup may be lost during focus changes → Mitigation: retain existing switcher commit and focus-loss cleanup paths.
- [Risk] Windows/WebView menu handling may compete with Alt chords → Mitigation: consume only supported chords in the capture phase and call `preventDefault()`.

## Migration Plan

1. Update the central mapping, switcher repeat key, and terminal target gate.
2. Update the registry, bilingual README files, and terminal-navigation/help-overlay requirements together.
3. Run static checks and manually verify every supported Alt chord from embedded and fullscreen terminal focus.
4. If a regression occurs, revert this frontend/doc change as one unit; no persisted or backend migration is required.

## Open Questions

None. Terminal focus intentionally includes both the embedded and fullscreen terminal, with no application-level Insert/Normal distinction.
