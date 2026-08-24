## Why

The two-key `t` prefix delays every tab action, conflicts with Vim's native `t` motion in editor Normal mode, and is implemented by multiple divergent state machines. Replacing it with direct Alt shortcuts makes tab actions immediate, preserves Vim input, and gives the tab switcher a familiar modifier-held interaction.

## What Changes

- **BREAKING** Replace all tab-operation `t` sequences with direct `Alt+<key>` shortcuts: `Alt+T`, `Alt+C`, `Alt+R`, `Alt+N`, `Alt+H`, `Alt+L`, `Alt+,`, `Alt+.`, `Alt+D`, and `Alt+1` through `Alt+9`.
- Remove the one-second `t` prefix timeout and status-prefix display for tab operations.
- Use the held Alt modifier to control the existing tab switcher: `Alt+N` cycles most-recently-used tabs, while `Alt+H`/`Alt+L` cycle display order backward/forward; releasing Alt commits the selected tab.
- Centralize tab-shortcut handling in the global capture-phase keyboard handler and remove duplicate `t`-prefix handling from preview and editor modes.
- Update the help registry, English and Chinese documentation, and affected terminal-navigation requirements to describe the actual supported shortcut set.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-panel-navigation`: Replace the globally available `t`-prefix tab operations with Alt-based shortcuts, including fullscreen terminal behavior and editable-context exclusions.
- `help-overlay`: Display the new Alt-based tab bindings from the declarative keybinding registry.

## Impact

- `src/lib/components/PanelLayout.svelte`: global shortcut dispatch, modifier-held tab switcher lifecycle, and tab-command routing.
- `src/lib/components/PreviewEditor.svelte`: remove local `t`-prefix state machines so Vim's `t` motion remains available.
- `src/lib/keybindings.ts`, `README.md`, and `README_zh.md`: shortcut reference updates and reconciliation with runtime behavior.
- No Rust, Tauri command, persistence, or dependency changes.
