## 1. Extract shared utilities (no internal dependencies)

- [x] 1.1 Create `archive/shared.rs` with `normalize_internal()`, `matches_internal_path()`, `is_dir_in_entries()`, `collect_entries_at_path()` — all `pub(crate)`
- [x] 1.2 Add `mod shared;` to `mod.rs`, replace moved function bodies with `use shared::*` or qualified calls
- [x] 1.3 Run `cargo check` to verify compilation

## 2. Extract password management (no internal dependencies)

- [x] 2.1 Create `archive/password.rs` with `ARCHIVE_PASSWORD_CACHE`, `archive_cache_key()`, `cached_archive_password()`, `store_archive_password()`, `resolve_archive_password()`, `remember_archive_password()`, `password_required_error()`, `password_incorrect_error()` — all `pub(crate)`
- [x] 2.2 Add `mod password;` to `mod.rs`, replace moved function bodies
- [x] 2.3 Run `cargo check` to verify compilation

## 3. Extract encoding detection (no internal dependencies)

- [x] 3.1 Create `archive/encoding.rs` with `ENCODING_CACHE`, `detect_archive_encoding()`, `detect_encoding_from_bytes()`, `decode_name()`, `decode_tar_name()` — all `pub(crate)`
- [x] 3.2 Add `mod encoding;` to `mod.rs`, replace moved function bodies
- [x] 3.3 Run `cargo check` to verify compilation

## 4. Extract TAR operations (depends on shared + encoding)

- [x] 4.1 Create `archive/tar.rs` with all `list_tar_*`, `read_tar_*`, `extract_tar_*` functions (TAR + TAR.GZ merged) — all `pub(crate)`
- [x] 4.2 Add `mod tar;` to `mod.rs`, replace moved function bodies
- [x] 4.3 Run `cargo check` to verify compilation

## 5. Extract ZIP operations (depends on shared + encoding + password)

- [x] 5.1 Create `archive/zip.rs` with `ZipCdEntry`, `parse_zip_central_dir()`, `find_eocd()`, `read_cd_info()`, `is_zip_password_error()`, `map_zip_error()`, `zip_entry_requires_password()`, `validate_zip_password()`, `read_zip_entry()`, all `list/read/extract/delete/rename/add/write/create_zip_*` functions, `compress_files()`, `add_dir_to_zip()` — all `pub(crate)`
- [x] 5.2 Add `mod zip;` to `mod.rs`, replace moved function bodies
- [x] 5.3 Run `cargo check` to verify compilation

## 6. Extract 7z operations (depends on shared + password)

- [x] 6.1 Create `archive/seven_z.rs` with `make_7z_password()`, `map_7z_error()`, all `list/read/extract_7z_*` functions — all `pub(crate)`
- [x] 6.2 Add `mod seven_z;` to `mod.rs`, replace moved function bodies
- [x] 6.3 Run `cargo check` to verify compilation

## 7. Finalize mod.rs as dispatch layer

- [x] 7.1 Clean up `mod.rs`: remove all moved function bodies, keep only `mod` declarations, `use` imports, `ArchiveFormat` enum, and 9 public dispatch functions
- [x] 7.2 Verify `mod.rs` is ~120 lines with no implementation logic
- [x] 7.3 Run `cargo check` to verify compilation
- [x] 7.4 Run `cargo build` to verify full build

## 8. Verification

- [x] 8.1 Verify all `pub` function signatures in `mod.rs` are unchanged from original
- [x] 8.2 Verify no `pub` functions exist in sub-modules (all should be `pub(crate)`)
- [x] 8.3 Run `npx svelte-check` to verify frontend type checking still passes
