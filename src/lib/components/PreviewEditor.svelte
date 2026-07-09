<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { layout } from '$lib/stores/layout';
  import { PreviewRouter, isVideoFileExt } from '$lib/previewers';
  import type { VideoMeta, TocHeading } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import TocSidebar from './TocSidebar.svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, StateField, StateEffect } from '@codemirror/state';
  import { keymap, Decoration } from '@codemirror/view';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { search, SearchQuery, setSearchQuery, findNext, findPrevious, openSearchPanel, closeSearchPanel } from '@codemirror/search';
  import { vim, Vim, getCM } from '@replit/codemirror-vim';
  import { getLanguage } from '$lib/utils/language';
  import { createVimCommandHandler } from '$lib/utils/vim-commands';
  import { initClipboardBridge, type ClipboardBridge } from '$lib/utils/clipboard-bridge';

  // Independent StateField for :s live preview (nvim inccommand style)
  const triggerSMatchUpdate = StateEffect.define<void>();
  const clearSMatch = StateEffect.define<void>();
  const sMatchField = StateField.define({
    create() { return Decoration.none as any; },
    update(value, tr) {
      for (const e of tr.effects) {
        if (e.is(clearSMatch)) return Decoration.none as any;
        if (e.is(triggerSMatchUpdate)) {
          const cmd = overlayCmdBuf;
          const m = cmd.match(/^(['<,'>]*)([%]?)s(.)/);
          if (!m) return Decoration.none as any;
          const delim = m[3];
          const esc = delim.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          const re = new RegExp(`s${esc}([^${esc}]*)(?:${esc}([^${esc}]*))?(?:${esc}([ggiI]*))?`);
          const pm = cmd.match(re);
          if (!pm) return Decoration.none as any;
          const pattern = pm[1];
          const replacement = pm[2] ?? '';
          const flags = pm[3] ?? '';
          const global = flags.includes('g');
          if (!pattern) return Decoration.none as any;
          let regex: RegExp;
          try {
            regex = new RegExp(pattern, flags.replace('g', '') + 'gi');
          } catch { return Decoration.none as any; }
          const hasRange = m[1] !== '' || m[2] !== '';
          const mark = Decoration.mark({ class: replacement ? 'cm-sMatch-replace' : 'cm-sMatch' });
          const decos: any[] = [];
          const doc = tr.state.doc;
          const startLine = hasRange ? 1 : doc.lineAt(tr.state.selection.main.head).number;
          const endLine = hasRange ? doc.lines : startLine;
          for (let i = startLine; i <= endLine; i++) {
            const line = doc.line(i);
            if (global) {
              let m: RegExpExecArray | null;
              while ((m = regex.exec(line.text)) !== null) {
                decos.push(mark.range(line.from + m.index, line.from + m.index + m[0].length));
                if (!m[0].length) break;
              }
            } else {
              const m = regex.exec(line.text);
              if (m) decos.push(mark.range(line.from + m.index, line.from + m.index + m[0].length));
            }
          }
          return Decoration.set(decos.sort((a, b) => a.from - b.from));
        }
      }
      return value.map(tr.changes);
    },
    provide: f => EditorView.decorations.from(f),
  });

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size: number | null;
  }

  let {
    filePath = null,
    currentTabId = 0,
    onFullscreen = () => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onToast = (message: string) => {},
    onTabCommand = (cmd: string) => {},
    onToggleLayout = () => {},
    batchRenameTempPath = null as string | null,
    onBatchRenameSave = (_content: string) => {},
    onBatchRenameCancel = () => {},
  }: {
    filePath: string | null;
    currentTabId?: number;
    onFullscreen?: () => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onToast?: (message: string) => void;
    onTabCommand?: (cmd: string) => void;
    onToggleLayout?: () => void;
    batchRenameTempPath?: string | null;
    onBatchRenameSave?: (content: string) => void;
    onBatchRenameCancel?: () => void;
  } = $props();

  let content: string = $state('');
  let savedContent: string = $state('');
  let binaryContent: ArrayBuffer | null = $state(null);
  let thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null = $state(null);
  let videoMeta: VideoMeta | null = $state(null);
  let originalFileSize: number = $state(0);
  let isModified: boolean = $state(false);
  let mode: 'global-normal' | 'editor-normal' | 'editor-insert' = $state('global-normal');
  let previewContainer: HTMLElement | undefined = $state(undefined);
  let editorContainer: HTMLElement | undefined = $state(undefined);
  let panelElement: HTMLElement | undefined = $state(undefined);
  let previewRouter: PreviewRouter | undefined;
  let directoryPreviewer: DirectoryPreviewer | undefined;
  let editorView: EditorView | undefined;
  let overlayElement: HTMLElement | undefined = $state(undefined);
  let renderRequestId: number = 0;
  let loadGeneration: number = 0;
  // t prefix state for tab operations
  let waitingForTabKey: boolean = false;

  // TOC state
  let tocHeadings: TocHeading[] = $state([]);
  let tocActiveLine: number = $state(-1);
  let tocSidebar: TocSidebar | undefined = $state(undefined);
  let tocFocused: boolean = $state(false);
  let tocOpen: boolean = $state(true);
  let pendingTocExpanded: Set<number> | null = null;

  function collectExpandedLines(headings: TocHeading[]): number[] {
    const lines: number[] = [];
    function walk(items: TocHeading[]) {
      for (const h of items) {
        if (h.expanded && h.children.length > 0) {
          lines.push(h.line);
          walk(h.children);
        }
      }
    }
    walk(headings);
    return lines;
  }

  function restoreExpandedLines(headings: TocHeading[], lines: Set<number>) {
    function walk(items: TocHeading[]) {
      for (const h of items) {
        if (lines.has(h.line) && h.children.length > 0) {
          h.expanded = true;
          walk(h.children);
        }
      }
    }
    walk(headings);
  }
  let scrollObserver: IntersectionObserver | undefined;
  let isMarkdown: boolean = $state(false);
  let editorTargetLine: number = -1;

  // Per-tab editor state cache
  interface TabEditorCache {
    filePath: string;
    content: string;
    savedContent: string;
    binaryContent: ArrayBuffer | null;
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    previewScrollTop: number;
    isModified: boolean;
    pdfCurrentPage: number;
    pdfPageCount: number;
    fileMtime: number;
    tocOpen: boolean;
    tocExpandedLines: number[];
    tocFocused: boolean;
    tocSelectedIndex: number;
  }
  const tabEditorCache = new Map<number, TabEditorCache>();
  let pendingRestoreScrollTop: number = -1;
  let pendingTocSelectedIndex: number = -1;
  let currentFileMtime: number = 0;
  let lastRenderedMtime: number = 0;
  let fileChangedUnlisten: (() => void) | null = null;

  // Per-filePath preview DOM cache (LRU, max 5 entries)
  interface CachedPreviewDom {
    dom: Node;
    scrollTop: number;
    tocHeadings: TocHeading[];
    tocExpandedLines: number[];
    fileMtime: number;
    lastAccess: number;
    /** Content snapshot for incremental previewers — when present,
     *  restore uses the snapshot to initialise prevLines state. */
    content?: string;
  }
  const previewDomCache = new Map<string, CachedPreviewDom>();
  const MAX_DOM_CACHE = 5;
  let lastRenderedPath: string = '';
  let isRendering: boolean = false;

  function normalizedPath(p: string): string {
    return p.replace(/\//g, '\\').toLowerCase();
  }

  function evictDomCache() {
    if (previewDomCache.size <= MAX_DOM_CACHE) return;
    let oldestKey = '';
    let oldestTime = Infinity;
    for (const [key, entry] of previewDomCache) {
      if (entry.lastAccess < oldestTime) {
        oldestTime = entry.lastAccess;
        oldestKey = key;
      }
    }
    previewDomCache.delete(oldestKey);
  }

  export function clearTabCache(tabId: number) {
    tabEditorCache.delete(tabId);
  }

  export function cacheTabState(tabId: number) {
    if (!filePath) return;
    console.log('[PreviewEditor] cacheTabState:', tabId, 'tocOpen:', tocOpen, 'mode:', mode);
    tabEditorCache.set(tabId, {
      filePath,
      content,
      savedContent,
      binaryContent,
      mode,
      previewScrollTop: previewContainer?.scrollTop ?? 0,
      isModified,
      pdfCurrentPage,
      pdfPageCount,
      fileMtime: currentFileMtime,
      tocOpen,
      tocExpandedLines: collectExpandedLines(tocHeadings),
      tocFocused,
      tocSelectedIndex: tocSidebar?.getSelectedIndex() ?? -1,
    });

    // Cache rendered DOM so tab switch back is instant.
    // Skip DOM cache for incremental previewers (text files handled by TextPreviewer) —
    // they re-render fast via codeToTokens and need prevLines state to be correct.
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const skipDomCache = !isMarkdown && ext !== 'json' && !isImageFile(filePath)
      && !isPdfFile(filePath) && !isArchiveFile(filePath) && !isVideoFile(filePath);
    console.log('[cacheTabState]', filePath.split(/[/\\]/).pop(), 'isMarkdown:', isMarkdown, 'ext:', ext, 'skipDomCache:', skipDomCache, 'mode:', mode);
    if (!skipDomCache && mode === 'global-normal' && currentFileMtime > 0 && previewContainer?.firstChild && (content || binaryContent)) {
      const normPath = normalizedPath(filePath);
      // Avoid double-caching if already in cache with same mtime
      if (previewDomCache.get(normPath)?.fileMtime !== currentFileMtime) {
        const savedScrollTop = previewContainer.scrollTop;
        const dom = previewContainer.removeChild(previewContainer.firstChild);
        evictDomCache();
        previewDomCache.set(normPath, {
          dom,
          scrollTop: savedScrollTop,
          tocHeadings: [...tocHeadings],
          tocExpandedLines: collectExpandedLines(tocHeadings),
          fileMtime: currentFileMtime,
          lastAccess: Date.now(),
        });
      }
    }
  }

  export function getEditorStateSnapshot(): {
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    previewScrollTop: number;
    isModified: boolean;
    pdfCurrentPage: number;
    tocOpen: boolean;
    tocExpandedLines: number[];
  } {
    const s = {
      mode,
      previewScrollTop: previewContainer?.scrollTop ?? 0,
      isModified,
      pdfCurrentPage,
      tocOpen,
      tocExpandedLines: collectExpandedLines(tocHeadings),
    };
    return s;
  }

  // File watching for real-time preview updates
  function startWatching(path: string) {
    stopWatching();
    console.log('[PreviewEditor] start_watch_file:', path);
    invoke('start_watch_file', { path }).catch(e => console.error('[PreviewEditor] start_watch_file error:', e));
  }

  function stopWatching() {
    invoke('stop_watch_file').catch(() => {});
  }

  async function handleFileChanged(eventPath: string) {
    console.log('[PreviewEditor] handleFileChanged:', eventPath, 'current:', filePath);
    if (!filePath) return;
    const a = eventPath.replace(/\//g, '\\').toLowerCase();
    const b = filePath.replace(/\//g, '\\').toLowerCase();
    if (a !== b) return;
    // Don't reload if we're in editor mode — we triggered this change ourselves
    // (e.g. via :w). Reload would reset mode to global-normal and discard editor state.
    if (mode !== 'global-normal') return;
    console.log('[PreviewEditor] Reloading due to external change');
    tabEditorCache.delete(currentTabId);
    previewDomCache.delete(a); // clear stale DOM cache
    await loadFile(filePath);
  }

  // Load file when filePath changes
  $effect(() => {
    if (filePath) {
      loadFile(filePath);
    }
  });

  onDestroy(() => {
    previewRouter?.dispose();
    if (editorView) {
      editorView.destroy();
    }
    scrollObserver?.disconnect();
    stopWatching();
    if (fileChangedUnlisten) {
      fileChangedUnlisten();
      fileChangedUnlisten = null;
    }
  });

  // Set up file-changed listener for real-time preview updates
  listen('file-changed', (event: any) => {
    console.log('[PreviewEditor] file-changed event:', event.payload);
    const changedPath = typeof event.payload === 'string' ? event.payload : String(event.payload ?? '');
    handleFileChanged(changedPath);
  }).then(unlisten => {
    fileChangedUnlisten = unlisten;
    console.log('[PreviewEditor] file-changed listener registered');
  });

  // Handle mode transitions (display toggling + focus)
  let prevMode: string = mode;
  $effect(() => {
    const m = mode;
    const changed = m !== prevMode;
    console.log('[mode-effect] mode:', m, 'changed:', changed, 'tocFocused:', tocFocused, 'scrollTop:', previewContainer?.scrollTop);
    prevMode = m;
    if (m === 'editor-normal' || m === 'editor-insert') {
      // Show editor, hide preview
      if (editorContainer) editorContainer.style.display = 'block';
      if (previewContainer) previewContainer.style.display = 'none';
      // Initialize editor if needed
      if (!editorView && editorContainer && filePath) {
        initEditor();
      } else if (editorView && editorTargetLine >= 0 && changed) {
        moveCursorToLine(editorTargetLine);
      }
      // Focus overlay in normal mode (IME won't activate on it),
      // or CodeMirror editor in insert mode.
      if (m === 'editor-normal') {
        if (overlayElement) overlayElement.focus();
      } else if (editorView) {
        editorView.focus();
      }
    } else {
      // global-normal: show preview, hide editor
      if (previewContainer) previewContainer.style.display = 'block';
      if (editorContainer) editorContainer.style.display = 'none';
      // Close search panel when leaving editor mode
      if (editorView) closeSearchPanel(editorView);
      // Return focus to the panel (only on mode transition, not initial mount)
      // Skip if TOC was focused before the switch (focus will be restored by renderPreview)
      if (changed && panelElement && !tocFocused) {
        panelElement.focus();
      }
    }
  });

  // Render preview when content is ready and in preview mode
  $effect(() => {
    // These reads register as reactive dependencies
    console.log('[render-effect] mode:', mode, 'filePath:', filePath?.split(/[/\\]/).pop(), 'hasContent:', !!(content || binaryContent));
    if (mode !== 'global-normal') return;
    if (!previewContainer || !filePath) return;

    if (content || binaryContent) {
      renderPreview();
    }
  });

  // Sync mode to layout store
  export function getMode(): string {
    return mode;
  }

  export function getIsModified(): boolean {
    return isModified;
  }

  // Focus forwarding: when panelElement gets focus, route to the correct inner element
  function handlePanelFocus() {
    console.log('[handlePanelFocus] mode:', mode, 'scrollTop:', previewContainer?.scrollTop, 'trigger:', document.activeElement?.className);
    if (mode === 'editor-normal' && overlayElement) {
      overlayElement.focus();
    } else if (mode === 'editor-insert' && editorView) {
      editorView.focus();
    }
    // global-normal: keep focus on panelElement for j/k preview scrolling
  }

  // Enter vim normal mode (used by batch rename)
  export function enterEditorMode() {
    mode = 'editor-normal';
    setTimeout(() => {
      if (overlayElement) overlayElement.focus();
    }, 50);
  }

  // Tab: indent line when after list marker, insert spaces otherwise (called from global Tab handler)
  export function pressTab() {
    if (!editorView || mode !== 'editor-insert') return;
    const { state } = editorView;
    const { from, to } = state.selection.main;
    const line = state.doc.lineAt(from);
    const col = from - line.from;
    // Match list markers: "- ", "* ", "+ ", "1. ", "- [ ] ", etc.
    const markerMatch = line.text.match(/^(\s*(?:[-*+]|\d+\.)\s(?:\[[ x]\]\s)?)/);

    if (markerMatch && col <= markerMatch[1].length || !state.selection.main.empty) {
      // After list marker or has selection: indent line(s)
      const lineFrom = state.doc.lineAt(from);
      const lineTo = state.doc.lineAt(to);
      const changes = [];
      for (let i = lineFrom.number; i <= lineTo.number; i++) {
        changes.push({ from: state.doc.line(i).from, insert: '    ' });
      }
      editorView.dispatch({ changes });
    } else {
      // In content: insert 4 spaces at cursor
      editorView.dispatch(state.replaceSelection('    '));
    }
  }


  // Shift+Tab: un-indent line or remove list marker (called from global Tab handler)
  export function pressShiftTab() {

    if (!editorView || mode !== 'editor-insert') return;
    const { state } = editorView;
    const line = state.doc.lineAt(state.selection.main.from);
    const indentMatch = line.text.match(/^(\s{1,4})/);
    if (indentMatch) {
      // Remove up to 4 leading spaces
      editorView.dispatch({ changes: { from: line.from, to: line.from + indentMatch[1].length } });
    } else {
      // Remove list marker: "- ", "* ", "1. ", "- [ ] ", etc.
      const markerMatch = line.text.match(/^((?:[-*+]|\d+\.)\s(?:\[[ x]\]\s)?)/);
      if (markerMatch) {
        editorView.dispatch({ changes: { from: line.from, to: line.from + markerMatch[1].length } });
      }
    }
  }

  function getPreviewRouter(): PreviewRouter {
    if (!previewRouter) {
      previewRouter = new PreviewRouter();
      previewRouter.onHeadings = (headings: TocHeading[]) => {
        tocHeadings = headings;
        tocActiveLine = -1;
      };
    }
    return previewRouter;
  }

  function getDirectoryPreviewer(): DirectoryPreviewer {
    if (!directoryPreviewer) {
      directoryPreviewer = new DirectoryPreviewer();
    }
    return directoryPreviewer;
  }

  // TOC: set up IntersectionObserver for scroll sync
  function setupScrollObserver() {
    scrollObserver?.disconnect();
    if (!previewContainer || !isMarkdown) return;

    const headings = previewContainer.querySelectorAll('h1, h2, h3, h4, h5, h6');
    if (headings.length === 0) return;

    // Add ids to headings for reference
    headings.forEach((el, i) => {
      if (!el.id) el.id = `heading-${i}`;
    });

    scrollObserver = new IntersectionObserver(
      (entries) => {
        // Find the topmost visible heading
        let topEntry: IntersectionObserverEntry | null = null;
        for (const entry of entries) {
          if (entry.isIntersecting) {
            if (!topEntry || entry.boundingClientRect.top < topEntry.boundingClientRect.top) {
              topEntry = entry;
            }
          }
        }
        if (topEntry) {
          const line = parseInt((topEntry.target as HTMLElement).dataset.line || '-1');
          if (line >= 0) {
            tocActiveLine = line;
          }
        }
      },
      { root: previewContainer, rootMargin: '-10% 0px -80% 0px', threshold: 0 }
    );

    headings.forEach((el) => {
      scrollObserver!.observe(el);
    });
  }

  // TOC: jump to heading by scrolling preview
  function handleTocJump(line: number) {
    if (!previewContainer) return;
    const heading = previewContainer.querySelector(`[data-line="${line}"]`);
    if (heading) {
      heading.scrollIntoView({ behavior: 'smooth', block: 'start' });
      tocActiveLine = line;
    }
    // Keep focus in TOC
  }

  // TOC: check if TOC should be visible
  export function isTocVisible(): boolean {
    return isMarkdown && tocHeadings.length > 0 && mode === 'global-normal' && tocOpen;
  }

  // TOC: focus management
  export function focusToc() {
    tocFocused = true;
    tocSidebar?.focus();
  }

  export function focusContent() {
    console.log('[focusContent] scrollTop before:', previewContainer?.scrollTop);
    tocFocused = false;
    if (panelElement) panelElement.focus({ preventScroll: true });
  }

  export function isTocFocused(): boolean {
    return tocFocused;
  }

  function handleTocFocusChange(focused: boolean) {
    tocFocused = focused;
    if (!focused && panelElement) {
      // Ctrl+W h from TOC: return focus to preview content
      panelElement.focus();
    }
  }

  function isTextFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    // Known binary formats that should NOT be treated as text
    const binaryExtensions = new Set([
      // Executables & libraries
      'exe', 'dll', 'so', 'dylib', 'bin', 'obj', 'o', 'a', 'lib', 'sys', 'drv',
      // Archives (zip handled separately)
      'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'zst', 'lz4', 'cab',
      // Media - audio
      'mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a', 'opus', 'mid', 'midi',
      // Media - video
      'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', 'ts',
      // Media - image (binary ones, SVG is text)
      'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'psd', 'raw', 'cr2', 'nef',
      // Documents
      'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp',
      // Fonts
      'ttf', 'otf', 'woff', 'woff2', 'eot',
      // Databases & compiled
      'db', 'sqlite', 'sqlite3', 'mdb', 'accdb', 'class', 'pyc', 'pyo',
      // Other binary
      'iso', 'img', 'vhd', 'vhdx', 'qcow2',
    ]);
    return !binaryExtensions.has(ext);
  }

  const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'svg']);

  function isImageFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return IMAGE_EXTENSIONS.has(ext);
  }

  function isPdfFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return ext === 'pdf';
  }

  function isVideoFile(path: string): boolean {
    return isVideoFileExt(path);
  }

  const ARCHIVE_EXTENSIONS = new Set(['zip']);

  function isArchiveFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return ARCHIVE_EXTENSIONS.has(ext);
  }

  // PDF state
  let pdfPageCount: number = $state(0);
  let pdfCurrentPage: number = $state(0);
  let pdfFileSize: number = $state(0);
  let pdfTitle: string | null = $state(null);
  let pdfRenderScale: number = 1.5;

  async function loadFile(path: string) {
    const t0 = performance.now();
    const gen = ++loadGeneration;
    const fileName = path.split(/[/\\]/).pop() || path;

    // Check per-tab cache during tab switch
    const cached = tabEditorCache.get(currentTabId);
    if (cached && cached.filePath === path) {
      if (gen !== loadGeneration) return;
      // Restore from cache immediately (no async gap = no flicker)
      content = cached.content;
      savedContent = cached.savedContent;
      binaryContent = cached.binaryContent;
      isModified = cached.isModified;
      pdfCurrentPage = cached.pdfCurrentPage;
      pdfPageCount = cached.pdfPageCount;
      currentFileMtime = cached.fileMtime;
      tocOpen = cached.tocOpen;
      tocFocused = cached.tocFocused;
      pendingTocSelectedIndex = cached.tocSelectedIndex;
      pendingTocExpanded = new Set(cached.tocExpandedLines);
      pendingRestoreScrollTop = cached.previewScrollTop;
      const ext = path.split('.').pop()?.toLowerCase() || '';
      isMarkdown = ext === 'md' || ext === 'markdown';
      console.log(`[load] ${fileName} tab-cache-hit ${(performance.now() - t0).toFixed(0)}ms`);
      if (!isMarkdown) {
        tocHeadings = [];
        tocActiveLine = -1;
      }
      if (cached.mode !== 'global-normal') {
        if (editorView) {
          editorView.destroy();
          editorView = undefined;
        }
      }
      mode = cached.mode;
      if (!content && !binaryContent && mode === 'global-normal') {
        renderPreview();
      }
      startWatching(path);
      // Validate mtime in background — if file changed on disk, reload
      invoke<{ size: number; modified: number }>('get_file_metadata', { path })
        .then(meta => {
          if (meta.modified !== cached.fileMtime) {
            tabEditorCache.delete(currentTabId);
            loadFile(path);
          }
        })
        .catch(() => {});
      return;
    }

    pendingRestoreScrollTop = -1;
    isModified = false;

    // Track markdown state for TOC
    const ext = path.split('.').pop()?.toLowerCase() || '';
    isMarkdown = ext === 'md' || ext === 'markdown';
    if (!isMarkdown) {
      tocHeadings = [];
      tocActiveLine = -1;
    }

    // Directory: list contents
    try {
      await invoke<FileEntry[]>('read_directory', { path });
      if (gen !== loadGeneration) return;
      content = '';
      binaryContent = null;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
      mode = 'global-normal';
      renderDirectoryPreview();
      stopWatching();
      return;
    } catch {
      // Not a directory, continue
    }

    // Binary image files (SVG is text, handled below)
    if (isImageFile(path) && !path.toLowerCase().endsWith('.svg')) {
      const fileName = path.split(/[/\\]/).pop() || path;
      const isGif = path.toLowerCase().endsWith('.gif');
      try {
        if (isGif) {
          // GIF: load original to preserve animation
          const t0 = performance.now();
          const base64 = await invoke<string>('read_binary_file', { path });
          console.log(`[image] ${fileName} gif loaded in ${(performance.now() - t0).toFixed(0)}ms, raw ${base64.length} chars`);
          if (gen !== loadGeneration) return;
          const binary = atob(base64);
          const bytes = new Uint8Array(binary.length);
          for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
          }
          binaryContent = bytes.buffer;
          thumbnailMeta = null;
        } else {
          // Other images: use thumbnail for large, binary for small
          const t0 = performance.now();
          const result = await invoke<{ data: string; width: number; height: number; original_size: number; is_thumbnail: boolean }>('read_image_thumbnail', { path });
          const loadMs = (performance.now() - t0).toFixed(0);
          if (gen !== loadGeneration) return;
          if (result.data) {
            // Large image: use compressed thumbnail
            console.log(`[image] ${fileName} thumbnail loaded in ${loadMs}ms, original ${(result.original_size / 1024 / 1024).toFixed(1)}MB → ${result.width}x${result.height}, data ${result.data.length} chars`);
            const binary = atob(result.data);
            const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {
              bytes[i] = binary.charCodeAt(i);
            }
            binaryContent = bytes.buffer;
            thumbnailMeta = {
              width: result.width,
              height: result.height,
              originalSize: result.original_size,
              isThumbnail: result.is_thumbnail,
            };
          } else {
            // Small image: load original directly
            const t1 = performance.now();
            const base64 = await invoke<string>('read_binary_file', { path });
            console.log(`[image] ${fileName} original loaded in ${(performance.now() - t1).toFixed(0)}ms, ${result.width}x${result.height}, data ${base64.length} chars`);
            if (gen !== loadGeneration) return;
            const binary = atob(base64);
            const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {
              bytes[i] = binary.charCodeAt(i);
            }
            binaryContent = bytes.buffer;
            thumbnailMeta = {
              width: result.width,
              height: result.height,
              originalSize: result.original_size,
              isThumbnail: false,
            };
          }
        }
        content = '[Binary Image]';
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load image:', error);
        binaryContent = null;
        thumbnailMeta = null;
        content = '';
      }
      mode = 'global-normal';
      return;
    }

    // PDF files
    if (isPdfFile(path)) {
      const fileName = path.split(/[/\\]/).pop() || path;
      try {
        const t0 = performance.now();
        const info = await invoke<{ page_count: number; title: string | null; author: string | null; file_size: number }>('get_pdf_info', { path });
        console.log(`[pdf] ${fileName} info loaded in ${(performance.now() - t0).toFixed(0)}ms, ${info.page_count} pages`);
        if (gen !== loadGeneration) return;

        pdfPageCount = info.page_count;
        pdfCurrentPage = 0;
        pdfFileSize = info.file_size;
        pdfTitle = info.title;

        // Render first page
        await loadPdfPage(path, 0, gen);
        if (gen !== loadGeneration) return;

        content = '[PDF]';
        mode = 'global-normal';
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load PDF:', error);
        pdfPageCount = 0;
        pdfCurrentPage = 0;
        content = '';
        binaryContent = null;
      }
      return;
    }

    if (isArchiveFile(path)) {
      content = '';
      binaryContent = null;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
      mode = 'global-normal';
      renderArchivePreview(path);
      return;
    }

    // Video files: get thumbnail via ffmpeg
    if (isVideoFile(path)) {
      const fileName = path.split(/[/\\]/).pop() || path;
      try {
        const t0 = performance.now();
        const result = await invoke<VideoMeta>('get_video_thumbnail', { path });
        console.log(`[video] ${fileName} thumbnail loaded in ${(performance.now() - t0).toFixed(0)}ms, ${result.width}x${result.height}, duration=${result.duration_seconds}s`);
        if (gen !== loadGeneration) return;
        videoMeta = result;
        binaryContent = null;
        content = JSON.stringify(result);
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load video thumbnail:', error);
        videoMeta = null;
        binaryContent = null;
        content = String(error);
      }
      mode = 'global-normal';
      return;
    }

    // Get file metadata (size + mtime) before reading content.
    // currentFileMtime must be set before content triggers the $effect,
    // otherwise renderPreview won't cache the DOM.
    const MAX_PREVIEW_SIZE = 200 * 1024; // 200KB
    originalFileSize = 0;
    try {
      const meta = await invoke<{ size: number; modified: number }>('get_file_metadata', { path });
      if (gen !== loadGeneration) return;
      originalFileSize = meta.size;
      currentFileMtime = meta.modified;
    } catch {
      // Can't get metadata, try loading anyway
    }

    const usePartial = originalFileSize > MAX_PREVIEW_SIZE;

    // Try reading as text first
    try {
      const newContent = usePartial
        ? await invoke<string>('read_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
        : await invoke<string>('read_file', { path });
      if (gen !== loadGeneration) return;
      content = newContent;
      binaryContent = null;
      savedContent = newContent;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
    } catch {
      if (gen !== loadGeneration) return;
      // Text read failed — likely binary, read raw bytes for hex dump
      try {
        const base64 = usePartial
          ? await invoke<string>('read_binary_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
          : await invoke<string>('read_binary_file', { path });
        if (gen !== loadGeneration) return;
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
          bytes[i] = binary.charCodeAt(i);
        }
        content = '';
        binaryContent = bytes.buffer;
      } catch {
        if (gen !== loadGeneration) return;
        content = '';
        binaryContent = null;
      }
    }
    mode = 'global-normal';
    startWatching(path);
    console.log(`[load] ${fileName} fresh ${(performance.now() - t0).toFixed(0)}ms (${(content?.length || (binaryContent as ArrayBuffer | null)?.byteLength || 0)} bytes)`);
    // Empty files: content is '' (falsy), effect won't trigger, render directly
    if (!content && !binaryContent) renderPreview();
  }

  async function renderPreview() {
    if (!previewContainer || !filePath) return;
    if (isRendering) { console.log(`[render] ${filePath.split(/[/\\]/).pop()} skipped (concurrent)`); return; }
    isRendering = true;
    const tRender = performance.now();
    try {

    // Cache previous file's DOM when switching to a different file.
    // lastRenderedPath must be set (skip initial load where it's empty).
    // Skip DOM cache for incremental previewers (text files) — same reasoning as cacheTabState.
    const prevExt = lastRenderedPath.split('.').pop()?.toLowerCase() || '';
    const skipPrevDomCache = prevExt !== 'md' && prevExt !== 'markdown' && prevExt !== 'json'
      && !isImageFile(lastRenderedPath) && !isPdfFile(lastRenderedPath)
      && !isArchiveFile(lastRenderedPath) && !isVideoFile(lastRenderedPath);
    if (!skipPrevDomCache && filePath !== lastRenderedPath && lastRenderedPath && currentFileMtime > 0 && previewContainer.firstChild && (content || binaryContent)) {
      const prevNormPath = normalizedPath(lastRenderedPath);
      const savedScrollTop = previewContainer.scrollTop;
      const dom = previewContainer.firstChild;
      previewContainer.removeChild(dom);
      evictDomCache();
      previewDomCache.set(prevNormPath, {
        dom,
        scrollTop: savedScrollTop,
        tocHeadings: [...tocHeadings],
        tocExpandedLines: collectExpandedLines(tocHeadings),
        fileMtime: lastRenderedMtime,
        lastAccess: Date.now(),
      });
    }

    // Check DOM cache for instant restore (only if mtime is known)
    const normPath = normalizedPath(filePath);
    const cachedDom = previewDomCache.get(normPath);
    console.log('[renderPreview] cache-check:', filePath.split(/[/\\]/).pop(), 'cached:', !!cachedDom,
      'mtime:', currentFileMtime, 'cachedMtime:', cachedDom?.fileMtime,
      'match:', !!(cachedDom && currentFileMtime > 0 && cachedDom.fileMtime === currentFileMtime));
    if (cachedDom && currentFileMtime > 0 && cachedDom.fileMtime === currentFileMtime) {
      // If preview already has content (spurious re-render from focus switch
      // etc.), skip re-attach — the current DOM and scrollTop are correct.
      if (previewContainer.firstChild) {
        console.log('[renderPreview] dom-cache-hit skipped (already has content)');
        return;
      }
      cachedDom.lastAccess = Date.now();
      previewContainer.innerHTML = '';
      previewContainer.appendChild(cachedDom.dom);
      lastRenderedPath = filePath;
      lastRenderedMtime = currentFileMtime;

      // Restore TOC state: TocSidebar was overwritten by the previous file's headings.
      // pendingTocExpanded (from tabEditorCache) takes priority over cachedDom.tocExpandedLines.
      const expandSet = pendingTocExpanded ?? new Set(cachedDom.tocExpandedLines);
      if (cachedDom.tocHeadings.length > 0) {
        if (expandSet.size > 0) {
          restoreExpandedLines(cachedDom.tocHeadings, expandSet);
        }
        tocHeadings = cachedDom.tocHeadings;
      }
      pendingTocExpanded = null;

      console.log(`[render] ${filePath.split(/[/\\]/).pop() || filePath} dom-cache-hit ${(performance.now() - tRender).toFixed(0)}ms mtime:${cachedDom.fileMtime}`);

      // Defer layout-dependent operations to avoid sync reflow on huge DOM
      const savedScrollTop = cachedDom.scrollTop;
      const restoreTocFocus = tocFocused && tocOpen;
      const restoreTocIdx = pendingTocSelectedIndex;
      requestAnimationFrame(() => {
        previewContainer!.scrollTop = savedScrollTop;
        if (isMarkdown) setupScrollObserver();
        if (restoreTocFocus) {
          if (restoreTocIdx >= 0) {
            tocSidebar?.setSelectedTocIndex(restoreTocIdx);
            pendingTocSelectedIndex = -1;
          }
          tocSidebar?.focus();
        }
      });
      return;
    }

    // Remove stale cache entry for same file (e.g. content changed externally)
    previewDomCache.delete(normPath);

    const requestId = ++renderRequestId;
    // Pass thumbnail metadata via dataset
    if (thumbnailMeta) {
      previewContainer.dataset.thumbWidth = String(thumbnailMeta.width);
      previewContainer.dataset.thumbHeight = String(thumbnailMeta.height);
      previewContainer.dataset.thumbOriginalSize = String(thumbnailMeta.originalSize);
      previewContainer.dataset.thumbIsThumbnail = String(thumbnailMeta.isThumbnail);
    } else {
      delete previewContainer.dataset.thumbWidth;
      delete previewContainer.dataset.thumbHeight;
      delete previewContainer.dataset.thumbOriginalSize;
      delete previewContainer.dataset.thumbIsThumbnail;
    }
    // Pass original file size for truncation notice
    if (originalFileSize > 0) {
      previewContainer.dataset.originalFileSize = String(originalFileSize);
    } else {
      delete previewContainer.dataset.originalFileSize;
    }
    const previewContent: string | ArrayBuffer = binaryContent ?? content;
    await getPreviewRouter().preview(filePath, previewContent, previewContainer);
    if (requestId !== renderRequestId) return;

    // Apply pending scroll restoration from tab cache
    if (pendingRestoreScrollTop >= 0) {
      previewContainer.scrollTop = pendingRestoreScrollTop;
      pendingRestoreScrollTop = -1;
    }

    // Restore TOC heading expand state from cache
    if (pendingTocExpanded && tocHeadings.length > 0) {
      restoreExpandedLines(tocHeadings, pendingTocExpanded);
      pendingTocExpanded = null;
      tocHeadings = [...tocHeadings];
    }

    // Restore TOC focus and selection from cache (defer to let Svelte update DOM)
    if (tocFocused && tocOpen) {
      requestAnimationFrame(() => {
        if (pendingTocSelectedIndex >= 0) {
          tocSidebar?.setSelectedTocIndex(pendingTocSelectedIndex);
          pendingTocSelectedIndex = -1;
        }
        tocSidebar?.focus();
      });
    }

    // Add PDF info bar
    if (isPdfFile(filePath) && pdfPageCount > 0) {
      addPdfInfoBar(previewContainer);
    }

    // Set up scroll observer for markdown TOC sync
    if (isMarkdown) {
      setupScrollObserver();
    }

    lastRenderedPath = filePath;
    lastRenderedMtime = currentFileMtime;
    console.log(`[render] ${filePath.split(/[/\\]/).pop() || filePath} full ${(performance.now() - tRender).toFixed(0)}ms`);
    } finally {
      isRendering = false;
    }
  }

  function scrollPreview(deltaY: number, deltaX: number = 0) {
    if (previewContainer) {
      previewContainer.scrollBy({ top: deltaY, left: deltaX, behavior: 'auto' });
    }
  }

  export function getVisibleLine(): number {
    if (!previewContainer || !content) return 0;
    // Use data-line attributes if available (markdown with block-level annotations)
    const rect = previewContainer.getBoundingClientRect();
    const el = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
    if (el) {
      const lined = el.closest('[data-line]');
      if (lined) {
        const line = parseInt(lined.getAttribute('data-line')!);
        if (!isNaN(line)) return line;
      }
    }
    // Fallback: scroll ratio (code files with uniform line height)
    const maxScroll = previewContainer.scrollHeight - previewContainer.clientHeight;
    if (maxScroll <= 0) return 0;
    const ratio = previewContainer.scrollTop / maxScroll;
    const totalLines = content.split('\n').length;
    return Math.round(ratio * (totalLines - 1));
  }

  function getPosAtLine(text: string, lineNumber: number): number {
    let pos = 0;
    let line = 0;
    for (let i = 0; i < text.length && line < lineNumber; i++) {
      if (text[i] === '\n') line++;
      if (line < lineNumber) pos = i + 1;
    }
    return pos;
  }

  function moveCursorToLine(lineNumber: number) {
    if (!editorView) return;
    const targetLine = Math.min(lineNumber + 1, editorView.state.doc.lines);
    const pos = editorView.state.doc.line(targetLine).from;
    editorView.dispatch({ selection: { anchor: pos } });
    scrollEditorToPos(editorView, pos);
    editorTargetLine = -1;
  }

  function scrollEditorToPos(view: EditorView, pos: number) {
    view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const s = view.scrollDOM;
        s.scrollTop = Math.max(0, Math.min(s.scrollTop, s.scrollHeight - s.clientHeight));
      });
    });
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function addPdfInfoBar(container: HTMLElement) {
    // Remove existing info bar
    container.querySelector('.pdf-info-bar')?.remove();

    const bar = document.createElement('div');
    bar.className = 'pdf-info-bar image-info-bar';

    const info = document.createElement('span');
    info.textContent = `${pdfCurrentPage + 1}/${pdfPageCount}`;
    if (pdfFileSize > 0) {
      info.textContent += ` · ${formatSize(pdfFileSize)}`;
    }
    if (pdfTitle) {
      info.textContent += ` · ${pdfTitle}`;
    }
    bar.appendChild(info);

    const hints = document.createElement('span');
    hints.className = 'pdf-hints';
    hints.textContent = 'J/K:翻页 E:全屏';
    hints.style.color = '#666';
    hints.style.fontSize = '11px';
    bar.appendChild(hints);

    container.appendChild(bar);
  }

  async function renderDirectoryPreview() {
    if (!previewContainer || !filePath) return;
    const requestId = ++renderRequestId;
    const previewer = getDirectoryPreviewer();
    previewContainer.dataset.filePath = filePath;
    await previewer.render('', previewContainer);
    if (requestId !== renderRequestId) return;
  }

  async function renderArchivePreview(path: string) {
    if (!previewContainer) return;
    const requestId = ++renderRequestId;
    previewContainer.dataset.filePath = path;
    await getPreviewRouter().preview(path, '', previewContainer);
    if (requestId !== renderRequestId) return;
  }

  async function loadPdfPage(path: string, page: number, gen?: number) {
    const g = gen ?? loadGeneration;
    try {
      const t0 = performance.now();
      const result = await invoke<{ data: string; width: number; height: number }>('render_pdf_page', {
        path,
        page,
        scale: pdfRenderScale,
      });
      console.log(`[pdf] page ${page} rendered in ${(performance.now() - t0).toFixed(0)}ms, ${result.width}x${result.height}`);
      if (g !== loadGeneration) return;

      // Convert base64 to ArrayBuffer
      const binary = atob(result.data);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      binaryContent = bytes.buffer;
      pdfCurrentPage = page;
    } catch (error) {
      if (g !== loadGeneration) return;
      console.error(`Failed to render PDF page ${page}:`, error);
    }
  }

  function initEditor() {
    if (!editorContainer || !filePath) return;

    const language = getLanguage(filePath);
    const extensions = [
      basicSetup,
      search({ top: true }),
      sMatchField,
      EditorView.lineWrapping,
      keymap.of([{
        key: 'Tab',
        run: (view) => {
          const { state } = view;
          if (state.selection.main.empty) {
            view.dispatch(state.replaceSelection('    '));
          } else {
            // Indent selected lines
            const { from, to } = state.selection.main;
            const lineFrom = state.doc.lineAt(from);
            const lineTo = state.doc.lineAt(to);
            const changes = [];
            for (let i = lineFrom.number; i <= lineTo.number; i++) {
              changes.push({ from: state.doc.line(i).from, insert: '    ' });
            }
            view.dispatch({ changes });
          }
          return true;
        },
      }]),
      vim({ status: false }),
      createVimCommandHandler(
        () => ({
          save: async () => {
            if (batchRenameTempPath) {
              onBatchRenameSave(content);
            } else {
              await saveFile();
            }
          },
          quit: () => {
            if (batchRenameTempPath) {
              onBatchRenameCancel();
            } else {
              mode = 'global-normal';
            }
          },
          forceQuit: () => {
            if (batchRenameTempPath) {
              onBatchRenameCancel();
            } else {
              content = savedContent;
              isModified = false;
              mode = 'global-normal';
            }
          },
          isModified: () => isModified,
        }),
        (msg) => onToast(msg)
      ),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          content = update.state.doc.toString();
          isModified = content !== savedContent;
        }
        // Sync PreviewEditor mode with vim state.
        // When vim switches between insert/normal, update the overlay.
        const cm = (update.view as any).cm;
        const vimState = cm?.state?.vim;
        if (vimState) {
          if (vimState.insertMode && mode === 'editor-normal') {
            mode = 'editor-insert';
          } else if (!vimState.insertMode && !vimState.visualMode && mode === 'editor-insert') {
            mode = 'editor-normal';
          }
        }
      }),
    ];

    if (language) {
      extensions.push(language);
    }

    const state = EditorState.create({
      doc: content,
      extensions,
      selection: editorTargetLine >= 0 ? { anchor: getPosAtLine(content, editorTargetLine) } : undefined,
    });
    const needsScroll = editorTargetLine >= 0;
    editorTargetLine = -1;

    editorView = new EditorView({
      state,
      parent: editorContainer,
    });

    if (needsScroll && editorView) {
      scrollEditorToPos(editorView, editorView.state.selection.main.head);
    }

    // Bridge vim clipboard with system clipboard
    clipboardBridge = initClipboardBridge(editorView, overlayElement);

    // Intercept Enter on list lines before CodeMirror/markdown extension handles it
    editorView.contentDOM.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key !== 'Enter' || !editorView) return;
      const { state } = editorView;
      const line = state.doc.lineAt(state.selection.main.head);
      const markerMatch = line.text.match(/^(\s*(?:[-*+]|\d+\.)\s(?:\[[ x]\]\s)?)/);
      if (!markerMatch) return;

      e.preventDefault();
      e.stopPropagation();

      const marker = markerMatch[1];
      const contentAfter = line.text.slice(marker.length).trim();
      const pos = state.selection.main.head;

      if (!contentAfter) {
        const indentMatch = line.text.match(/^(\s{1,4})/);
        if (indentMatch) {
          // Has leading whitespace: remove one level, cursor after marker
          editorView.dispatch({
            changes: { from: line.from, to: line.from + indentMatch[1].length },
            selection: { anchor: line.from + marker.length - indentMatch[1].length },
          });
        } else {
          // No indentation: remove marker, cursor at line start
          editorView.dispatch({
            changes: { from: line.from, to: line.from + marker.length },
            selection: { anchor: line.from },
          });
        }
      } else {
        editorView.dispatch({
          changes: { from: pos, insert: '\n' + marker },
          selection: { anchor: pos + 1 + marker.length },
        });
      }
    }, true); // capture phase

    // Block IME composition on the overlay div (defense-in-depth).
    // Some Windows IME versions may still try to compose when they
    // detect contentEditable in the DOM tree, even if it's not focused.
    if (overlayElement) {
      overlayElement.addEventListener('compositionstart', (e: CompositionEvent) => {
        e.preventDefault();
        e.stopPropagation();
      }, true);
      overlayElement.addEventListener('compositionend', (e: CompositionEvent) => {
        e.preventDefault();
        e.stopPropagation();
      }, true);
    }
  }

  // Overlay keydown handler for editor normal mode.
  // The overlay is a plain <div tabindex=0> (NOT contentEditable),
  // so the Chinese IME won't activate on it. We capture physical keys
  // and forward them to the vim engine via the Vim API directly.
  function codeToVimKey(event: KeyboardEvent): string | null {
    const code = event.code;
    let key = '';
    if (event.ctrlKey) key += 'C-';
    if (event.altKey) key += 'A-';
    if (event.metaKey) key += 'M-';
    if (code.startsWith('Key') && code.length === 4) {
      key += event.shiftKey ? code[3] : code[3].toLowerCase();
    } else if (code === 'Enter') { key += 'Enter'; }
    else if (code === 'Space') { key += 'Space'; }
    else if (code === 'Escape') { key = 'Esc'; }
    else if (code === 'Backspace') { key += 'BS'; }
    else if (code === 'Tab') { key += 'Tab'; }
    else if (code === 'Delete') { key += 'Del'; }
    else if (code.startsWith('Digit')) { key += code[5]; }
    else if (code.startsWith('Arrow')) { key += code.slice(5); }
    else if (code === 'BracketLeft') { key += event.shiftKey ? '{' : '['; }
    else if (code === 'BracketRight') { key += event.shiftKey ? '}' : ']'; }
    else if (code === 'Semicolon') { key += event.shiftKey ? ':' : ';'; }
    else if (code === 'Quote') { key += event.shiftKey ? '"' : "'"; }
    else if (code === 'Comma') { key += event.shiftKey ? '<' : ','; }
    else if (code === 'Period') { key += event.shiftKey ? '>' : '.'; }
    else if (code === 'Slash') { key += event.shiftKey ? '?' : '/'; }
    else if (code === 'Backslash') { key += '\\'; }
    else if (code === 'Minus') { key += event.shiftKey ? '_' : '-'; }
    else if (code === 'Equal') { key += event.shiftKey ? '+' : '='; }
    else if (code === 'Backquote') { key += event.shiftKey ? '~' : '`'; }
    else { return null; }
    if (key.length > 1) key = '<' + key + '>';
    return key;
  }

  let overlayCmdBuf = $state('');
  let overlayCmdActive = $state(false);
  let clipboardBridge: ClipboardBridge | null = null;

  let searchActive: boolean = $state(false);
  let searchBuf: string = $state('');

  function executeSearch() {
    if (!editorView || !searchBuf) return;
    // Open search panel so the highlighter activates (panel is hidden by CSS)
    openSearchPanel(editorView);
    // Set our query
    const query = new SearchQuery({ search: searchBuf, caseSensitive: false });
    editorView.dispatch({ effects: setSearchQuery.of(query) });
    // Select first match
    findNext(editorView);
  }

  // Live-highlight :s matches as user types (nvim inccommand style)
  function highlightSMatches() {
    if (!editorView) return;
    // Parse :s command to extract replacement for CSS variable
    const cmd = overlayCmdBuf;
    const delimMatch = cmd.match(/^(['<,'>]*)([%]?)s(.)/);
    if (delimMatch) {
      const delim = delimMatch[3];
      const esc = delim.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const re = new RegExp(`s${esc}([^${esc}]*)(?:${esc}([^${esc}]*))?(?:${esc}([ggiI]*))?`);
      const pm = cmd.match(re);
      if (pm && pm[2] !== undefined) {
        editorView.dom.style.setProperty('--s-replacement', JSON.stringify(pm[2]));
      } else {
        editorView.dom.style.removeProperty('--s-replacement');
      }
    } else {
      editorView.dom.style.removeProperty('--s-replacement');
    }
    editorView.dispatch({ effects: triggerSMatchUpdate.of() });
  }

  function processOverlayCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (trimmed === 'w' || trimmed === 'write') {
      if (batchRenameTempPath) {
        onBatchRenameSave(content);
      } else {
        saveFile();
      }
    } else if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') {
      if (batchRenameTempPath) {
        onBatchRenameCancel();
      } else {
        content = savedContent;
        isModified = false;
        mode = 'global-normal';
      }
    } else if (trimmed === 'q' || trimmed === 'quit') {
      if (batchRenameTempPath) {
        onBatchRenameCancel();
      } else if (isModified) {
        onToast('E37: No write since last change (add ! to override)');
      } else {
        mode = 'global-normal';
      }
    } else if (trimmed === 'wq' || trimmed === 'x') {
      if (batchRenameTempPath) {
        onBatchRenameSave(content);
      } else {
        saveFile().then(() => { mode = 'global-normal'; });
      }
    } else if (trimmed === 'wqall' || trimmed === 'wqall!') {
      if (batchRenameTempPath) {
        onBatchRenameSave(content);
      } else {
        saveFile().then(() => { mode = 'global-normal'; });
      }
    } else if (editorView) {
      const cm = getCM(editorView);
      if (cm) {
        Vim.handleEx(cm as any, trimmed);
        // Clear :s highlights after ex command completes
        editorView.dispatch({ effects: clearSMatch.of() });
        editorView.dom.style.removeProperty('--s-replacement');
      }
    }
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    // Custom command mode (triggered by ':')
    if (overlayCmdActive) {
      if (event.key === 'Enter') {
        overlayCmdActive = false;
        processOverlayCommand(overlayCmdBuf);
        overlayCmdBuf = '';
        return;
      }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') {
        overlayCmdActive = false;
        overlayCmdBuf = '';
        onToast('');
        if (editorView) { editorView.dispatch({ effects: clearSMatch.of() }); editorView.dom.style.removeProperty('--s-replacement'); }
        return;
      }
      if (event.key === 'Backspace') {
        if (overlayCmdBuf.length > 0) {
          overlayCmdBuf = overlayCmdBuf.slice(0, -1);
        } else {
          overlayCmdActive = false;
        }
        highlightSMatches();
        return;
      }
      if (event.key.length === 1) {
        overlayCmdBuf += event.key;
        highlightSMatches();
      }
      return;
    }

    // Search mode (triggered by '/' or '?')
    if (searchActive) {
      if (event.key === 'Enter') {
        searchActive = false;
        executeSearch();
        return;
      }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') {
        searchActive = false;
        searchBuf = '';
        return;
      }
      if (event.key === 'Backspace') {
        if (searchBuf.length > 0) {
          searchBuf = searchBuf.slice(0, -1);
        } else {
          searchActive = false;
        }
        return;
      }
      if (event.key.length === 1) {
        searchBuf += event.key;
      }
      return;
    }

    if (event.key === ':') {
      overlayCmdActive = true;
      overlayCmdBuf = '';
      return;
    }

    if (event.key === '/' || event.key === '?') {
      searchActive = true;
      searchBuf = '';
      return;
    }

    // n/N: repeat last search
    if (event.code === 'KeyN' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      if (editorView) {
        if (event.shiftKey) {
          findPrevious(editorView);
        } else {
          findNext(editorView);
        }
      }
      return;
    }

    // Escape: close search panel if open
    if (event.key === 'Escape' && editorView) {
      closeSearchPanel(editorView);
    }

    // Normal vim key handling
    const vimKey = codeToVimKey(event);
    if (!vimKey) return;
    if (!editorView) return;

    // Inject system clipboard before put
    if ((vimKey === 'p' || vimKey === 'P') && clipboardBridge) {
      clipboardBridge.injectClipboard();
    }

    const cm = getCM(editorView);
    if (!cm) return;
    (Vim as any).multiSelectHandleKey?.(cm, vimKey, 'user');

    // If vim entered insert mode (e.g. user pressed i/a/o),
    // switch to editor-insert so the overlay hides.
    const vimState = cm.state?.vim;
    if (vimState?.insertMode) {
      mode = 'editor-insert';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Only handle keys in global-normal mode
    // editor-normal and editor-insert are handled by CodeMirror vim
    if (mode !== 'global-normal') return;

    // Ctrl+W l: switch focus from preview to TOC (when TOC is visible)
    if (event.ctrlKey && event.code === 'KeyL' && isMarkdown && tocHeadings.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      focusToc();
      return;
    }

    // t prefix for tab operations
    if (waitingForTabKey) {
      waitingForTabKey = false;
      layout.clearKeyPrefix();
      const code = event.code;
      const key = event.key;
      event.preventDefault();
      if (code === 'KeyT') {
        onTabCommand('new');
      } else if (code === 'KeyC') {
        onTabCommand('close');
      } else if (code === 'KeyR') {
        onTabCommand('rename-hint');
      } else if (code === 'KeyN' || code === 'BracketRight') {
        onTabCommand('next');
      } else if (code === 'KeyP' || code === 'BracketLeft') {
        onTabCommand('prev');
      } else if (code === 'Comma') {
        onTabCommand('swap-prev');
      } else if (code === 'Period') {
        onTabCommand('swap-next');
      } else if (key >= '1' && key <= '9') {
        onTabCommand('switch-' + key);
      }
      return;
    }

    if (event.code === 'KeyT' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      waitingForTabKey = true;
      layout.setKeyPrefix('t');
      setTimeout(() => { waitingForTabKey = false; layout.clearKeyPrefix(); }, 1000);
      return;
    }

    if (event.code === 'KeyE' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (!filePath || !isTextFile(filePath)) {
        onToast('此文件类型不支持编辑');
        return;
      }
      editorTargetLine = getVisibleLine();
      mode = 'editor-normal';
    } else if (event.code === 'KeyE' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (!filePath) {
        onToast('此文件类型不支持全屏查看');
        return;
      }
      onFullscreen();
    } else if (event.code === 'KeyJ' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      scrollPreview(40);
    } else if (event.code === 'KeyK' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      scrollPreview(-40);
    } else if (event.code === 'KeyH' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      scrollPreview(0, -40);
    } else if (event.code === 'KeyL' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      scrollPreview(0, 40);
    } else if (event.code === 'KeyJ' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      // PDF: next page
      if (filePath && isPdfFile(filePath) && pdfCurrentPage < pdfPageCount - 1) {
        event.preventDefault();
        loadPdfPage(filePath, pdfCurrentPage + 1);
      }
    } else if (event.code === 'KeyK' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      // PDF: previous page
      if (filePath && isPdfFile(filePath) && pdfCurrentPage > 0) {
        event.preventDefault();
        loadPdfPage(filePath, pdfCurrentPage - 1);
      }
    } else if (event.code === 'KeyG' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (previewContainer) previewContainer.scrollTop = 0;
    } else if (event.code === 'KeyG' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (previewContainer) previewContainer.scrollTop = previewContainer.scrollHeight;
    } else if (event.ctrlKey && event.code === 'KeyS') {
      event.preventDefault();
      saveFile();
    }
  }

  async function saveFile() {
    if (!filePath || !isModified) return;
    try {
      await invoke('write_file', { path: filePath, content });
      savedContent = content;
      isModified = false;
      // Clear cached DOM since file content changed
      previewDomCache.delete(normalizedPath(filePath));
      // Refresh preview so it's up-to-date when user goes back to global-normal
      if (mode === 'editor-normal' || mode === 'editor-insert') {
        // Preview will be refreshed when switching to global-normal via $effect
      }
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  }

  function getFileName(): string {
    if (!filePath) return '';
    return filePath.split('\\').pop() || filePath.split('/').pop() || '';
  }

  export function setContent(newContent: string) {
    content = newContent;
    savedContent = newContent;
    isModified = false;
  }

  export function getContent(): string {
    return content;
  }

  export function getFile(): string | null {
    return filePath;
  }

  export function getPdfInfo(): { currentPage: number; pageCount: number; filePath: string | null } {
    return { currentPage: pdfCurrentPage, pageCount: pdfPageCount, filePath };
  }

  export function setPdfPage(page: number) {
    if (filePath && isPdfFile(filePath) && page >= 0 && page < pdfPageCount) {
      loadPdfPage(filePath, page);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="preview-editor"
  bind:this={panelElement}
  onkeydown={handleKeydown}
  onfocus={handlePanelFocus}
  role="region"
  aria-label="Preview/Editor"
  tabindex="0"
>
  <div class="panel-header">
    <span class="panel-title">{batchRenameTempPath ? 'Batch Rename' : mode === 'global-normal' ? 'Preview' : 'Editor'}</span>
    {#if batchRenameTempPath}
      <span class="file-name">:w to rename, :q! to cancel</span>
    {:else if filePath}
      <span class="file-name">{getFileName()}</span>
      {#if isModified}
        <span class="modified-indicator">●</span>
      {/if}
    {/if}
    <span class="mode-indicator" class:insert={mode === 'editor-insert'} class:normal={mode === 'editor-normal'} class:toc={mode === 'global-normal' && tocFocused}>
      {#if mode === 'global-normal' && tocFocused}TOC{:else if mode === 'global-normal'}PREVIEW{:else if mode === 'editor-normal'}NORMAL{:else}INSERT{/if}
    </span>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <button
      class="layout-toggle"
      class:expanded={$layout.previewExpanded}
      onclick={onToggleLayout}
      title={$layout.previewExpanded ? 'Collapse to 3-column (Ctrl+W M)' : 'Expand to 2-column (Ctrl+W M)'}
    >
      {#if $layout.previewExpanded}
        <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
          <rect x="0" y="0" width="4" height="12" rx="1" fill="currentColor" opacity="0.3"/>
          <rect x="5" y="0" width="4" height="12" rx="1" fill="currentColor" opacity="0.6"/>
          <rect x="10" y="0" width="4" height="12" rx="1" fill="currentColor"/>
        </svg>
      {:else}
        <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
          <rect x="0" y="0" width="7" height="12" rx="1" fill="currentColor" opacity="0.3"/>
          <rect x="8" y="0" width="6" height="12" rx="1" fill="currentColor"/>
        </svg>
      {/if}
    </button>
    {#if isMarkdown && tocHeadings.length > 0 && mode === 'global-normal'}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <button
        class="toc-toggle"
        class:closed={!tocOpen}
        onclick={() => { tocOpen = !tocOpen; console.log('[PreviewEditor] TOC toggle ->', tocOpen); }}
        title={tocOpen ? 'Hide outline' : 'Show outline'}
      >
        ☰
      </button>
    {/if}
  </div>

  <div class="panel-content">
    {#if filePath}
      <div class="preview-with-toc">
        <div class="preview-area" bind:this={previewContainer} aria-hidden="true"></div>
        {#if isMarkdown && tocHeadings.length > 0 && mode === 'global-normal' && tocOpen}
          <TocSidebar
            bind:this={tocSidebar}
            headings={tocHeadings}
            activeLine={tocActiveLine}
            onJump={handleTocJump}
            onFocusChange={handleTocFocusChange}
          />
        {/if}
      </div>
      <div class="editor-area" bind:this={editorContainer}>
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="editor-overlay"
          class:overlay-hidden={mode !== 'editor-normal'}
          bind:this={overlayElement}
          onkeydown={handleOverlayKeydown}
          tabindex={mode === 'editor-normal' ? 0 : -1}
          role="region"
          aria-label="Editor navigation"
        ></div>
      </div>
    {:else}
      <div class="welcome">
        <h2>Welcome to Wind</h2>
        <p>Select a file to preview or edit</p>
        <div class="shortcuts">
          <h3>Keyboard Shortcuts</h3>
          <ul>
            <li><kbd>e</kbd> - Enter editor (vim normal mode)</li>
            <li><kbd>E</kbd> - Open fullscreen editor</li>
            <li><kbd>i</kbd> - Enter insert mode (in editor)</li>
            <li><kbd>Esc</kbd> - Back to normal mode (in editor)</li>
            <li><kbd>:w</kbd> - Save file</li>
            <li><kbd>:q</kbd> - Quit to preview</li>
            <li><kbd>:wq</kbd> - Save and quit</li>
            <li><kbd>:q!</kbd> - Force quit</li>
          </ul>
        </div>
      </div>
    {/if}
  </div>

  {#if mode === 'editor-normal' && overlayCmdActive}
    <div class="panel-cmdline">:{overlayCmdBuf}</div>
  {/if}

  {#if mode === 'editor-normal' && searchActive}
    <div class="panel-cmdline">/{searchBuf}</div>
  {/if}
</div>

<style>
  .preview-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-primary);
  }

  .panel-header {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    font-family: var(--font-mono);
  }

  .panel-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .file-name {
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modified-indicator {
    color: var(--warning);
    font-size: 12px;
  }

  .mode-indicator {
    font-size: 10px;
    padding: 1px 6px;
    background-color: var(--accent);
    color: var(--bg-primary);
    margin-left: auto;
  }

  .mode-indicator.normal {
    background-color: var(--success);
  }

  .mode-indicator.insert {
    background-color: var(--warning);
    color: var(--bg-primary);
  }

  .mode-indicator.toc {
    background-color: var(--success);
  }

  .toc-toggle {
    background: none;
    border: 1px solid var(--border);
    cursor: pointer;
    font-size: 10px;
    padding: 1px 6px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-left: 2px;
  }

  .toc-toggle:hover {
    color: var(--text-primary);
    border-color: var(--text-muted);
  }

  .toc-toggle:not(.closed) {
    color: var(--bg-primary);
    background-color: var(--accent);
    border-color: var(--accent);
  }

  .toc-toggle.closed {
    color: var(--text-muted);
  }

  .layout-toggle {
    background: none;
    border: 1px solid var(--border);
    cursor: pointer;
    padding: 2px 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    margin-left: auto;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .layout-toggle:hover {
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .layout-toggle.expanded {
    color: var(--accent);
    border-color: var(--accent);
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .preview-with-toc {
    display: flex;
    width: 100%;
    height: 100%;
  }

  .preview-area {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }

  .editor-area {
    width: 100%;
    height: 100%;
    display: none;
    position: relative;
  }

  .editor-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 10;
    background: transparent;
    outline: none;
  }

  .editor-overlay.overlay-hidden {
    display: none;
  }

  .panel-cmdline {
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-primary);
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--text-muted);
    max-width: 400px;
    margin: 0 auto;
    font-family: var(--font-mono);
  }

  .welcome h2 {
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .welcome p {
    margin-bottom: 24px;
  }

  .shortcuts {
    text-align: left;
    background-color: var(--bg-secondary);
    padding: 16px;
    width: 100%;
    border: 1px solid var(--border);
  }

  .shortcuts h3 {
    color: var(--text-primary);
    margin-bottom: 12px;
    font-size: 13px;
  }

  .shortcuts ul {
    list-style: none;
    padding: 0;
  }

  .shortcuts li {
    padding: 3px 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .shortcuts kbd {
    background-color: var(--bg-tertiary);
    padding: 1px 4px;
    font-family: var(--font-mono);
    font-size: 11px;
    border: 1px solid var(--border);
  }

  :global(.cm-editor) {
    height: 100%;
  }

  /* Hide CodeMirror search panel (we use our own overlay input) */
  :global(.cm-panel) {
    display: none !important;
  }

  /* Search match highlights */
  :global(.cm-searchMatch) {
    background-color: #fabd2f55;
    outline: 1px solid #fabd2f88;
    border-radius: 2px;
  }

  :global(.cm-searchMatch-selected) {
    background-color: #fabd2faa;
    outline: 1px solid #fabd2fcc;
    border-radius: 2px;
  }

  :global(.cm-sMatch) {
    background-color: #b8bb2644;
    outline: 1px solid #b8bb2688;
    border-radius: 2px;
  }

  :global(.cm-sMatch-replace) {
    background-color: #b8bb2644;
    outline: 1px solid #b8bb2688;
    border-radius: 2px;
    font-size: 0;
    color: transparent;
  }

  :global(.cm-sMatch-replace::after) {
    content: var(--s-replacement, '');
    font-size: initial;
    color: #83a598;
    font-style: italic;
  }

  :global(.dir-list) {
    font-family: 'Cascadia Code', 'Consolas', monospace;
    font-size: 13px;
    min-width: 0;
  }

  :global(.dir-entry) {
    display: flex;
    align-items: center;
    padding: 2px 0;
    gap: 6px;
    min-width: 0;
  }

  :global(.entry-name) {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--file-color);
  }

  :global(.entry-name.is-dir) {
    color: var(--dir-color);
    font-weight: 500;
  }

  :global(.entry-size) {
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 12px;
    min-width: 60px;
    text-align: right;
  }

  :global(.entry-size.dir) {
    visibility: hidden;
  }

  :global(.archive-header) {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 0 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
  }

  :global(.archive-icon) {
    font-size: 16px;
  }

  :global(.archive-name) {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 14px;
  }

  :global(.archive-meta) {
    color: var(--text-muted);
    font-size: 12px;
    margin-left: auto;
  }

  :global(.preview-empty) {
    color: var(--text-muted);
    text-align: center;
    padding: 24px;
    font-size: 13px;
  }

  :global(.preview-unsupported) {
    color: var(--text-muted);
    text-align: center;
    padding: 24px;
    font-size: 13px;
  }

  :global(.preview-code) {
    white-space: pre-wrap;
    word-break: break-all;
  }

  :global(.preview-code pre) {
    white-space: pre-wrap;
    word-break: break-all;
  }

  :global(.preview-plain) {
    white-space: pre-wrap;
    word-break: break-all;
  }

  :global(.preview-hex) {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  :global(.hex-notice) {
    padding: 8px 12px;
    background: #3c3836;
    color: #d79921;
    font-size: 12px;
    margin-bottom: 8px;
    border-radius: 4px;
  }

  :global(.hex-dump) {
    margin: 0;
    line-height: 1.5;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  :global(.hex-dump code) {
    font-family: var(--font-mono);
    font-size: 12px;
  }

  :global(.preview-image) {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
  }

  :global(.image-info-bar) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    color: var(--text-muted);
    font-size: 11px;
    flex-shrink: 0;
  }

  :global(.image-view-original) {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 1px 6px;
    cursor: pointer;
    font-size: 11px;
    font-family: var(--font-mono);
  }

  :global(.image-view-original:hover) {
    background-color: var(--bg-hover);
  }

  :global(.image-view-original:disabled) {
    opacity: 0.5;
    cursor: default;
  }

  :global(.preview-pdf-page) {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
  }

  :global(.pdf-info-bar) {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  /* ═══════════════════════════════════════════════════════
     Markdown Preview — Terminal style (Obsidian-inspired)
     ═══════════════════════════════════════════════════════ */

  :global(.preview-markdown) {
    font-family: var(--font-mono);
    font-size: 14px;
    line-height: 1.7;
    color: var(--text-primary);
  }

  /* ── Headings ── */

  :global(.preview-markdown h1),
  :global(.preview-markdown h2),
  :global(.preview-markdown h3),
  :global(.preview-markdown h4),
  :global(.preview-markdown h5),
  :global(.preview-markdown h6) {
    font-family: var(--font-mono);
    font-weight: 700;
    line-height: 1.3;
    margin-top: 1.6em;
    margin-bottom: 0.6em;
    color: var(--accent);
  }

  :global(.preview-markdown h1) {
    font-size: 1.8em;
    padding-bottom: 0.3em;
    border-bottom: 2px solid var(--border);
  }

  :global(.preview-markdown h2) {
    font-size: 1.5em;
    padding-bottom: 0.25em;
    border-bottom: 1px solid var(--border);
  }

  :global(.preview-markdown h3) {
    font-size: 1.25em;
  }

  :global(.preview-markdown h4) {
    font-size: 1.1em;
    color: var(--text-secondary);
  }

  :global(.preview-markdown h5) {
    font-size: 1em;
    color: var(--text-secondary);
  }

  :global(.preview-markdown h6) {
    font-size: 0.9em;
    color: var(--text-muted);
  }

  :global(.preview-markdown h1:first-child),
  :global(.preview-markdown h2:first-child),
  :global(.preview-markdown h3:first-child) {
    margin-top: 0;
  }

  /* ── Paragraphs & Text ── */

  :global(.preview-markdown p) {
    margin-top: 0;
    margin-bottom: 1em;
  }

  :global(.preview-markdown strong) {
    font-weight: 700;
    color: var(--text-primary);
  }

  :global(.preview-markdown em) {
    font-style: italic;
    color: var(--text-primary);
  }

  :global(.preview-markdown del) {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  /* ── Links ── */

  :global(.preview-markdown a) {
    color: var(--accent);
    text-decoration: none;
    border-bottom: 1px dashed var(--accent);
    transition: border-bottom-style 0.15s;
  }

  :global(.preview-markdown a:hover) {
    border-bottom-style: solid;
  }

  /* ── Horizontal Rule ── */

  :global(.preview-markdown hr) {
    border: none;
    border-top: 1px dashed var(--border);
    margin: 2em 0;
  }

  /* ── Lists ── */

  :global(.preview-markdown ul),
  :global(.preview-markdown ol) {
    padding-left: 2em;
    margin-bottom: 1em;
  }

  :global(.preview-markdown li) {
    margin-bottom: 0.3em;
  }

  :global(.preview-markdown li > p) {
    margin-bottom: 0.3em;
  }

  :global(.preview-markdown ul.contains-task-list) {
    list-style: none;
    padding-left: 0.5em;
  }

  :global(.preview-markdown .task-list-item) {
    display: flex;
    align-items: baseline;
    gap: 0.5em;
  }

  :global(.preview-markdown .task-list-item input[type="checkbox"]) {
    appearance: none;
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border: 1px solid var(--border);
    background: var(--bg-primary);
    cursor: default;
    flex-shrink: 0;
    position: relative;
    top: 2px;
  }

  :global(.preview-markdown .task-list-item input[type="checkbox"]:checked) {
    background: var(--accent);
    border-color: var(--accent);
  }

  :global(.preview-markdown .task-list-item input[type="checkbox"]:checked::after) {
    content: '✓';
    position: absolute;
    top: -2px;
    left: 1px;
    font-size: 11px;
    color: var(--bg-primary);
    font-weight: bold;
  }

  /* ── Blockquotes ── */

  :global(.preview-markdown blockquote) {
    margin: 1em 0;
    padding: 0.5em 1em;
    border-left: 3px solid var(--accent);
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }

  :global(.preview-markdown blockquote p:last-child) {
    margin-bottom: 0;
  }

  /* ── Inline Code ── */

  :global(.preview-markdown code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    padding: 0.15em 0.4em;
    background: var(--bg-tertiary);
    border-radius: 3px;
    color: var(--warning);
  }

  /* ── Fenced Code Blocks (Shiki) ── */

  :global(.preview-markdown pre) {
    margin: 1em 0;
    border-radius: 4px;
    white-space: pre-wrap;
    word-break: break-all;
    border: 1px solid var(--border);
  }

  :global(.preview-markdown pre code) {
    display: block;
    padding: 1em;
    font-size: 0.85em;
    line-height: 1.6;
    background: none;
    color: inherit;
    border-radius: 0;
  }

  /* Reset Shiki's inline styles so our theme takes over */
  :global(.preview-markdown pre code) {
    background-color: transparent !important;
  }

  :global(.preview-markdown pre code span) {
    /* Preserve Shiki token colors */
  }

  /* ── Tables ── */

  :global(.preview-markdown table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
    font-size: 0.9em;
  }

  :global(.preview-markdown thead th) {
    background: var(--bg-secondary);
    font-weight: 600;
    text-align: left;
    padding: 0.6em 0.8em;
    border: 1px solid var(--border);
  }

  :global(.preview-markdown tbody td) {
    padding: 0.5em 0.8em;
    border: 1px solid var(--border);
  }

  :global(.preview-markdown tbody tr:nth-child(even)) {
    background: var(--bg-secondary);
  }

  :global(.preview-markdown tbody tr:hover) {
    background: var(--bg-hover);
  }

  /* ── Images ── */

  :global(.preview-markdown img) {
    max-width: 100%;
    border-radius: 4px;
    margin: 0.5em 0;
  }

  /* ── Definition Lists ── */

  :global(.preview-markdown dl) {
    margin: 1em 0;
  }

  :global(.preview-markdown dt) {
    font-weight: 700;
    margin-top: 0.5em;
  }

  :global(.preview-markdown dd) {
    margin-left: 2em;
    color: var(--text-secondary);
  }

  /* ── LaTeX / KaTeX ── */

  :global(.preview-markdown .math-placeholder) {
    font-family: var(--font-mono);
    font-style: italic;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 0.1em 0.3em;
    border-radius: 3px;
    border: 1px dashed var(--border);
  }

  :global(.preview-markdown .katex-display) {
    margin: 1em 0;
    overflow-x: visible;
    text-align: center;
  }

  :global(.preview-markdown eq),
  :global(.preview-markdown eqn) {
    display: inline;
  }

  :global(.preview-markdown section.eqno),
  :global(.preview-markdown section:not(.eqno)) {
    display: block;
    text-align: center;
    margin: 1em 0;
  }

  /* ── Mermaid ── */

  :global(.preview-markdown .mermaid-container) {
    margin: 1em 0;
    text-align: center;
    overflow-x: auto;
  }

  :global(.preview-markdown .mermaid-container svg) {
    max-width: 100%;
    height: auto;
  }

  :global(.preview-markdown pre.mermaid-error) {
    border-left: 3px solid var(--error, #e74c3c);
  }
</style>
