## 1. RAR suffix handling

- [x] 1.1 Add a shared, Unicode-safe parser for ordinary RAR and supported volume suffixes; reject misleading or incomplete suffixes.
- [x] 1.2 Use the shared parser for archive format detection and password-cache RAR classification.
- [x] 1.3 Derive first-volume cache paths from original filename boundaries, preserving Unicode prefixes and padded `.partN.rar` numbering; retain non-RAR cache-key behavior.

## 2. Regression coverage

- [x] 2.1 Add Rust tests for Unicode and emoji paths across archive formats, including lengths that formerly split a multibyte character.
- [x] 2.2 Add Rust tests for lowercase and uppercase RAR suffixes, `.rXX`, `.partN.rar`, padded numbering, misleading `.part` text, and missing first-volume fallback.
- [x] 2.3 Verify password-cache sharing across valid RAR volumes and no unintended sharing with unrelated archives.

## 3. Verification

- [x] 3.1 Run the relevant Rust tests and `cargo check` from `src-tauri`; resolve any failures.
- [x] 3.2 Exercise read-only preview of the reported Unicode-path 7z archive, if accessible, and distinguish any subsequent archive or password error from the fixed panic.
