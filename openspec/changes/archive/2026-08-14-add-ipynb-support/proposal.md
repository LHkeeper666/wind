## Why

Jupyter notebooks (.ipynb) are widely used in data science and Python development. Currently Wind shows them as raw JSON in the code editor — no preview, no formatting, no cell structure. Users browsing a project containing notebooks get a poor experience.

## What Changes

- Add `IpynbPreviewer` that parses .ipynb JSON and renders cells as styled HTML
- Markdown cells rendered with markdown-it, code cells highlighted with Shiki
- Output text/images displayed inline below each code cell
- Modify `PreviewEditor.loadFile()` to route .ipynb to preview mode instead of direct editor mode
- Pressing `e` still enters editor mode showing raw JSON for editing

## Capabilities

### New Capabilities
- `ipynb-preview`: Render Jupyter notebook (.ipynb) files as readable preview with cell structure, syntax-highlighted code, and formatted outputs

### Modified Capabilities

(none)

## Impact

- **Frontend only**: No Rust backend changes needed — .ipynb is JSON, parsed entirely in TypeScript
- **No new dependencies**: Reuses existing Shiki and markdown-it
- **New file**: `src/lib/previewers/IpynbPreviewer.ts` (~200 lines)
- **Modified files**: `PreviewRouter.ts` (registration), `PreviewEditor.svelte` (routing + keyboard shortcut), `index.ts` (export)
- **No breaking changes**
