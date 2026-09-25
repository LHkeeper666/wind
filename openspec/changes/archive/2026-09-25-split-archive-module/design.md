## Context

`src-tauri/src/archive/mod.rs` is 1663 lines, mixing 7 concerns. The module handles ZIP, TAR, TAR.GZ, and 7z archive operations with password management, encoding detection, and shared path utilities all in one file. The public API (9 functions) dispatches by `ArchiveFormat` to format-specific implementations.

Current file structure is a flat single file. No existing sub-modules.

## Goals / Non-Goals

**Goals:**
- Split into 7 files with clear single responsibilities
- Preserve all public API signatures (zero external caller changes)
- Maintain `cargo check` passing at each intermediate step
- Eliminate TAR/TAR.GZ code duplication (~95% identical)

**Non-Goals:**
- Changing any behavior or adding new features
- Modifying existing specs (`archive-browsing`, `archive-encoding-detection`, `archive-operations`)
- Refactoring the `zip` crate or `sevenz_rust` crate usage patterns
- Adding tests (existing behavior is preserved, not changed)

## Decisions

### 1. Module structure: 5 sub-modules + shared + encoding

**Decision:** `password.rs`, `encoding.rs`, `shared.rs`, `zip.rs`, `tar.rs`, `seven_z.rs` with `mod.rs` as dispatch.

**Alternatives considered:**
- 4 modules (merge encoding into shared) — rejected because encoding has its own static cache and distinct responsibility
- Keep TAR/TAR.GZ separate — rejected because ~95% code duplication

### 2. Format-specific password functions stay with format modules

**Decision:** `map_zip_error()`, `is_zip_password_error()`, `zip_entry_requires_password()`, `validate_zip_password()`, `read_zip_entry()` go to `zip.rs`. `map_7z_error()`, `make_7z_password()` go to `seven_z.rs`. Only generic password cache/resolution/error-message helpers go to `password.rs`.

**Rationale:** These functions depend on format-specific types (`zip::result::ZipError`, `sevenz_rust::Error`). Moving them to `password.rs` would create reverse dependencies.

### 3. ZIP central directory parsing stays in `zip.rs`

**Decision:** `ZipCdEntry`, `parse_zip_central_dir()`, `find_eocd()`, `read_cd_info()` go to `zip.rs`, not `encoding.rs`.

**Rationale:** These are ZIP binary format protocol details. `encoding.rs` only needs `detect_archive_encoding()` and `decode_name()` which are format-agnostic.

### 4. TAR + TAR.GZ merged into one module

**Decision:** Single `tar.rs` with separate entry-point functions (`list_entries` vs `list_gz_entries`).

**Rationale:** The code is ~95% identical — only the `File::open` → `GzDecoder` wrapping differs. Internal generic functions can eliminate duplication.

### 5. `ArchiveFormat` stays in `mod.rs`

**Decision:** Keep the enum in `mod.rs` alongside the dispatch functions.

**Rationale:** It's the public API's type. External callers import it from `crate::archive::ArchiveFormat`.

### 6. All sub-module functions are `pub(crate)`

**Decision:** Sub-module functions use `pub(crate)` visibility, not `pub`.

**Rationale:** External code should only access through `mod.rs` dispatch. This prevents bypassing the format-detection layer.

## Risks / Trade-offs

**[Risk] `&'static` encoding cache lifetime** → `encoding_rs::Encoding` references are inherently `'static`. Moving `ENCODING_CACHE` across modules has no lifetime impact.

**[Risk] TAR generic constraints** → `tar::Archive<File>` vs `tar::Archive<GzDecoder<File>>` need compatible `Read` bounds. `decode_tar_name` already uses `impl Read` so this should work. Verify with `cargo check` after extraction.

**[Risk] Missed internal dependency** → Extract in dependency order (shared → password → encoding → tar → zip → seven_z → mod.rs) and `cargo check` after each step.

**[Trade-off]** Slightly more files to navigate for archive work → offset by each file being focused and under 560 lines.
