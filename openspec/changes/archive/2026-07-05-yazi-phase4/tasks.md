## 1. File Info

- [x] 1.1 Add `get_file_info` Rust command returning FileMetadata struct (name, path, size, is_dir, created, modified, accessed, is_readonly, is_hidden, is_system)
- [x] 1.2 Create `src/lib/components/FileInfoPanel.svelte` overlay component displaying file metadata
- [x] 1.3 Add `i` key handler in DirectoryPanel to toggle FileInfoPanel
- [x] 1.4 Format file sizes as B/KB/MB/GB/TB with appropriate precision
- [x] 1.5 Register `get_file_info` in invoke_handler and update keybindings.ts

## 2. File Sort

- [x] 2.1 Add sort state to DirectoryPanel: `sortBy` (name/size/ext), `sortReverse` (bool), `dirFirst` (bool, default true)
- [x] 2.2 Implement sort logic in DirectoryPanel: apply sort to displayFiles after filter
- [x] 2.3 Add `s` prefix key handler with sub-keys: `n` (name), `s` (size), `e` (ext), `r` (reverse), `t` (dir-first toggle)
- [x] 2.4 Show toast on sort change indicating current mode
- [x] 2.5 Update keybindings.ts with s prefix entries

## 3. File Filter

- [x] 3.1 Add `filterPattern` state to DirectoryPanel
- [x] 3.2 Add `f` key handler showing InputDialog in filter mode
- [x] 3.3 Implement glob matching logic (support `*` and `?` wildcards)
- [x] 3.4 Apply filter in displayFiles derived: filter files matching pattern (always include `..`)
- [x] 3.5 Show active filter in panel header when filter is set
- [x] 3.6 Update keybindings.ts with f entry

## 4. Open With

- [x] 4.1 Add `open` crate to Cargo.toml dependencies
- [x] 4.2 Add `open_file` Rust command using `open::that()` for default program
- [x] 4.3 Add `open_with_dialog` Rust command using `open::with()` for system picker
- [x] 4.4 Change `o` key in DirectoryPanel/PanelLayout to call `open_file` (file) or reveal in explorer (dir)
- [x] 4.5 Add `O` key handler to call `open_with_dialog`
- [x] 4.6 Register new commands in invoke_handler and update keybindings.ts

## 5. Batch Rename

- [x] 5.1 Add `batch_rename` Rust command: accept list of (old_path, new_name) pairs, execute renames
- [x] 5.2 Modify `r` key handler: if multi-selection exists, enter batch rename mode instead of InputDialog
- [x] 5.3 Implement batch rename flow: write filenames to temp file, open Neovim in floating terminal
- [x] 5.4 Parse Neovim output after editor closes, compare with original names
- [x] 5.5 Call batch_rename with changed files, handle conflicts (skip + warn)
- [x] 5.6 Check Neovim availability, show error toast if not found
- [x] 5.7 Update keybindings.ts with batch rename note on r entry
