## Why

Preview panel currently only supports text files and images. Selecting a directory shows nothing, and archive files (.zip) are treated as unsupported. Users browsing a project cannot preview folder structures or inspect archive contents without opening them externally.

## What Changes

- Add `DirectoryPreviewer` to render a flat list of directory contents (name, type, size) in the preview panel
- Add `ArchivePreviewer` to render zip archive entry trees in the preview panel
- Add Rust backend commands: `read_directory_preview` (directory listing with file sizes) and `list_archive_entries` (zip structure)
- Modify `PreviewEditor.loadFile()` to detect directories and archives, routing them to the appropriate previewer
- j/k navigation on a directory in DirectoryPanel already triggers `onSelect`; this now shows a preview instead of blank

## Capabilities

### New Capabilities
- `directory-preview`: Preview a directory's contents (one level) in the preview panel, showing subdirectories and files with sizes
- `archive-preview`: Preview zip archive entry structure in the preview panel, showing nested paths with sizes

### Modified Capabilities

(none)

## Impact

- **Rust backend**: New `zip` crate dependency; new `ArchiveEntry` struct; new Tauri commands; `FileEntry` gains `size: Option<u64>` field
- **Frontend**: New `DirectoryPreviewer.ts`, `ArchivePreviewer.ts`; updates to `PreviewRouter`, `PreviewEditor.svelte`
- **No breaking changes**: `FileEntry.size` is `Option`, existing callers unaffected
