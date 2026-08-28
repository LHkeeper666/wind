## 1. Rust Backend: Tauri Commands

- [x] 1.1 Add `list_recycle_bin` command that calls `trash::os_limited::list()` and returns `Vec<TrashItemDto>` with id, name, original_path, date_deleted, size fields
- [x] 1.2 Add `restore_recycle_items` command that takes `Vec<String>` ids and calls `trash::os_limited::restore_all()`, handling `RestoreCollision` error
- [x] 1.3 Add `purge_recycle_items` command that takes `Vec<String>` ids and calls `trash::os_limited::purge_all()`
- [x] 1.4 Add `empty_recycle_bin` command using `SHEmptyRecycleBinW` (trash crate v4.1.1 has no public `empty()`)
- [x] 1.5 Register all new commands in `run()` function's `generate_handler![]` macro

## 2. Frontend: RecycleBinPanel Component

- [x] 2.1 Create `RecycleBinPanel.svelte` with file list rendering (name, original path, date deleted, size columns)
- [x] 2.2 Implement `j/k` navigation, `gg`/`G` jump, `Space` multi-select in recycle bin panel
- [x] 2.3 Implement `Enter` to select file and trigger preview in preview panel
- [x] 2.4 Implement `r` restore, `d` permanent delete, `gd` empty recycle bin key handlers
- [x] 2.5 Implement `h` to exit recycle bin mode
- [x] 2.6 Disable `y`/`x`/`a` operations in recycle bin mode (no-op - not registered in handler)
- [x] 2.7 Add loading state and empty state display

## 3. Frontend: PanelLayout Integration

- [x] 3.1 Add `recycleBinMode` field to layout store (`src/lib/stores/layout.ts`)
- [x] 3.2 Add `gr` keyboard shortcut in `PanelLayout.svelte` to toggle `recycleBinMode`
- [x] 3.3 Render `RecycleBinPanel` in current column when `recycleBinMode` is true (replacing `DirectoryPanel`)
- [x] 3.4 Render recycle bin overview in parent column (stats + shortcut hints) when `recycleBinMode` is true
- [x] 3.5 Auto-exit recycle bin mode when `cd` command is executed from command palette
- [x] 3.6 Ensure global shortcuts (Ctrl+`, Ctrl+P, `:`, Ctrl+W, zoom) work normally in recycle bin mode

## 4. Frontend: Preview Integration

- [x] 4.1 Wire selected recycle bin item's `id` to the existing preview system (pass `id` as file path to `read_file`)
- [x] 4.2 Display original path, deletion date, and size in preview panel header for recycle bin files
- [x] 4.3 Handle preview errors gracefully (permission denied, file corrupted)

## 5. Confirmation Dialogs & Error Handling

- [x] 5.1 Add confirmation dialog for permanent delete (`d` key) using existing `ConfirmModal`
- [x] 5.2 Add confirmation dialog for empty recycle bin (`gd` key) using existing `ConfirmModal`
- [x] 5.3 Add collision dialog for restore when target path already exists
- [x] 5.4 Add error toast for failed operations (restore failure, purge failure, empty failure)

## 6. Keybinding Documentation

- [x] 6.1 Add recycle bin mode keybindings to `src/lib/keybindings.ts` for HelpOverlay display
- [x] 6.2 Add recycle bin mode section in help overlay showing `r`/`d`/`gd`/`h`/`gr` shortcuts