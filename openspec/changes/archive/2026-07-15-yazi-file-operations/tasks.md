## 1. InputDialog Component

- [x] 1.1 Create `src/lib/components/InputDialog.svelte` with input field, placeholder, onConfirm/onCancel callbacks
- [x] 1.2 Style InputDialog to appear inline at top of directory panel content area
- [x] 1.3 Support pre-filled value (for rename) and placeholder text (for create)
- [x] 1.4 Handle Enter to confirm, Escape to cancel, restore panel focus on close

## 2. Hidden Files

- [x] 2.1 Add `is_hidden` field to FileEntry struct in Rust backend
- [x] 2.2 Set is_hidden based on FILE_ATTRIBUTE_HIDDEN or dot-prefix in read_directory
- [x] 2.3 Add `showHidden` state to DirectoryPanel
- [x] 2.4 Implement `.` key toggle for hidden files in DirectoryPanel
- [x] 2.5 Filter file list based on showHidden state
- [x] 2.6 Style hidden files with reduced opacity when displayed

## 3. File Rename (r key)

- [x] 3.1 Add `r` key handler in DirectoryPanel: show InputDialog with current file name
- [x] 3.2 On confirm: invoke rename_file, refresh directory, select renamed file
- [x] 3.3 Handle rename conflict: show error toast if name exists
- [x] 3.4 Support renaming selected files (multi-select: rename cursor file)

## 4. File Create (a key)

- [x] 4.1 Add `a` key handler in DirectoryPanel: show InputDialog in create-file mode
- [x] 4.2 Add `a/` key sequence handler: show InputDialog in create-directory mode
- [x] 4.3 On confirm: invoke create_file with is_dir flag, refresh directory, select new file
- [x] 4.4 Handle create conflict: show error toast if name exists

## 5. File Delete with Confirmation (d key)

- [x] 5.1 Add `d` key handler in DirectoryPanel: show confirmation dialog
- [x] 5.2 Support single file delete and multi-select delete
- [x] 5.3 On confirm: invoke delete_file, refresh directory
- [x] 5.4 Show toast with deleted file name(s)

## 6. Trash Support

- [x] 6.1 Add `trash = "4"` to Cargo.toml dependencies
- [x] 6.2 Modify delete_file Rust command to use trash::delete() instead of fs::remove_file
- [x] 6.3 Update delete confirmation message to say "move to trash"
- [x] 6.4 Show toast "Moved to trash: filename" on success

## 7. Permanent Delete (D key)

- [x] 7.1 Add `D` key handler in DirectoryPanel: show confirmation with permanent delete warning
- [x] 7.2 On confirm: invoke new permanent_delete command (fs::remove_file/remove_dir_all)
- [x] 7.3 Show toast with permanently deleted file name(s)

## 8. Force Paste (P key)

- [x] 8.1 Add `P` key handler in PanelLayout global keydown
- [x] 8.2 Modify handlePaste to accept force parameter
- [x] 8.3 When force=true: skip conflict modal, auto-delete existing files before paste

## 9. Keybindings Documentation

- [x] 9.1 Update `src/lib/keybindings.ts` with d, D, r, a, a/, ., P entries
