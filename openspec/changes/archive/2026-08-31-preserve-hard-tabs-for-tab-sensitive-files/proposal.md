## Why

Vim insert mode currently converts Tab input and automatic indentation into spaces for all file types. This breaks files where a literal tab has semantic meaning, especially Makefile recipes, causing `make` to reject otherwise valid targets.

## What Changes

- Preserve literal tab characters when editing tab-sensitive files.
- Detect Makefile-style files and use hard tabs for insert-mode Tab indentation and automatic line continuation where required.
- Treat TSV-style files as tab-delimited text where pressing Tab inserts a literal field separator.
- Keep the existing space-based behavior for ordinary files, including Markdown list indentation and code files that do not require hard tabs.
- Apply the behavior consistently in both the preview editor and fullscreen editor.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vim-editor`: Insert-mode Tab and indentation behavior changes for files whose syntax or data format requires literal tab characters.

## Impact

- Affects frontend editor text-key handling in `src/lib/utils/editor-text-keys.ts`.
- Affects editor initialization and per-file indentation policy in `PreviewEditor.svelte` and `FullscreenEditor.svelte`.
- May add focused tests around Tab insertion, multi-line indentation, Shift+Tab, and Enter continuation for Makefile and TSV-like files.
