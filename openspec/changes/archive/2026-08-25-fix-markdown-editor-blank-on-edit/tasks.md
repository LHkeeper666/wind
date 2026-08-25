## 1. Implementation

- [x] 1.1 Add a bounded layout-stability helper for active CodeMirror sessions in `PreviewEditor.svelte`.
- [x] 1.2 Defer initial Markdown/editor scroll restoration until after the editor container and session host have measurable dimensions.
- [x] 1.3 Re-measure active editor sessions after activation, mode transition, and resize observer callbacks.
- [x] 1.4 Preserve Markdown list marker indentation when `Tab` is pressed from CodeMirror insert mode.

## 2. Verification

- [x] 2.1 Run `npx svelte-check --tsconfig ./tsconfig.json`.
- [x] 2.2 Run `npm run build`.
