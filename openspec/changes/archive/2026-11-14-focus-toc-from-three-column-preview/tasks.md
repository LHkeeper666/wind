## 1. Navigation fix

- [x] 1.1 Remove the `previewExpanded` condition from `Ctrl+W l` outline entry in `keyboard-shortcuts.svelte.ts`, retaining outline visibility and focus checks.
- [x] 1.2 Apply the same condition change to directory-panel rightward navigation in `PanelLayout.svelte`.

## 2. Verification

- [x] 2.1 Verify Markdown and PDF outline focus with `Ctrl+W l` in three-column mode; verify navigation from a directory column takes one additional `Ctrl+W l` to enter the outline.
- [x] 2.2 Verify two-column and one-column behavior, hidden or absent outlines, and `Ctrl+W h` returning focus to preview content.
- [x] 2.3 Run `npx svelte-check` and resolve any issues caused by this change.
