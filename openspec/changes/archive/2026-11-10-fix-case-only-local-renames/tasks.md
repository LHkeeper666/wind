## 1. Shared Local Rename Behavior

- [x] 1.1 Add a shared backend helper that distinguishes exact-name collisions, unchanged names, and case-only aliases using parent directory entries and filesystem identity.
- [x] 1.2 Attempt direct rename, verify the stored entry casing, and implement a same-directory temporary-name fallback with recovery and explicit error reporting.
- [x] 1.3 Use the helper from both `rename_file` and `batch_rename` without changing their command arguments or batch error aggregation.

## 2. Verification

- [x] 2.1 Add focused tests for file and directory case-only renames in both directions, unchanged names, and batch rename.
- [x] 2.2 Test true destination collisions, including distinct hard links and a case-sensitive directory where supported; verify neither entry is overwritten.
- [x] 2.3 Test fallback success and recovery failure reporting with an injectable rename operation or another deterministic mechanism.
- [x] 2.4 Run the relevant Rust tests and `cargo check` from `src-tauri`, then manually verify the rename flow in the Windows UI.
