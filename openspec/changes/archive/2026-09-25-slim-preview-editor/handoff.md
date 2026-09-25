# Slim PreviewEditor.svelte — Design & Handoff

## 1. Current Structure

`src/lib/components/PreviewEditor.svelte` — 1518 lines, ~41 functions, 37 `$state` variables.

Already extracted sub-components:
- `TextEditorHost.svelte` (528 lines) — CodeMirror editor host
- `VimOverlay.svelte` (465 lines) — Vim command overlay
- `PreviewPane.svelte` (519 lines) — Preview rendering + Markdown TocSidebar

### 1.1 State Variables by Domain

| Domain | Variables | Count |
|--------|-----------|-------|
| Core content | `content`, `savedContent`, `binaryContent`, `thumbnailMeta`, `videoMeta`, `originalFileSize`, `isModified` | 7 |
| Mode | `mode`, `suppressModeEffect`, `editorInitInFlight` | 3 |
| Editor refs | `panelElement`, `editorView`, `editorFilePath`, `archiveEditPath`, `archiveEditInternalPath`, `textEditorHost`, `vimOverlay`, `previewPane` | 8 |
| Editor position | `editorTargetLine`, `pendingEditorPos`, `pendingEditorScrollTop` | 3 |
| Text readiness | `readyTextContent`, `loadGeneration`, `renderTrigger`, `codeFileDirectEdit`, `directEdit` | 5 |
| **PDF** | `pdfPageCount`, `pdfCurrentPage`, `pdfFileSize`, `pdfTitle`, `pdfPageDimensions`, `pdfOutline`, `pdfTocOpen`, `pdfTocFocused`, `pdfTocSidebar`, `pdfPreviewPanel` | **10** |
| **Markdown TOC** | `tocHeadings`, `tocActiveLine`, `tocFocused`, `tocOpen` | **4** |
| Tab cache | `tabCache` (instance), `pendingRestoreScrollTop`, `pendingTocSelectedIndex`, `pendingTocExpanded` | 4 |
| File watch | `currentFileMtime`, `fileChangedUnlisten` | 2 |
| Misc | `isMarkdown`, `isDirectory`, `clipboardBridge` | 3 |

### 1.2 Functions by Domain

**Tab lifecycle (5):** `clearTabCache`, `cacheTabState`, `deactivateTab`, `getEditorStateSnapshot`, `prepareTabFocus`

**Exported API (15):** `focusActiveInput`, `getMode`, `getIsModified`, `enterEditorMode`, `pressTab`, `pressShiftTab`, `isTocVisible`, `focusToc`, `focusContent`, `isTocFocused`, `setContent`, `getContent`, `getFile`, `getPdfInfo`, `togglePdfToc`, `jumpToPdfPage`, `getVisibleLine`

**PDF (2 exported):** `togglePdfToc`, `jumpToPdfPage`

**File watch (3):** `startWatching`, `stopWatching`, `handleFileChanged`

**Editor helpers (6):** `destroyEditorSession`, `activateEditorSession`, `hideEditorSessions`, `focusActiveEditor`, `scrollPreview`, `moveCursorToLine`, `scrollEditorToPos`, `initEditor`

**Core (3):** `loadFile` (~350 lines), `handleKeydown` (~30 lines), `saveFile` (~30 lines)

**Internal (3):** `isCurrentTextSnapshot`, `handleTocFocusChange`, `getFileName`

### 1.3 The `loadFile` Problem

`loadFile` is the single largest function (~350 lines). It handles **7 distinct file type paths**:

| Path | Lines | State Written |
|------|-------|---------------|
| Cache hit | ~60 | All content/mode/PDF/TOC vars |
| Archive directory | ~35 | content, binaryContent, mode |
| Archive file | ~80 | content, binaryContent, readyTextContent, mode |
| Directory | ~10 | content, binaryContent, mode, isDirectory |
| Image | ~35 | content, binaryContent, thumbnailMeta |
| PDF | ~20 | pdfPageCount, pdfCurrentPage, pdfFileSize, pdfTitle, pdfPageDimensions, pdfOutline, pdfTocOpen |
| Archive (top-level) | ~8 | content, binaryContent, mode |
| Video | ~15 | videoMeta, content, binaryContent |
| Text/binary | ~65 | content, binaryContent, savedContent, readyTextContent, currentFileMtime, originalFileSize, mode |

All paths share: `gen` (loadGeneration counter), `loadTabId` (renderTabId), archive state detection, and mode assignment. Each path mutates multiple `$state` variables.

---

## 2. Dependency Graph

```
filePath ──────► loadFile ──────┬──► content, binaryContent, mode (all types)
    │                          ├──► pdfPageCount, pdfCurrentPage, ... (PDF only)
    │                          ├──► tocHeadings, tocActiveLine (Markdown only)
    │                          └──► startWatching()
    │
    ├─► mode ──► mode $effect ──┬──► activateEditorSession / initEditor
    │                          ├──► hideEditorSessions
    │                          └──► focusActiveEditor
    │
    ├─► readyTextContent ──► codeFileDirectEdit ──► directEdit
    │
    └─► renderTabId ◄── previewTabId ?? currentTabId

tabCache ◄──► loadFile (cache hit/miss), cacheTabState (serialize), clearTabCache (destroy)

pdfOutline ──► pdfTocOpen ──► template (PdfTocSidebar)
pdfPreviewPanel ◄── scrollToPage, focusPanel
pdfTocSidebar ◄── focus (TOC focus management)

tocHeadings ──► tocOpen ──► template (TocSidebar in PreviewPane)
tocFocused ──► isTocFocused ──► mode indicator display

file-changed event ──► handleFileChanged ──► loadFile (reload)
```

---

## 3. Proposed Extraction Plan

### 3.1 `createPdfState` (composable)

**Extracts from PreviewEditor:** 10 `$state` variables + 3 functions + template PDF section

**Owns:**
```ts
// State
pdfPageCount: number
pdfCurrentPage: number
pdfFileSize: number
pdfTitle: string | null
pdfPageDimensions: PdfPageDimensions[]
pdfOutline: PdfOutlineItem[]
pdfTocOpen: boolean
pdfTocFocused: boolean
pdfTocSidebar: PdfTocSidebar | undefined
pdfPreviewPanel: PdfPreviewPanel | undefined

// Functions
togglePdfToc()
jumpToPdfPage(page: number)
focusPdfPanel()
focusPdfToc()
reset()  // clear all state when filePath changes away from PDF

// Cache integration
toCacheSnapshot(): { pdfCurrentPage, pdfPageCount, pdfPageDimensions, pdfOutline, pdfTocOpen }
fromCacheSnapshot(cached): void
```

**API:**
```ts
interface PdfStateDeps {
  getFilePath: () => string | null;
  onToast: (msg: string) => void;
}

interface PdfStateAPI {
  // Getters (reactive)
  getPdfPageCount: () => number;
  getPdfCurrentPage: () => number;
  getPdfFileSize: () => number;
  getPdfTitle: () => string | null;
  getPdfPageDimensions: () => PdfPageDimensions[];
  getPdfOutline: () => PdfOutlineItem[];
  getPdfTocOpen: () => boolean;
  getPdfTocFocused: () => boolean;

  // Refs (for bind:this)
  pdfPreviewPanel: PdfPreviewPanel | undefined;
  pdfTocSidebar: PdfTocSidebar | undefined;

  // Actions
  togglePdfToc: () => void;
  jumpToPdfPage: (page: number) => void;
  focusPanel: () => void;
  focusToc: () => void;
  reset: () => void;

  // Cache integration
  toCacheSnapshot: () => PdfCacheData;
  fromCacheSnapshot: (data: PdfCacheData) => void;

  // Load integration
  loadPdfInfo: (path: string, gen: number) => Promise<void>;
}

function createPdfState(deps: PdfStateDeps): PdfStateAPI
```

**How it integrates with `loadFile`:**
```ts
// In loadFile, PDF branch:
if (isPdfFile(path)) {
  content = ''; binaryContent = null; mode = 'global-normal';
  previewPane?.clearActiveSlot();
  await pdfState.loadPdfInfo(path, gen);
  return;
}
```

**How it integrates with template:**
```svelte
{#if filePath && isPdfFile(filePath) && mode === 'global-normal'}
  <div class="pdf-with-toc">
    <PdfPreviewPanel
      bind:this={pdfState.pdfPreviewPanel}
      pdfPath={filePath}
      pageDimensions={pdfState.getPdfPageDimensions()}
      pageCount={pdfState.getPdfPageCount()}
      fileSize={pdfState.getPdfFileSize()}
      title={pdfState.getPdfTitle()}
      onPageChange={(page) => { pdfState.setPage(page); }}
      onFullscreen={() => onFullscreen()}
    />
    {#if pdfState.getPdfOutline().length > 0 && pdfState.getPdfTocOpen()}
      <PdfTocSidebar
        bind:this={pdfState.pdfTocSidebar}
        outline={pdfState.getPdfOutline()}
        currentPage={pdfState.getPdfCurrentPage()}
        pageDimensions={pdfState.getPdfPageDimensions()}
        onJump={(page, y) => pdfState.jumpToPage(page, y)}
        onFocusChange={(focused) => pdfState.setTocFocused(focused)}
        onExit={() => focusContent()}
      />
    {/if}
  </div>
{/if}
```

**Lines removed from PreviewEditor:** ~80 (state declarations + functions + PDF template wiring)

---

### 3.2 `createMarkdownTocState` (composable)

**Extracts from PreviewEditor:** 4 `$state` variables + 2 functions

**Owns:**
```ts
// State
tocHeadings: TocHeading[]
tocActiveLine: number
tocFocused: boolean
tocOpen: boolean

// Functions
handleTocFocusChange(focused: boolean)
reset()  // clear when filePath changes away from Markdown

// Cache integration
toCacheSnapshot(): { tocOpen, tocHeadings, tocExpandedLines, tocFocused, tocSelectedIndex }
fromCacheSnapshot(cached): void
```

**API:**
```ts
interface MarkdownTocDeps {
  getPanelElement: () => HTMLElement | undefined;
}

interface MarkdownTocAPI {
  getTocHeadings: () => TocHeading[];
  getTocActiveLine: () => number;
  getTocFocused: () => boolean;
  getTocOpen: () => boolean;
  setTocOpen: (open: boolean) => void;
  setTocFocused: (focused: boolean) => void;
  handleTocFocusChange: (focused: boolean) => void;
  onHeadingsChange: (headings: TocHeading[]) => void;
  onActiveLineChange: (line: number) => void;
  reset: () => void;
  toCacheSnapshot: (getTocSelectedIndex: () => number) => MarkdownTocCacheData;
  fromCacheSnapshot: (data: MarkdownTocCacheData) => void;
}

function createMarkdownTocState(deps: MarkdownTocDeps): MarkdownTocAPI
```

**Lines removed from PreviewEditor:** ~30

---

### 3.3 `createFileWatcher` (composable)

**Extracts from PreviewEditor:** 2 `$state` variables + 3 functions + event listener setup

**Owns:**
```ts
// State
currentFileMtime: number
fileChangedUnlisten: (() => void) | null

// Functions
startWatching(path: string): void
stopWatching(): void
getFileMtime(): number
setFileMtime(mtime: number): void

// Lifecycle
setup(): void   // listen('file-changed', ...)
teardown(): void
```

**API:**
```ts
interface FileWatcherDeps {
  getFilePath: () => string | null;
  getMode: () => string;
  getRenderTabId: () => number;
  onFileChanged: (path: string) => Promise<void>;
}

interface FileWatcherAPI {
  startWatching: (path: string) => void;
  stopWatching: () => void;
  getFileMtime: () => number;
  setFileMtime: (mtime: number) => void;
  setup: () => void;
  teardown: () => void;
}

function createFileWatcher(deps: FileWatcherDeps): FileWatcherAPI
```

**Integration:**
```ts
// In PreviewEditor:
const fileWatcher = createFileWatcher({
  getFilePath: () => filePath,
  getMode: () => mode,
  getRenderTabId: () => renderTabId,
  onFileChanged: async (changedPath) => {
    // current handleFileChanged logic
  },
});

onMount(() => fileWatcher.setup());
onDestroy(() => fileWatcher.teardown());
```

**Lines removed from PreviewEditor:** ~35

---

### 3.4 Refactor `loadFile` into Sub-Loaders

**Approach:** Keep `loadFile` in PreviewEditor as an orchestrator, but extract per-type loading logic into pure-ish helper functions.

**Create `src/lib/utils/file-loaders.ts`:**

```ts
// Context shared by all loaders
interface LoadContext {
  gen: number;
  loadTabId: number;
  filePath: string;
  archiveState: ArchiveState | null;
  selectedEntryIsDir: boolean | null;
}

// Each loader returns a result object; PreviewEditor applies it to $state
interface LoadResult {
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  mode: 'global-normal' | 'editor-normal' | 'editor-insert';
  isDirectory: boolean;
  // PDF-specific (only set by loadPdf)
  pdfInfo?: { pageCount: number; currentPage: number; fileSize: number; title: string | null; pageDimensions: PdfPageDimensions[]; outline: PdfOutlineItem[]; tocOpen: boolean };
  // Image-specific
  thumbnailMeta?: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null;
  // Video-specific
  videoMeta?: VideoMeta | null;
  // For partial content
  originalFileSize?: number;
  savedContent?: string;
  currentFileMtime?: number;
  // Editor init flag
  editorInitInFlight?: boolean;
  // Archive paths
  archiveEditPath?: string | null;
  archiveEditInternalPath?: string | null;
}

// Pure-ish loader functions (async, use invoke, but don't touch $state)
async function loadArchiveFile(ctx: LoadContext): Promise<LoadResult>
async function loadArchiveDirectory(ctx: LoadContext): Promise<LoadResult>
async function loadDirectory(ctx: LoadContext): Promise<LoadResult>
async function loadImage(ctx: LoadContext): Promise<LoadResult>
async function loadPdf(ctx: LoadContext): Promise<LoadResult>
async function loadArchive(ctx: LoadContext): Promise<LoadResult>
async function loadVideo(ctx: LoadContext): Promise<LoadResult>
async function loadTextOrBinary(ctx: LoadContext): Promise<LoadResult>
```

**Simplified `loadFile` in PreviewEditor:**
```ts
async function loadFile(path: string) {
  const gen = ++loadGeneration;
  const loadTabId = renderTabId;
  const ctx: LoadContext = { gen, loadTabId, filePath: path, archiveState, selectedEntryIsDir };

  // Cache hit
  const cached = tabCache.get(loadTabId);
  if (cached && cached.filePath === path && !browsingArchive) {
    applyCacheResult(cached, gen, loadTabId, path);
    return;
  }

  // Reset state
  resetForNewLoad();

  // Route to loader
  let result: LoadResult;
  if (browsingArchive && selectedEntryIsDir) result = await loadArchiveDirectory(ctx);
  else if (browsingArchive) result = await loadArchiveFile(ctx);
  else if (await isDirectory(path)) result = await loadDirectory(ctx);
  else if (isImageFile(path)) result = await loadImage(ctx);
  else if (isPdfFile(path)) result = await loadPdf(ctx);
  else if (isArchiveFile(path)) result = await loadArchive(ctx);
  else if (isVideoFile(path)) result = await loadVideo(ctx);
  else result = await loadTextOrBinary(ctx);

  // Apply result to $state
  applyLoadResult(result, gen, loadTabId, path);
}
```

**Lines removed from PreviewEditor:** ~250 (loadFile body moves to file-loaders.ts)

---

### 3.5 CSS Extraction — NOT Recommended

The 600+ lines of preview-specific CSS (Markdown, ipynb, JSON, image, PDF) are in the `<style>` block. Extracting them to a separate CSS file is possible but:
- Svelte's scoped CSS is useful here (prevents style leaks)
- The styles reference `:global()` selectors already
- CSS extraction doesn't reduce component logic complexity
- It's a separate concern from the composable extraction

**Keep inline.** If desired later, extract to `src/lib/styles/preview.css` as global styles.

---

## 4. What NOT to Extract

### 4.1 Mode Effect — Keep Inline

The mode `$effect` block (lines 327-398, ~70 lines) orchestrates editor session activation, scroll position sync, and focus management. It depends on `mode`, `editorView`, `textEditorHost`, `vimOverlay`, `previewPane`, `readyTextContent`, `editorTargetLine`, `activeColumn`, `codeFileDirectEdit`, `panelElement`, `tocFocused`, and `renderTabId`.

Extracting this into a composable would require passing ~15 dependencies — diminishing returns. The effect is the _orchestration layer_ that ties all composables together. Keep it in PreviewEditor.

### 4.2 Keyboard Handler — Keep Inline

`handleKeydown` (~30 lines) dispatches to PDF panel, TOC focus, editor mode, preview scroll, and save. It's the central keyboard router. Extracting it would add indirection without reducing complexity.

### 4.3 saveFile — Keep Inline

`saveFile` (~30 lines) handles both local and archive writes. It's small and directly mutates `content`, `savedContent`, `isModified`, `currentFileMtime`, and `tabCache`. Keep inline.

### 4.4 Tab Cache Serialization — Keep Inline

`cacheTabState`, `clearTabCache`, `deactivateTab`, `getEditorStateSnapshot` (~50 lines total) are serialization glue between PreviewEditor's state and `TabCacheManager`. They reference state from multiple composables (PDF, TOC, editor, content). Moving them to a separate module would require passing all state — keep inline but call composable `toCacheSnapshot()` methods.

---

## 5. Component Structure After Extraction

```
PreviewEditor (orchestrator, ~700 lines)
├── createPdfState() → PDF state + actions + loadPdfInfo
├── createMarkdownTocState() → TOC state + focus management
├── createFileWatcher() → file-changed event handling
├── Core state (~15 $state vars: content, mode, editor refs, etc.)
├── loadFile orchestrator (calls file-loaders.ts helpers)
├── Mode effect (orchestrates editor sessions)
├── Keyboard handler
├── saveFile
├── Tab cache serialization (calls composable snapshots)
├── Exported API functions
└── Template
    ├── PDF section (delegates to pdfState getters)
    ├── PreviewPane (receives tocState getters)
    ├── TextEditorHost
    ├── VimOverlay
    └── Welcome screen
```

---

## 6. Task Breakdown

### Phase 1: Extract `createPdfState`

**Files:**
- Create: `src/lib/composables/pdf-state.svelte.ts`
- Modify: `src/lib/components/PreviewEditor.svelte`

**Steps:**
1. Create composable with 10 PDF state variables
2. Move `togglePdfToc`, `jumpToPdfPage` into composable
3. Add `loadPdfInfo(path, gen)` — extracted from `loadFile`'s PDF branch (~20 lines)
4. Add `toCacheSnapshot()` / `fromCacheSnapshot()` for tab cache integration
5. Add `focusPanel()` / `focusToc()` / `reset()` helpers
6. In PreviewEditor: instantiate `createPdfState`, remove PDF state vars, update `loadFile` PDF branch to call `pdfState.loadPdfInfo()`
7. Update template PDF section to use `pdfState` getters
8. Update `cacheTabState` / cache-hit path to use `pdfState.toCacheSnapshot()` / `fromCacheSnapshot()`
9. Update `isTocVisible`, `focusToc`, `focusContent`, `getPdfInfo`, `togglePdfToc`, `jumpToPdfPage` exports to delegate

**Risk:** Low. PDF state is well-isolated. The only coupling is `pdfPreviewPanel?.focusPanel()` in `handlePanelFocus` — pass as composable method.

**Estimated reduction:** ~80 lines

---

### Phase 2: Extract `createMarkdownTocState`

**Files:**
- Create: `src/lib/composables/markdown-toc-state.svelte.ts`
- Modify: `src/lib/components/PreviewEditor.svelte`

**Steps:**
1. Create composable with 4 TOC state variables
2. Move `handleTocFocusChange` into composable
3. Add `onHeadingsChange` / `onActiveLineChange` callbacks (called by PreviewPane)
4. Add `toCacheSnapshot(getTocSelectedIndex)` / `fromCacheSnapshot()` for tab cache
5. Add `reset()` to clear headings when switching away from Markdown
6. In PreviewEditor: instantiate composable, remove TOC state vars
7. Update template to pass `tocState` getters to PreviewPane
8. Update `isTocVisible`, `focusToc`, `focusContent` exports
9. Update `cacheTabState` / cache-hit path

**Risk:** Low. TOC state has minimal coupling — `tocFocused` is checked in `handlePanelFocus` and mode effect, but these can read from composable getter.

**Estimated reduction:** ~30 lines

---

### Phase 3: Extract `createFileWatcher`

**Files:**
- Create: `src/lib/composables/file-watcher.svelte.ts`
- Modify: `src/lib/components/PreviewEditor.svelte`

**Steps:**
1. Create composable with `currentFileMtime` and `fileChangedUnlisten`
2. Move `startWatching`, `stopWatching`, `handleFileChanged` logic
3. Set up `listen('file-changed', ...)` in `setup()`, clean up in `teardown()`
4. In PreviewEditor: instantiate composable, call `setup()` in event listener (or onMount), `teardown()` in onDestroy
5. Replace all `currentFileMtime` references with `fileWatcher.getFileMtime()`
6. Replace all `startWatching` / `stopWatching` calls with `fileWatcher.startWatching()` / `fileWatcher.stopWatching()`

**Risk:** Low. File watching is self-contained. The `handleFileChanged` callback references `filePath`, `mode`, `tabCache`, `previewPane`, and `loadFile` — all passed as DI.

**Estimated reduction:** ~35 lines

---

### Phase 4: Extract `loadFile` Sub-Loaders

**Files:**
- Create: `src/lib/utils/file-loaders.ts`
- Modify: `src/lib/components/PreviewEditor.svelte`

**Steps:**
1. Define `LoadContext` and `LoadResult` interfaces
2. Extract `loadArchiveDirectory` (~35 lines) — reads archive directory entries, returns HTML
3. Extract `loadArchiveFile` (~80 lines) — reads archive file bytes, detects binary/text
4. Extract `loadDirectory` (~10 lines) — invokes `read_directory`
5. Extract `loadImage` (~35 lines) — reads image with thumbnail support
6. Extract `loadPdf` (~20 lines) — invokes `get_pdf_info`, calls `fetchPdfOutline` (delegates to `pdfState.loadPdfInfo`)
7. Extract `loadArchive` (~8 lines) — renders archive preview
8. Extract `loadVideo` (~15 lines) — invokes `get_video_thumbnail`
9. Extract `loadTextOrBinary` (~65 lines) — reads text/binary with partial support
10. Refactor `loadFile` to be an orchestrator: cache check → reset → route → apply result
11. Create `applyLoadResult(result)` to write `LoadResult` fields to `$state`
12. Create `applyCacheResult(cached)` for cache hit path
13. Create `resetForNewLoad()` to clear state before fresh load

**Risk:** Medium. Each loader mutates different `$state` variables. The `LoadResult` interface must capture all possible outputs. The `applyLoadResult` function must handle the `editorInitInFlight` flag for direct-edit files. Generation checking (`gen !== loadGeneration`) must be handled in each loader or in the orchestrator after each await.

**Key design decision:** Loaders return results instead of mutating state. PreviewEditor's `applyLoadResult` does the mutations. This keeps loaders testable and PreviewEditor as the single source of truth for `$state`.

**Estimated reduction:** ~250 lines (loadFile body → file-loaders.ts)

---

### Phase 5: Cleanup

1. Review all exported functions — some may now delegate entirely to composables
2. Remove dead code paths (e.g., PDF-related conditionals that now live in `pdfState`)
3. Final line count check
4. Verify tab cache round-trip works with composable snapshots

---

## 7. Cross-Composable Dependencies

After extraction, PreviewEditor orchestrates 3 composables + file-loaders:

```
PreviewEditor
├── pdfState = createPdfState({ getFilePath, onToast })
├── tocState = createMarkdownTocState({ getPanelElement })
├── fileWatcher = createFileWatcher({ getFilePath, getMode, getRenderTabId, onFileChanged })
│
├── loadFile() ──► file-loaders.ts helpers ──► applyLoadResult()
│   └── PDF branch: await pdfState.loadPdfInfo(path, gen)
│
├── cacheTabState() ──► pdfState.toCacheSnapshot() + tocState.toCacheSnapshot()
├── cache-hit path ──► pdfState.fromCacheSnapshot() + tocState.fromCacheSnapshot()
│
├── mode effect ──► reads tocState.getTocFocused(), calls pdfState.focusPanel()
├── handleKeydown ──► calls pdfState.togglePdfToc(), tocState focus
└── template ──► reads composable getters, binds refs
```

No circular dependencies. All composables receive state getters via DI (same pattern as `createDirectorySortFilter`).

---

## 8. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| `$effect` must stay in component | Low | Composables return values; effects in PreviewEditor watch them |
| PDF ref binding (`bind:this`) | Medium | Svelte 5 `bind:this` works on composable properties — test that `pdfState.pdfPreviewPanel` reactivity triggers template update |
| Tab cache snapshot completeness | Medium | Ensure `toCacheSnapshot()` captures all fields in `TabEditorCache`. Cross-check with `tab-cache.ts` interface |
| Generation check in sub-loaders | Medium | Each loader must check `gen` after every async call. Return a sentinel `{ aborted: true }` result if gen changed, or check in orchestrator |
| `loadFile` re-entry guard | Low | `loadGeneration` counter already handles this. Preserve in orchestrator |
| File watcher event timing | Low | `handleFileChanged` already guards against stale events via path comparison. Move guard to composable |

---

## 9. Expected Outcome

| Metric | Before | After |
|--------|--------|-------|
| PreviewEditor.svelte | 1518 lines | ~700 lines |
| New composables | 0 | 3 (~180 lines total) |
| file-loaders.ts | 0 | 1 (~280 lines) |
| `$state` in PreviewEditor | 37 | ~15 (core only) |
| Functions in PreviewEditor | 41 | ~25 (core + orchestrator) |
| `loadFile` length | ~350 lines | ~60 lines (orchestrator) |

---

## 10. Existing Patterns to Follow

The extraction follows the **factory function** pattern established by the DirectoryPanel composables:

- **`createDirectorySortFilter(opts)`** — DI via typed interface, `$state` inside closure, returns typed API
- **`createProjectTree(deps)`** — same pattern
- **`createDirectoryDialogs(deps)`** — same pattern

The new composables (`createPdfState`, `createMarkdownTocState`, `createFileWatcher`) follow this exact convention. Each is a factory function that takes a deps interface and returns a typed API object.

**Key difference from DirectoryPanel composables:** PreviewEditor composables are single-instance (not per-panel). The factory function pattern still applies — it just happens to be called once.

**file-loaders.ts** is a utility module (not a composable). It exports pure async functions that take a `LoadContext` and return a `LoadResult`. No `$state` runes. This follows the pattern of `src/lib/utils/file-loader.ts` (existing utility).