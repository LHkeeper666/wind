## 1. CSS Variable

- [x] 1.1 Add `--zoom-level: 1` CSS variable in `src/lib/styles/themes.css`

## 2. Panel Zoom Application

- [x] 2.1 Apply `zoom: var(--zoom-level)` to `.panel` elements in `PanelLayout.svelte`

## 3. Keyboard Shortcuts

- [x] 3.1 Add `zoomLevel` state variable in `PanelLayout.svelte` (default 1.0)
- [x] 3.2 Add `Ctrl+=` handler to increase zoom by 0.1 (max 2.0)
- [x] 3.3 Add `Ctrl+-` handler to decrease zoom by 0.1 (min 0.5)
- [x] 3.4 Add `Ctrl+0` handler to reset zoom to 1.0
- [x] 3.5 Update `--zoom-level` CSS variable on `:root` when zoom level changes
- [x] 3.6 Show toast with "Zoom: X%" when zoom level changes

## 4. Verification

- [x] 4.1 Run `npx svelte-check` to verify no type errors
- [ ] 4.2 Manually test Ctrl+= / Ctrl+- / Ctrl+0 in dev mode
