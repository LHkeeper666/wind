## Context

Wind's preview system uses a `PreviewRouter` with pluggable previewers. Each previewer implements `match(filePath)` and `render(content, container)`. The `PreviewEditor.loadFile()` function reads file content and passes it to the router.

Current state:
- `read_directory` command returns `Vec<FileEntry>` without file sizes
- Directories are navigated into (Enter) but never previewed (j/k triggers `onSelect` already, but `loadFile()` doesn't handle directories)
- Archives are treated as unsupported binary files

## Goals / Non-Goals

**Goals:**
- Preview directory contents (one level) with file names and sizes
- Preview zip archive structure with entry paths and sizes
- j/k selection on directories shows preview; Enter still navigates into directory

**Non-Goals:**
- Recursive directory tree preview (only one level)
- Archive extraction or file content preview inside archives
- Other archive formats (tar.gz, 7z, rar) — future extension point, not in this change
- Editing files inside archives

## Decisions

### 1. Reuse `read_directory` for directory preview, add `size` field to `FileEntry`

**Decision**: Add `size: Option<u64>` to the existing `FileEntry` struct rather than creating a separate command.

**Rationale**: `FileEntry` is already used by `DirectoryPanel`. Adding an optional size field is backward-compatible (serde defaults `Option` to `None`). DirectoryPreviewer can call the same `read_directory` command. No new Rust command needed for directories.

**Alternative considered**: Separate `read_directory_preview` command with a different struct. Rejected because it duplicates logic and adds maintenance burden.

### 2. New `list_archive_entries` command with `ArchiveEntry` struct

**Decision**: Create a new `ArchiveEntry` struct (`name`, `path`, `is_dir`, `size`) and a `list_archive_entries` Tauri command. Use the `zip` crate for zip file parsing.

**Rationale**: Archive entries have different semantics from filesystem entries (virtual paths, no real filesystem metadata). A separate struct keeps concerns clean. The `zip` crate is mature and widely used.

**Alternative considered**: Reusing `FileEntry` with a flag. Rejected because `children: Option<Vec<FileEntry>>` and `path` semantics don't map cleanly to archive internals.

### 3. Frontend detection: directory vs archive vs file

**Decision**: In `loadFile()`, check in order: directory (try `invoke('read_directory')`, catch = not a dir) → archive (extension check `.zip`) → image → text → unsupported.

**Rationale**: Directory check via `read_directory` is reliable and already exists. Archive detection by extension is simple and sufficient for now. No need for magic-byte detection.

### 4. Pure HTML rendering for DirectoryPreviewer and ArchivePreviewer

**Decision**: Both new previewers render using plain HTML (no Shiki syntax highlighting).

**Rationale**: These are structured data displays, not code. Shiki adds latency with no benefit. Simple HTML with CSS styling matches the existing preview panel aesthetic.

### 5. Visual design: file icons and size formatting

**Decision**: Use Unicode characters for icons (`📁` for dirs, `📄` for files, `📦` for archive header). Format sizes with B/KB/MB/GB units. Entries are sorted: directories first, then files, alphabetically.

**Rationale**: Consistent with the existing three-column layout aesthetic. Sorting matches the existing `read_directory` behavior.

## Risks / Trade-offs

**[Risk] Large directories cause slow preview** → Mitigation: `read_directory` is already used by DirectoryPanel and handles large dirs. No recursion, only one level.

**[Risk] Large zip files cause slow preview** → Mitigation: `zip` crate reads central directory (metadata only), not file contents. Fast even for large archives.

**[Risk] j/k rapid navigation triggers many `read_directory` calls** → Mitigation: 200ms debounce already exists in DirectoryPanel. User confirmed this is acceptable.

**[Risk] `FileEntry.size` field breaks existing deserialization** → Mitigation: `Option<u64>` defaults to `None` when missing. Existing callers unaffected.
