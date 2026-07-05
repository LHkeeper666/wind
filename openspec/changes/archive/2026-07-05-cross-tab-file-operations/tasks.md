## 1. Clipboard Store

- [x] 1.1 Create `src/lib/stores/clipboard.ts` with ClipboardEntry interface and store (yank, cut, clear, getSummary, hasItems)
- [x] 1.2 Export derived store for clipboard summary text (e.g., "3 files yanked")

## 2. Rust Backend

- [x] 2.1 Add `move_file` Tauri command in `src-tauri/src/lib.rs` (same-drive rename, cross-drive copy+delete)
- [x] 2.2 Register `move_file` in `invoke_handler`

## 3. DirectoryPanel Multi-Select

- [x] 3.1 Add `selectedPaths: Set<string>` state to DirectoryPanel.svelte
- [x] 3.2 Implement Space key: toggle selection + cursor advance (skip `..`)
- [x] 3.3 Implement `v` key: select all / deselect all toggle
- [x] 3.4 Add visual styling for selected files (distinct background)
- [x] 3.5 Add visual styling for cut files (reduced opacity + `x` marker)
- [x] 3.6 Clear selection on directory navigation

## 4. Yank / Cut Keybindings

- [x] 4.1 Implement `y` key in DirectoryPanel: yank selected or cursor file to clipboard
- [x] 4.2 Implement `x` key in DirectoryPanel: cut selected or cursor file to clipboard
- [x] 4.3 Show toast notification with file count on yank/cut

## 5. Paste Operation

- [x] 5.1 Implement `p` key handler in PanelLayout.svelte global keydown
- [x] 5.2 Implement paste logic: iterate clipboard entries, invoke copy_file or move_file
- [x] 5.3 Clear clipboard and cut indicators after paste completes
- [x] 5.4 Show toast "Clipboard empty" when pasting with empty clipboard

## 6. Conflict Resolution Modal

- [x] 6.1 Create `ConfirmModal.svelte` component with Overwrite/Skip/Abort options
- [x] 6.2 Style modal with application CSS variables for theme consistency
- [x] 6.3 Integrate modal into paste flow: detect conflict, pause, await user choice
- [x] 6.4 Handle Overwrite: delete target then execute paste
- [x] 6.5 Handle Skip: continue to next file
- [x] 6.6 Handle Abort: stop entire paste operation

## 7. Status Bar & Commands

- [x] 7.1 Show clipboard summary in status bar (right side, before theme toggle)
- [x] 7.2 Add `:clip` command to command palette: show clipboard file list
- [x] 7.3 Add `:clear` command to command palette: clear clipboard

## 8. Keybindings Documentation

- [x] 8.1 Update `src/lib/keybindings.ts` with Space, v, y, x, p entries under Directory Panel group
