## 1. Rust Backend — Data Structures

- [x] 1.1 Add `size: Option<u64>` field to `FileEntry` struct in `src-tauri/src/lib.rs`
- [x] 1.2 Add `ArchiveEntry` struct (`name`, `path`, `is_dir`, `size`) in `src-tauri/src/lib.rs`
- [x] 1.3 Add `zip = "2"` to `src-tauri/Cargo.toml` dependencies

## 2. Rust Backend — Commands

- [x] 2.1 Populate `size` field in `read_directory` using `metadata.len()` for files, `None` for directories
- [x] 2.2 Implement `list_archive_entries(path)` command using `zip` crate to read central directory
- [x] 2.3 Register `list_archive_entries` in `invoke_handler` in `src-tauri/src/lib.rs`

## 3. Frontend — DirectoryPreviewer

- [x] 3.1 Create `src/lib/previewers/DirectoryPreviewer.ts` with `match()` (detect directory paths) and `render()` (HTML list with icons and sizes)
- [x] 3.2 Implement `formatSize()` helper for human-readable B/KB/MB/GB display

## 4. Frontend — ArchivePreviewer

- [x] 4.1 Create `src/lib/previewers/ArchivePreviewer.ts` with `match()` (`.zip` extension) and `render()` (HTML tree with header and entry list)

## 5. Frontend — Integration

- [x] 5.1 Register `ArchivePreviewer` in `PreviewRouter.ts` (before existing previewers)
- [x] 5.2 Update `PreviewEditor.svelte` `loadFile()` to detect directories (try `invoke('read_directory')`) and archives (extension check), loading content for the new previewers
- [x] 5.3 Add CSS for directory/archive preview entries in `PreviewEditor.svelte`
