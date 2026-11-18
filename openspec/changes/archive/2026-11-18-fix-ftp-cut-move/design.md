## Context

The 10:46 incident moved a directory between paths on one FTP connection. `RNFR/RNTO` returned 550; `ftp_rename` then used a file-only RETR/put/rm fallback. A directory cannot be retrieved as a file, and the shared session subsequently showed protocol errors. Cross-backend cut paths enqueue work but currently schedule source deletion before transfer completion. The existing folder scheduler expands directories into file tasks, but its move metadata does not provide a reliable completion barrier for all source cleanup.

## Goals / Non-Goals

**Goals:**
- Complete same-connection FTP file and directory moves when rename is unsupported, with observable progress and cancellation.
- Preserve all source content until destination transfer and required verification succeed.
- Return accurate per-entry status and permit retry after failure.

**Non-Goals:**
- Server-side FTP copy extensions or cross-server moves.
- Atomic directory moves across unrelated FTP paths; FTP has no transaction primitive.

## Decisions

1. Keep server-side rename as the fast path. Treat 550 as a failed rename, then branch on `FileEntry.is_dir` (validated against remote metadata where needed). Never issue RETR or RM for a directory. Reject moving a directory into itself or a descendant. An alternative of disabling directory moves would avoid the bug but would leave the user's operation unsupported.
2. Route fallback moves through transfer scheduling with a move group that owns a source, destination, all child tasks, and cleanup state. Use independent FTP sessions for file transfers; recursive listing provides directories and file sizes. Create target directories, enqueue files, and report group progress. A single synchronous `ftp_rename` fallback would block the UI and lack cancellation.
3. Completion is a barrier: only when every child task succeeds and destination files are verified against expected sizes does cleanup begin. For empty directories, successful target creation satisfies the barrier. Delete source files and then directories deepest first. On failed/cancelled transfer, leave source untouched and show the partial destination. On cleanup failure, report a distinct partial-move state, retain source paths, and allow retry without silent overwrite.
4. Apply the same barrier to FTP→local and local→FTP cut operations. Remove immediate source deletion from the front end; the transfer group performs cleanup. Do not reuse the current folder move metadata as a proxy for success until its lifecycle is corrected.
5. Check destination existence and apply the paste conflict choice before rename or transfer. Do not silently overwrite. Retain failed cut clipboard entries; show queued, completed, failed, and partially moved counts from actual outcomes. Refresh source and destination after terminal state.
6. Bound FTP operations with timeout and finalize/close a RETR data stream before reusing its control session. Reconnect stale sessions before retrying safe reads; do not blindly retry destructive cleanup commands without checking remote state.

## Risks / Trade-offs

- [Partial destination after failure] → Keep source intact, display partial state, and support explicit retry/conflict handling.
- [Source cleanup fails after verified copy] → Keep both copies and report cleanup failure; never claim a completed move.
- [Directory changes during transfer] → Compare scanned metadata with transferred results and abort cleanup if source changed; document that FTP cannot provide a snapshot.
- [Large directories consume time/bandwidth] → Stream files through bounded temporary storage or direct streaming, with per-file progress and cancellation; clean temporary files on every terminal path.

## Migration Plan

No data migration. Implement and verify the move group before switching cut entry points to it. Retain the existing rename fast path. Rollback by disabling the new fallback and returning a clear failure while preserving sources.

## Open Questions

- Confirm the desired conflict behavior for a partially populated target on retry; default to the existing paste conflict choices and never silently overwrite.
