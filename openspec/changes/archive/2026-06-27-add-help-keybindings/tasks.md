## 1. Keybinding Registry

- [x] 1.1 Create `src/lib/keybindings.ts` with `Keybinding`, `KeybindingGroup` interfaces and `keybindingGroups` data covering all 6 groups (Global, Directory Panel, Tab, Terminal, Preview/Editor, Commands)

## 2. HelpOverlay Component

- [x] 2.1 Create `src/lib/components/HelpOverlay.svelte` with fixed overlay, multi-column layout rendering from `keybindingGroups`, and any-key-to-close behavior

## 3. PanelLayout Integration

- [x] 3.1 Add `Help` command to the `commands` array in `PanelLayout.svelte`
- [x] 3.2 Add `F1` key handler in `handleGlobalKeydown` to show help overlay (with same modal/palette blocking checks)
- [x] 3.3 Handle `help` command in `handleCommandKeydown` Enter handler (close palette, show overlay)
- [x] 3.4 Change ratio presets from `Set Ratio 1:2:2`/`Set Ratio 1:1:1` to `ratio 1:2:2`/`ratio 1:1:1`
- [x] 3.5 Import and render `HelpOverlay` component in PanelLayout template

## 4. Verification

- [x] 4.1 Run `npx svelte-check` to verify no type errors
- [ ] 4.2 Manual test: F1 opens help, any key closes it, focus restores correctly
- [ ] 4.3 Manual test: `:help` command opens help overlay
- [ ] 4.4 Manual test: ratio presets show as `ratio X:Y:Z` format

