## Context

Local paste has two conflict paths. Top-level files use `promptConflict` in `clipboard-operations.ts`; pasted directories use `scan_transfer_conflicts` and a streamed modal for conflicting files inside the directory. The current scan returns only skip paths, and phase 2 reconstructs each transfer destination from the source basename. The streamed modal is also used by FTP and archive operations, so a local-only choice must be explicitly scoped.

## Goals / Non-Goals

**Goals:**
- Offer Keep Copy for top-level local files and conflicting files within locally pasted directories.
- Preserve the existing destination and source during a copy; preserve the existing destination during a cut/move and remove the source only after successful transfer.
- Keep existing conflict actions and `P` behavior.

**Non-Goals:**
- FTP upload/download, archive extraction, or a general rename command.
- Renaming a whole pasted directory when its top-level directory name already exists; existing directory merge behavior remains.

## Decisions

1. **Naming:** Insert ` (n)` before the final extension, starting at 1 and choosing the first name absent from the destination directory and the current paste batch. For extensionless names, append the suffix. Treat leading-dot names such as `.gitignore` as extensionless. This is predictable and works for repeated pastes.
2. **Decision flow:** Add `keep-copy` to the local conflict result. The top-level flow records an explicit destination for each source instead of rebuilding all destinations from basenames. The directory scan records a destination rename map keyed by source-relative path alongside its skip map. The local directory transfer uses that map to write the incoming file under its selected name. A local-only flag or separate modal configuration prevents the shared streamed dialog from exposing this action to FTP and archive callers.
3. **Transfer safety:** Select and reserve candidate names before queueing, then recheck at execution. Keep Copy must never delete or truncate an existing destination, including if another process creates the chosen name between scan and transfer. On collision at execution, choose another suffix or fail safely; the source remains intact on failure.
4. **Duplicate All policy:** The local streamed conflict scanner applies Keep Copy to its remaining conflicts and returns that policy to the top-level file loop. Choosing Duplicate All in the top-level file dialog applies it to later top-level conflicts. The policy is scoped to one paste invocation. FTP and archive callers do not see this choice.
   The top-level dialog offers Duplicate All only when the paste contains multiple clipboard entries.
5. **Transfer feedback:** Use the chosen destination for top-level transfer progress. Directory transfers continue to report the root directory while preserving renamed paths inside it.

## Risks / Trade-offs

- [A name is created after conflict resolution] → Recheck during execution and use exclusive creation or equivalent protection for Keep Copy targets.
- [Directory copy is cancelled after writing a renamed file] → Track its actual created path for cleanup; retain pre-existing files.
- [Cut paste with skipped nested paths] → Preserve the current filtered source deletion behavior while removing only successfully transferred items.
- [Shared conflict dialog leaks a local-only action] → Pass local capability explicitly instead of changing all streamed callers.

## Migration Plan

No data migration. Existing paste choices retain their behavior. Reverting the frontend and local transfer changes removes the new choice without changing stored data.

## Open Questions

None for this scoped change.
