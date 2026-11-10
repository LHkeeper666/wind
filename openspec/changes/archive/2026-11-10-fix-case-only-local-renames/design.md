## Context

`rename_file` and `batch_rename` each reject a destination when `Path::exists()` is true. On a case-insensitive Windows directory, a case-only destination resolves to the source, so both commands reject a valid operation. The frontend already supplies the source path and requested new name; FTP and archive commands are separate.

## Goals / Non-Goals

**Goals:**
- Support case-only changes for local files and directories through single and batch rename.
- Preserve collision protection, including case-sensitive directories and existing hard links or aliases.
- Report success only when the requested name casing is visible in the parent directory.

**Non-Goals:**
- Change FTP or archive renaming, the frontend command contract, or general batch scheduling semantics.
- Make a multi-entry batch atomic.

## Decisions

1. Share a local rename helper between the two commands. Inspect the parent directory's entry names with exact `OsStr` equality. An exact target entry other than the source is a collision; an exact source/target name is a no-op. For an existing destination without an exact target entry, verify that source and destination refer to the same filesystem object before treating it as a case alias. Use `symlink_metadata` to detect occupied names, including dangling symlinks, and a file-identity check (for example `same_file`) for the alias check. Exact entry comparison is essential because identity alone would accept two distinct hard links to one file. A case-insensitive string comparison alone would reject valid distinct names in a case-sensitive directory.
2. Attempt `fs::rename` directly first. Verify the requested entry name by reading the parent directory afterward. If the direct call fails or reports success without changing the stored casing for a confirmed case-only alias, use a unique temporary name in the same parent: source to temporary, then temporary to requested name. If the second rename fails, attempt to restore the original name and report both the original error and any recovery failure, including the temporary path. Do not delete or overwrite an occupied entry.
3. Keep the existing batch behavior: process each entry, collect errors, and return an error if any entry fails. Apply the shared helper per entry so case-only names work in both paths. Do not introduce implicit swaps or reorderings.

## Risks / Trade-offs

- Two-step fallback briefly changes the visible name and may trigger watcher refreshes twice. Use the direct rename path first and keep the temporary name in the same directory.
- A concurrent process may create the destination after validation. Recheck before each rename and propagate OS failures; the existing API does not provide an atomic no-replace primitive.
- Recovery can fail if permissions or external changes intervene. Preserve the temporary item and return its path rather than hiding the failure.
- File-identity checks can follow links; exact directory-entry checks must remain the first collision guard.

## Migration Plan

No data migration or frontend API change is required. Rollback is reverting the backend change; existing names remain ordinary filesystem entries.

## Open Questions

Validate on the project's supported Windows filesystems whether direct `fs::rename` updates the stored casing. Keep the fallback only where this validation shows it is needed.
