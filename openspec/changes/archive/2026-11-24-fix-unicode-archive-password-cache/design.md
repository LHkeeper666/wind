## Context

All archive reads resolve a cached password before format-specific processing. `password.rs` tests for RAR volume suffixes with byte-indexed string slices, so a Unicode path to any supported archive can panic. The format dispatcher in `mod.rs` uses a separate RAR suffix check. Multipart RAR cache keys normalize to the first volume; other archive keys include canonical path, size, and modification time.

## Goals / Non-Goals

**Goals:**
- Eliminate invalid UTF-8 boundary slicing from archive password-key generation.
- Keep format detection and password-cache RAR volume recognition consistent.
- Preserve first-volume password sharing and existing non-RAR cache invalidation behavior.
- Cover Unicode, case, misleading suffixes, and volume naming in tests.

**Non-Goals:**
- Change archive decoding, password prompts, or public Tauri command signatures.
- Diagnose or repair the contents of the reported 7z file.
- Redesign password-cache lifetime or persistence.

## Decisions

1. Parse suffixes from the filename, using safe string operations (`strip_suffix`, `ends_with`, and character-aware boundaries) instead of byte offsets computed from total path length. Match ASCII extensions and volume digits without lowercasing the entire path before calculating an index into the original string. This also avoids Unicode case-folding length mismatches. Alternative: use `get(range)` around the existing indices; that would prevent panic but could silently fail valid suffix recognition.
2. Put RAR suffix parsing in one shared helper consumed by both archive format detection and password-key normalization. Accept ordinary `.rar`, two-digit `.rXX`, and suffix-anchored `.partN.rar` with one or more ASCII digits, case-insensitively. A `.part` sequence earlier in a filename is not a volume marker. Alternative: fix only the unsafe slice in `password.rs`; this would leave two recognition implementations to diverge.
3. Build the first-volume path from the original path prefix, replacing only the matched ASCII suffix. Preserve the digit width for `.partN.rar` when deriving `.part1.rar` or `.part01.rar`; preserve the existing `.rXX` to `.rar` mapping. Canonicalize the candidate when possible, retaining the current fallback for absent first volumes. Non-RAR keys retain path, size, and modified-time components. Alternative: normalize every volume to the currently opened file; this would lose cross-volume password sharing.
4. Test the parser and cache-key behavior with temporary Unicode paths and representative volume names. Add a read-only regression path through archive listing and, if practical, extraction password resolution. The real 7z file is an optional read-only manual verification target, never a test fixture committed to the repository.

## Risks / Trade-offs

- [RAR sets using a different first-volume naming convention] → Keep current `.rXX` convention and document/test the `.partN.rar` width rule; do not broaden recognition to unrelated formats.
- [Canonicalization fallback when the first volume is missing] → Retain existing behavior and test that it remains stable; do not treat a missing first volume as a panic.
- [A separate 7z decoding or password error appears after the panic is fixed] → Report that error independently; successful cache-key generation is not proof that archive contents are valid.

## Migration Plan

No stored data or API migration is needed. The password cache is process-local; deploying the backend change starts with an empty cache. Revert the code change if regression tests reveal a format-recognition issue.

## Open Questions

None blocking the implementation. The observed 7z file's actual readability can only be confirmed after the panic is fixed.
