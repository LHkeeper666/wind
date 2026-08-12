## 1. Frontend — IpynbPreviewer

- [x] 1.1 Create `src/lib/previewers/IpynbPreviewer.ts` implementing `Previewer` interface
  - `match()`: detect `.ipynb` extension
  - `render()`: parse JSON, iterate cells, build styled HTML
  - Markdown cells → markdown-it rendered HTML
  - Code cells → Shiki syntax highlighted source + output display
  - Raw cells → plain `<pre>` block
  - Cell type badges (Markdown/Code/Raw)
  - `dispose()`: cleanup container
- [x] 1.2 Add CSS for notebook preview styling (cell borders, input/output areas, badges)

## 2. Frontend — Integration

- [x] 2.1 Register `IpynbPreviewer` in `PreviewRouter.ts`
- [x] 2.2 Export `IpynbPreviewer` in `src/lib/previewers/index.ts`
- [x] 2.3 Update `PreviewEditor.svelte` `loadFile()` routing for .ipynb
- [x] 2.4 Update `PreviewEditor.svelte` `isTextFile()` already allows .ipynb editing
- [x] 2.5 `E` fullscreen editor already works for .ipynb (no additional change needed)

## 3. Validation

- [x] 3.1 Test build passes (`vite build` succeeds)
- [x] 3.2 Test TypeScript type checking (`svelte-check` no new errors)
- [ ] 3.3 Test with a real .ipynb file (manual — run `npm run tauri dev`)
