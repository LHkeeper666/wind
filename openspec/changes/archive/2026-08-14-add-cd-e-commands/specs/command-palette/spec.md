## Command Palette: cd and e commands with Tab completion

### `:cd` command

Switches the current tab's directory.

- `:cd path` — navigate to absolute or relative path
- `:cd ..` — navigate to parent directory
- `:cd` (no args) — navigate to home directory
- Path not found → toast: `E344: Can't find directory: <path>`
- Only affects current tab

Path resolution:
- Absolute: starts with `X:\`, `X:/`, `\`, or `/` → use directly
- Relative: prepend `currentPath + '\'`
- Both `/` and `\` accepted, normalized to `\` internally

### `:e` command

Opens a file or navigates to a directory.

- `:e filepath` — select file in preview panel
- `:e dirpath` — navigate to directory
- `:e .` — refresh current directory
- `:e` (no args) — refresh current directory
- Path not found → toast: `E344: Can't find directory: <path>`

### Tab completion

Pressing Tab in the command input triggers path completion.

- Parse input as `(command, pathPrefix)`
- Determine parent directory and partial name from pathPrefix
- Call `read_directory` on parent to get entries
- Filter: `cd` → only is_dir; `e` → files + directories
- Single match: fill immediately with trailing `\` for directories
- Multiple matches: Tab cycles through them
- No match: no-op
- Any non-Tab keypress resets completion state
