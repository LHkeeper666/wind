## Why

When a local `p` paste encounters a file with the same name, users must currently overwrite, skip, or cancel. They need a way to keep both files without manually renaming one before pasting.

## What Changes

- Add a Keep Copy choice to local `p` paste conflict dialogs for top-level files and files inside pasted directories.
- Add a Duplicate All choice that keeps suffixed copies for the current and all later conflicts in the same local paste.
- Name a kept copy with the first available numeric suffix before its extension, such as `report (1).pdf`.
- Route the selected destination name through the transfer task for both copy and cut paste.
- Preserve the current Overwrite, Skip, Abort, and existing apply-to-all behavior. `P` continues to force overwrite.
- Limit this change to local-to-local paste; FTP transfers and archive extraction are outside its scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `paste-conflict-resolution`: Allow local paste conflicts to retain both files by assigning the incoming file a unique suffixed name.

## Impact

The local clipboard paste flow, conflict dialog state and buttons, conflict scan results, and local transfer destination handling will change. No new external dependency or database change is required.
