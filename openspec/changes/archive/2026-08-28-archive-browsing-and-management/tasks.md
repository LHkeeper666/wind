## 1. Backend: Archive module and dependencies

- [x] 1.1 Add `tar`, `flate2`, `sevenz-rust` dependencies to Cargo.toml
- [x] 1.2 Create `src-tauri/src/archive/mod.rs` with `ArchiveFormat` enum, `ArchiveReader` trait, and format detection from file extension
- [x] 1.3 Implement `ZipReader` struct (wrapping `zip::ZipArchive`) with `list_entries`, `read_file`, `extract_files`
- [x] 1.4 Implement `ZipWriter` struct with `delete_entries`, `rename_entry` (random access via `zip` crate)
- [x] 1.5 Implement `TarReader` struct (wrapping `tar::Archive`) with `list_entries`, `read_file`, `extract_files`
- [x] 1.6 Implement `TarGzReader` struct (wrapping `flate2::GzDecoder` + `tar::Archive`) with same methods
- [x] 1.7 Implement `SevenZReader` struct (wrapping `sevenz_rust::SevenZReader`) with `list_entries`, `read_file`, `extract_files`

## 2. Backend: Tauri commands

- [x] 2.1 Implement `read_archive_directory` command: takes `archive_path` + `internal_path`, returns `Vec<FileEntry>` with is_dir/size/name
- [x] 2.2 Implement `read_archive_file` command: takes `archive_path` + `internal_path`, returns file content as `Vec<u8>` (with 10MB limit, frontend handles truncation)
- [x] 2.3 Implement `extract_archive_files` command: takes `archive_path` + `internal_paths` + `dest_dir`, extracts selected files via transfer_enqueue
- [x] 2.4 Implement `extract_archive` command: takes `archive_path` + `dest_dir`, extracts entire archive via transfer_enqueue with progress events
- [x] 2.5 Implement `compress_files` command: takes `sources` + `dest_path`, creates ZIP archive via transfer_enqueue with progress events
- [x] 2.6 Implement `archive_delete_entry` command: takes `archive_path` + `internal_paths`, deletes entries (ZIP only, error for other formats)
- [x] 2.7 Implement `archive_rename_entry` command: takes `archive_path` + `old_path` + `new_path`, renames entry (ZIP only, error for other formats)
- [x] 2.8 Register all new commands in `tauri::generate_handler![]`

## 3. Frontend: ArchiveState and layout store

- [x] 3.1 Define `ArchiveFormat` type and `ArchiveState` interface in `src/lib/stores/layout.ts`
- [x] 3.2 Add `archiveState` field to layout store with `setArchiveState` and `clearArchiveState` methods
- [x] 3.3 Add `markType` state (`'copy' | 'cut' | 'extract' | null`) and `markPaths` to layout store for E/y/x mutual exclusion
- [x] 3.4 Implement `setMark(type, paths)` that clears previous mark before setting new one

## 4. Frontend: DirectoryPanel archive mode

- [x] 4.1 Modify `loadDirectory` in DirectoryPanel to call `read_archive_directory` when `archiveState !== null`
- [x] 4.2 Modify `handleKeydown` `l`/Enter: if archiveState present and entry is directory, update `archiveState.internalPath`; if entry is file, preview via `read_archive_file`
- [x] 4.3 Modify `handleKeydown` `h`: if in archive mode and at root, call `clearArchiveState`; if in subdirectory, go up one level
- [x] 4.4 Modify `handleKeydown` `R` (refresh): call `read_archive_directory` when in archive mode
- [x] 4.5 Update path bar display to show archive path when in archive mode
- [x] 4.6 Disable `c` and `e` keys in archive mode (no-op)

## 5. Frontend: E/y/x mutual exclusion and p dispatch

- [x] 5.1 Bind `E` key in DirectoryPanel to call `setMark('extract', [archivePath])` with archive format validation
- [x] 5.2 Modify existing `y` handler to call `setMark('copy', paths)` in addition to clipboard set
- [x] 5.3 Modify existing `x` handler: in archive mode -> extract selected files; in normal mode -> call `setMark('cut', paths)`
- [x] 5.4 Modify `p` handler to dispatch: `extract` -> invoke `extract_archive`; `copy` -> paste copy; `cut` -> paste cut; `null` -> toast "Clipboard empty"
- [x] 5.5 Update `P` (force paste) handler: skip conflict prompt for extract too

## 6. Frontend: Preview from archive

- [x] 6.1 Modify `PreviewEditor.loadFile` to detect archive mode and call `read_archive_file` instead of `read_file` when `archiveState !== null`
- [x] 6.2 Apply previewer routing to archive file content (text detection, image, etc.) based on internal file extension

## 7. Frontend: Compress and extract shortcuts

- [x] 7.1 Bind `c` key in DirectoryPanel (normal mode): prompt for archive name, invoke `compress_files` with selected files
- [x] 7.2 Bind `e` key in DirectoryPanel (normal mode, on archive file): invoke `extract_archive` to current directory
- [x] 7.3 Transfer Manager integration: ensure `extract_archive` and `compress_files` emit progress events that Transfer Manager displays

## 8. Frontend: Archive operations (delete/rename in ZIP)

- [x] 8.1 Bind `d` key in archive mode: if ZIP -> invoke `archive_delete_entry`; if non-ZIP -> toast "Delete is only supported for ZIP archives"
- [x] 8.2 Bind `r` key in archive mode: if ZIP -> inline rename input then invoke `archive_rename_entry`; if non-ZIP -> toast "Rename is only supported for ZIP archives"
- [x] 8.3 After delete/rename, refresh the archive directory listing

## 9. Status bar and UI polish

- [x] 9.1 Update status bar to show "Archive marked for extraction" when markType is 'extract'
- [x] 9.2 Update keybindings documentation in `keybindings.ts` with new shortcuts
- [x] 9.3 Add toast notifications for archive operations (extract started, compress complete, delete confirmed)
- [x] 9.4 Handle error cases: corrupted archive, unsupported format, permission denied, file not found

## 10. Archive file editing (ZIP only)

- [x] 10.1 Implement `archive_write_file` Rust command: rewrite ZIP with modified entry content
- [x] 10.2 Modify `PreviewEditor.saveFile` to detect archive context and call `archive_write_file`
- [x] 10.3 Add `archiveEditPath` / `archiveEditInternalPath` state tracking for archive save

## 11. Paste files INTO archive (ZIP only)

- [x] 11.1 Implement `archive_add_files` Rust command: rewrite ZIP with new entries from source paths
- [x] 11.2 Modify `handlePaste` in PanelLayout to detect archive mode and add files to archive
- [x] 11.3 Add `archivePath` field to ClipboardState for archive yank tracking

## 12. Create files and directories inside archive (ZIP only)

- [x] 12.1 Implement `archive_create_entry` Rust command: rewrite ZIP with new empty file or directory entry
- [x] 12.2 Modify `handleInputConfirm` in DirectoryPanel to handle 'create-file' and 'create-dir' in archive mode
- [x] 12.3 Show format-specific error toast when creating in non-ZIP archives

## 13. Compress mark (C + p)

- [x] 13.1 Add `'compress'` to `MarkType` in layout store
- [x] 13.2 Bind `C` key in DirectoryPanel to mark selected files for compression via `setMark('compress', paths)`
- [x] 13.3 Add compress mark handling in PanelLayout `handlePaste`: show InputDialog for archive name
- [x] 13.4 Implement `handleCompressDialogConfirm` / `handleCompressDialogCancel` in PanelLayout

## 14. Bug fixes

- [x] 14.1 Fix `..` disappearing after refresh in archive mode: move `..` addition to `normalDisplayFiles` $derived
- [x] 14.2 Fix `is_dir()` check in `read_zip_file` / `read_tar_file` / `read_tar_gz_file` to distinguish empty files from directories
- [x] 14.3 Fix `h` key focus loss when exiting archive: set `pendingSelectName` before `handleArchiveUp()`