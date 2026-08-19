## 1. Per-tab layout state

- [x] 1.1 Extend `TabState` defaults and snapshots with column ratios, preview expansion state, and original ratios.
- [x] 1.2 Update `layout.restoreTabState()` to restore the complete layout snapshot atomically.
- [x] 1.3 Wire layout snapshot save and restore through normal tab switches, MRU previews, and close-tab restoration.
- [x] 1.4 Verify a newly created tab starts with the default layout without inheriting its source tab's custom layout.

## 2. Custom window titlebar

- [x] 2.1 Disable native window decorations in the Tauri main-window configuration.
- [x] 2.2 Create a theme-aware `WindowTitlebar` component with a Tauri drag region and accessible window-control buttons.
- [x] 2.3 Implement minimize, maximize/restore, close, and maximized-state synchronization using the Tauri window API.
- [x] 2.4 Mount the titlebar above the tab bar and ensure its controls do not initiate window dragging.

## 3. Verification

- [x] 3.1 Run `npx svelte-check` and resolve only issues introduced by this change.
- [x] 3.2 Run `cargo check` from `src-tauri` to validate the Tauri configuration and Rust build path.
- [x] 3.3 Manually verify independent layouts across click, keyboard, MRU, and close-tab switching paths.
- [x] 3.4 Manually verify titlebar appearance in dark/light themes and Windows drag/minimize/maximize/restore/close behavior.
