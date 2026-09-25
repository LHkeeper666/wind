## ADDED Requirements

### Requirement: Archive module split into sub-modules
The `src-tauri/src/archive/` module SHALL be organized into sub-modules with clear single responsibilities: `password.rs` (password cache/management), `encoding.rs` (encoding detection), `shared.rs` (path utilities), `zip.rs` (ZIP operations), `tar.rs` (TAR/TAR.GZ operations), `seven_z.rs` (7z operations), and `mod.rs` (public API dispatch + ArchiveFormat).

#### Scenario: Public API preserved
- **WHEN** external code calls `crate::archive::list_entries`, `extract_files`, `compress_files`, or any other public function
- **THEN** the function signatures and behavior SHALL be identical to the pre-refactor version

#### Scenario: No behavioral changes
- **WHEN** any archive operation is performed (list, read, extract, delete, rename, add, compress)
- **THEN** the results SHALL be identical to the pre-refactor behavior
