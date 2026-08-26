<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy, tick } from 'svelte';
  import { layout } from '$lib/stores/layout';
  import { PreviewRouter, isVideoFileExt } from '$lib/previewers';
  import type { VideoMeta, TocHeading } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import TocSidebar from './TocSidebar.svelte';
  import { EditorView } from 'codemirror';
  import { EditorState, StateField, StateEffect, Compartment } from '@codemirror/state';
  import {
    keymap, Decoration, lineNumbers, highlightActiveLineGutter, highlightSpecialChars,
    drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine,
  } from '@codemirror/view';
  import { search, SearchQuery, setSearchQuery, findNext, findPrevious, openSearchPanel, closeSearchPanel, highlightSelectionMatches, searchKeymap } from '@codemirror/search';
  import {
    indentUnit, foldGutter, indentOnInput, bracketMatching,
    syntaxHighlighting, defaultHighlightStyle, foldKeymap,
  } from '@codemirror/language';
  import { autocompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { history, historyKeymap, defaultKeymap } from '@codemirror/commands';
  import { lintKeymap } from '@codemirror/lint';
  import { vim, Vim, getCM } from '@replit/codemirror-vim';
  import { getLanguage } from '$lib/utils/language';
  import { pythonCompletionSource } from '$lib/completions/python-completion';
  import { pythonLanguage } from '@codemirror/lang-python';
  import { createVimCommandHandler, setupAllVimCommands, getRegistersOutput } from '$lib/utils/vim-commands';
  import { initClipboardBridge, type ClipboardBridge } from '$lib/utils/clipboard-bridge';
  import { gruvboxDark, gruvboxLight, gruvboxTheme, getSyntaxTheme, suppressNativeSelection } from '$lib/utils/editor-theme';
  import { setupVimLineNumbers, teardownVimLineNumbers } from '$lib/utils/vim-line-numbers';
  import {
    editorAutocompleteKeymap,
    handleInsertModeEnter,
    handleInsertModeShiftTab,
    handleInsertModeTab,
  } from '$lib/utils/editor-text-keys';

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
            regex = new RegExp(pattern, flags.replace('g', '') + 'i');
          } catch { return Decoration.none as any; }
          const isVisualRange = m[1] === "'<,'>";
          const hasRange = !isVisualRange && (m[1] !== '' || m[2] !== '');
          const mark = Decoration.mark({ class: replacement ? 'cm-sMatch-replace' : 'cm-sMatch' });
          const decos: any[] = [];
          const doc = tr.state.doc;
          let startLine: number;
          let endLine: number;
          if (isVisualRange) {
            const sel = tr.state.selection.main;
            startLine = doc.lineAt(sel.from).number;
            endLine = doc.lineAt(sel.to).number;
          } else if (hasRange) {
            startLine = 1;
            endLine = doc.lines;
          } else {
            startLine = doc.lineAt(tr.state.selection.main.head).number;
            endLine = startLine;
          }
          for (let i = startLine; i <= endLine; i++) {
            const line = doc.line(i);
            if (global) {
              const lineRegex = new RegExp(pattern, 'gi');
              let m: RegExpExecArray | null;
              while ((m = lineRegex.exec(line.text)) !== null) {
                decos.push(mark.range(line.from + m.index, line.from + m.index + m[0].length));
                if (!m[0].length) break;
              }
            } else {
              const m = line.text.match(regex);
              if (m) decos.push(mark.range(line.from + m.index!, line.from + m.index! + m[0].length));
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
    previewTabId = undefined,
    onFullscreen = () => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onToast = (message: string) => {},
    onToggleLayout = () => {},
    batchRenameTempPath = null as string | null,
    onBatchRenameSave = (_content: string) => {},
    onBatchRenameCancel = () => {},
    activeColumn = '',
  }: {
    filePath: string | null;
    currentTabId?: number;
    previewTabId?: number;
    onFullscreen?: () => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onToast?: (message: string) => void;
    onToggleLayout?: () => void;
    batchRenameTempPath?: string | null;
    onBatchRenameSave?: (content: string) => void;
    onBatchRenameCancel?: () => void;
    activeColumn?: string;
  } = $props();

  // During t+n/t+p the selected tab is previewed before activeTabId commits.
  // Keep preview state keyed by that target tab instead of the origin tab.
  let renderTabId = $derived(previewTabId ?? currentTabId);

  let content: string = $state('');
  let savedContent: string = $state('');
  let binaryContent: ArrayBuffer | null = $state(null);
  let thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null = $state(null);
  let videoMeta: VideoMeta | null = $state(null);
  let originalFileSize: number = $state(0);
  let isModified: boolean = $state(false);
  let mode: 'global-normal' | 'editor-normal' | 'editor-insert' = $state('global-normal');
  let previewArea: HTMLElement | undefined = $state(undefined);
  let previewWithToc: HTMLElement | undefined = $state(undefined);
  let editorContainer: HTMLElement | undefined = $state(undefined);
  let panelElement: HTMLElement | undefined = $state(undefined);
  let previewRouter: PreviewRouter | undefined;
  let directoryPreviewer: DirectoryPreviewer | undefined;
  let editorView: EditorView | undefined = $state(undefined);
  let editorFilePath: string | null = $state(null);
  let themeObserver: MutationObserver | null = null;
  interface TextContentSnapshot {
    tabId: number;
    path: string;
    generation: number;
    content: string;
  }
  let loadGeneration: number = 0;
  let readyTextContent = $state<TextContentSnapshot | null>(null);
  let codeFileDirectEdit = $derived(
    !!filePath &&
    !binaryContent &&
    !!readyTextContent &&
    readyTextContent.tabId === renderTabId &&
    readyTextContent.path === filePath &&
    readyTextContent.generation === loadGeneration &&
    isDirectEditorFile(filePath)
  );
  let overlayElement: HTMLElement | undefined = $state(undefined);
  let renderRequestId: number = 0;
  const tabRenderVersions = new Map<number, number>();

  interface EditorSession {
    tabId: number;
    filePath: string;
    host: HTMLDivElement;
    view: EditorView;
    lineNumberCompartment: Compartment;
    themeCompartment: Compartment;
    resizeObserver: ResizeObserver;
    clipboardBridge: ClipboardBridge;
  }
  const editorSessions = new Map<number, EditorSession>();

  function tracePerformance(name: string, start: string): void {
    if (!import.meta.env.DEV || typeof performance === 'undefined') return;
    performance.mark(name);
    const measure = performance.measure(name, start, name);
    if (measure.duration > 16) console.debug(`[PreviewEditor] ${name}: ${measure.duration.toFixed(1)}ms`);
  }

  function destroyEditorSession(tabId: number): void {
    const session = editorSessions.get(tabId);
    if (!session) return;
    session.resizeObserver.disconnect();
    session.clipboardBridge.dispose();
    teardownVimLineNumbers(session.view);
    session.view.destroy();
    session.host.remove();
    editorSessions.delete(tabId);
    if (editorView === session.view) {
      editorView = undefined;
      editorFilePath = null;
      clipboardBridge = null;
    }
  }

  function activateEditorSession(tabId: number, path: string): boolean {
    const session = editorSessions.get(tabId);
    if (!session || session.filePath !== path) return false;
    if (import.meta.env.DEV) performance.mark('vim-session-activate-start');
    for (const candidate of editorSessions.values()) {
      candidate.host.style.display = candidate === session ? 'block' : 'none';
    }
    editorView = session.view;
    editorFilePath = session.filePath;
    clipboardBridge = session.clipboardBridge;
    let savedTopToRestore: number | null = null;
    if (pendingEditorPos >= 0 && pendingEditorPos <= session.view.state.doc.length) {
      session.view.dispatch({ selection: { anchor: pendingEditorPos } });
      pendingEditorPos = -1;
    }
    if (pendingEditorScrollTop >= 0) {
      savedTopToRestore = pendingEditorScrollTop;
      pendingEditorScrollTop = -1;
    }
    void refreshEditorLayoutAfterPaint(session, () => {
      if (savedTopToRestore !== null) session.view.scrollDOM.scrollTop = savedTopToRestore;
    });
    if (import.meta.env.DEV) tracePerformance('vim-session-activate', 'vim-session-activate-start');
    return true;
  }

  function hideEditorSessions(): void {
    for (const session of editorSessions.values()) session.host.style.display = 'none';
  }

  function nextAnimationFrame(): Promise<void> {
    return new Promise(resolve => requestAnimationFrame(() => resolve()));
  }

  function isVisibleBox(element: HTMLElement | undefined): boolean {
    if (!element || element.getClientRects().length === 0) return false;
    const rect = element.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }

  function isActiveEditorSession(session: EditorSession): boolean {
    return editorSessions.get(session.tabId) === session
      && editorView === session.view
      && renderTabId === session.tabId
      && filePath === session.filePath;
  }

  function getSessionForView(view: EditorView): EditorSession | undefined {
    for (const session of editorSessions.values()) {
      if (session.view === view) return session;
    }
  }

  function clampEditorScroll(view: EditorView): void {
    const s = view.scrollDOM;
    s.scrollTop = Math.max(0, Math.min(s.scrollTop, s.scrollHeight - s.clientHeight));
  }

  function hasStableEditorLayout(session: EditorSession): boolean {
    return isVisibleBox(editorContainer)
      && isVisibleBox(session.host)
      && isVisibleBox(session.view.dom)
      && isVisibleBox(session.view.scrollDOM);
  }

  async function waitForStableEditorLayout(session: EditorSession, maxFrames: number = 6): Promise<boolean> {
    await tick();
    for (let i = 0; i < maxFrames; i++) {
      if (!isActiveEditorSession(session)) return false;
      await nextAnimationFrame();
      if (!isActiveEditorSession(session)) return false;
      session.view.requestMeasure();
      if (hasStableEditorLayout(session)) return true;
    }
    return isActiveEditorSession(session) && hasStableEditorLayout(session);
  }

  async function refreshEditorLayoutAfterPaint(session: EditorSession, applyAfterMeasure?: () => void): Promise<void> {
    if (!await waitForStableEditorLayout(session)) return;
    if (!isActiveEditorSession(session)) return;
    session.view.requestMeasure();
    await nextAnimationFrame();
    if (!isActiveEditorSession(session)) return;
    applyAfterMeasure?.();
    clampEditorScroll(session.view);
    session.view.requestMeasure();
  }

  function focusActiveEditor(): void {
    if (mode === 'editor-normal' && overlayElement) {
      clipboardBridge?.refresh();
      overlayElement.focus({ preventScroll: true });
    } else if (mode === 'editor-insert' && editorView) {
      editorView.focus();
    }
  }

  export function focusActiveInput(): void {
    if (!outputVisible) focusActiveEditor();
  }

  export function prepareTabFocus(
    tabId: number,
    path: string | null,
    targetMode: 'editor-normal' | 'editor-insert' | 'global-normal',
  ): boolean {
    if (!path || targetMode === 'global-normal' || !activateEditorSession(tabId, path)) return false;
    mode = targetMode;
    if (previewWithToc) previewWithToc.style.display = 'none';
    if (editorContainer) editorContainer.style.display = 'block';
    focusActiveEditor();
    return true;
  }

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
  let pendingEditorPos: number = -1;
  let pendingEditorScrollTop: number = -1;

  // Per-tab editor state cache
  interface TabEditorCache {
    filePath: string;
    content: string;
    savedContent: string;
    binaryContent: ArrayBuffer | null;
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    editorCursorPos: number;
    editorScrollTop: number;
    previewScrollTop: number;
    isModified: boolean;
    pdfCurrentPage: number;
    pdfPageCount: number;
    fileMtime: number;
    tocOpen: boolean;
    tocHeadings: TocHeading[];
    tocExpandedLines: number[];
    tocFocused: boolean;
    tocSelectedIndex: number;
  }
  const tabEditorCache = new Map<number, TabEditorCache>();
  let pendingRestoreScrollTop: number = -1;
  let pendingTocSelectedIndex: number = -1;
  let currentFileMtime: number = 0;
  let fileChangedUnlisten: (() => void) | null = null;

  // Per-tab persistent preview slots.
  // Each tabId gets its own DOM container; slots stay in the document.
  // Switching tabs only detaches/attaches slots — no DOM tree is moved,
  // so the browser does not re-layout on every tab switch.
  const tabSlots = new Map<number, HTMLDivElement>();

  function getActiveSlot(): HTMLDivElement | undefined {
    return tabSlots.get(renderTabId);
  }

  function getOrCreateSlot(tabId: number): HTMLDivElement {
    let slot = tabSlots.get(tabId);
    if (!slot) {
      slot = document.createElement('div');
      slot.className = 'tab-preview-slot';
      slot.style.zIndex = '0';
      const fm = getComputedStyle(document.documentElement).getPropertyValue('--font-mono').trim();
      if (fm) slot.style.fontFamily = fm;
      previewArea?.appendChild(slot);
      tabSlots.set(tabId, slot);
    }
    return slot;
  }

  function showTabSlot(tabId: number) {
    // All slots remain visible — only z-index is toggled.
    // This avoids any layout invalidation (display/visibility changes
    // may cause browsers to discard cached layout for hidden elements).
    for (const [id, slot] of tabSlots) {
      slot.style.zIndex = id === tabId ? '1' : '0';
    }
  }

  let renderInFlight: boolean = false;
  let renderQueued: boolean = false;

  export function clearTabCache(tabId: number) {
    tabEditorCache.delete(tabId);
    tabRenderVersions.delete(tabId);
    const slot = tabSlots.get(tabId);
    if (slot) {
      slot.remove();
      tabSlots.delete(tabId);
    }
    destroyEditorSession(tabId);
  }

  export function cacheTabState(tabId: number) {
    if (!filePath) return;
    tabEditorCache.set(tabId, {
      filePath, content, savedContent, binaryContent,
      mode,
      editorCursorPos: editorView?.state.selection.main.head ?? 0,
      editorScrollTop: editorView?.scrollDOM.scrollTop ?? 0,
      previewScrollTop: tabSlots.get(tabId)?.scrollTop ?? 0,
      isModified, pdfCurrentPage, pdfPageCount, fileMtime: currentFileMtime,
      tocOpen, tocHeadings: [...tocHeadings],
      tocExpandedLines: collectExpandedLines(tocHeadings),
      tocFocused, tocSelectedIndex: tocSidebar?.getSelectedIndex() ?? -1,
    });
  }

  export function deactivateTab() {
    // Visual cleanup only: reset mode so the editor is hidden during tab
    // switch. editorView is NOT destroyed here — teardown is loadFile's job.
    mode = 'global-normal';
  }

  export function getEditorStateSnapshot(): {
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    previewScrollTop: number;
    isModified: boolean;
    pdfCurrentPage: number;
    tocOpen: boolean;
    tocExpandedLines: number[];
  } {
    return {
      mode,
      previewScrollTop: getActiveSlot()?.scrollTop ?? 0,
      isModified, pdfCurrentPage,
      tocOpen, tocExpandedLines: collectExpandedLines(tocHeadings),
    };
  }

  function startWatching(path: string) {
    stopWatching();
    invoke('start_watch_file', { path }).catch(e => console.error('[PreviewEditor] start_watch_file error:', e));
  }

  function stopWatching() {
    invoke('stop_watch_file').catch(() => {});
  }

  function isCurrentTextSnapshot(snapshot: TextContentSnapshot): boolean {
    const current = readyTextContent;
    return current !== null
      && current.tabId === snapshot.tabId
      && current.path === snapshot.path
      && current.generation === snapshot.generation
      && current.content === snapshot.content
      && snapshot.tabId === renderTabId
      && snapshot.path === filePath
      && snapshot.generation === loadGeneration
      && binaryContent === null;
  }

  async function handleFileChanged(eventPath: string) {
    if (!filePath) return;
    const a = eventPath.replace(/\//g, '\\').toLowerCase();
    const b = filePath.replace(/\//g, '\\').toLowerCase();
    if (a !== b) return;
    if (mode !== 'global-normal') return;
    tabEditorCache.delete(renderTabId);
    const slot = tabSlots.get(renderTabId);
    if (slot) { slot.innerHTML = ''; delete slot.dataset.rendered; }
    await loadFile(filePath);
  }

  // Redirect focus when active column switches away from preview
  $effect(() => {
    if (activeColumn && activeColumn !== 'preview' && mode !== 'global-normal') {
      if (activeColumn === 'terminal') return; // terminal focus handled by PanelLayout.focusPanel
      const target = activeColumn === 'current'
        ? document.querySelector('.current-panel .directory-panel')
        : document.querySelector('.parent-panel .directory-panel');
      if (target instanceof HTMLElement) {
        target.focus();
      }
    }
  });

  // When the active/preview tab changes, show the target slot.
  // Only clear the slot's content if it holds a different file;
  // keeping the rendered DOM for the same file avoids expensive
  // re-renders (especially for markdown with syntax highlighting).
  $effect(() => {
    const t0 = performance.now();
    const slot = tabSlots.get(renderTabId);
    const cleared = slot && slot.dataset.filePath !== filePath;
    if (cleared) {
      slot.innerHTML = '';
      delete slot.dataset.rendered;
      delete slot.dataset.filePath;
      delete slot.dataset.fileMtime;
    }
    showTabSlot(renderTabId);
    const t1 = performance.now();
    if (t1 - t0 > 1 || cleared) {
      console.log(`[tab-perf] preview showSlot tab=${renderTabId} cleared=${cleared} time=${(t1-t0).toFixed(1)}ms`);
    }
  });

  // Re-focus overlay after mouse selection (mouseup may fire outside editor panel)
  $effect(() => {
    if (mode !== 'editor-normal' || !overlayElement) return;
    function handleDocMouseUp() {
      requestAnimationFrame(() => {
        if (overlayElement && mode === 'editor-normal' && activeColumn === 'preview' && document.activeElement !== overlayElement) {
          focusActiveEditor();
        }
      });
    }
    document.addEventListener('mouseup', handleDocMouseUp);
    return () => document.removeEventListener('mouseup', handleDocMouseUp);
  });

  $effect(() => {
    if (!overlayElement) return;
    const blockComposition = (event: CompositionEvent) => { event.preventDefault(); event.stopPropagation(); };
    overlayElement.addEventListener('compositionstart', blockComposition, true);
    overlayElement.addEventListener('compositionend', blockComposition, true);
    return () => {
      overlayElement?.removeEventListener('compositionstart', blockComposition, true);
      overlayElement?.removeEventListener('compositionend', blockComposition, true);
    };
  });

  // Auto-focus output panel when it becomes visible
  $effect(() => {
    if (outputVisible) {
      requestAnimationFrame(() => {
        const el = document.querySelector('.panel-output') as HTMLElement | null;
        if (el) el.focus();
      });
    }
  });

  // Load file when filePath changes
  let _prevLoadKey: string | null = null;
  $effect(() => {
    const loadKey = filePath ? `${renderTabId}:${filePath}` : null;
    if (loadKey === _prevLoadKey) return;
    _prevLoadKey = loadKey;
    if (filePath) {
      const t0 = performance.now();
      getOrCreateSlot(renderTabId);
      showTabSlot(renderTabId);
      loadFile(filePath);
      const t1 = performance.now();
      console.log(`[tab-perf] preview loadFile dispatch tab=${renderTabId} file=${filePath.split(/[/\\]/).pop()} time=${(t1-t0).toFixed(1)}ms`);
    }
  });

  onDestroy(() => {
    previewRouter?.dispose();
    for (const tabId of [...editorSessions.keys()]) destroyEditorSession(tabId);
    scrollObserver?.disconnect();
    themeObserver?.disconnect();
    stopWatching();
    if (fileChangedUnlisten) { fileChangedUnlisten(); fileChangedUnlisten = null; }
  });

  listen('file-changed', (event: any) => {
    const changedPath = typeof event.payload === 'string' ? event.payload : String(event.payload ?? '');
    handleFileChanged(changedPath);
  }).then(unlisten => { fileChangedUnlisten = unlisten; });

  let prevMode: string = mode;
  $effect(() => {
    const m = mode;
    const changed = m !== prevMode;
    prevMode = m;
    if (m === 'editor-normal' || m === 'editor-insert') {
      if (previewWithToc) previewWithToc.style.display = 'none';
      if (editorContainer) editorContainer.style.display = 'block';
      const activeSession = editorView ? getSessionForView(editorView) : undefined;
      if (activeSession && isActiveEditorSession(activeSession)) {
        void refreshEditorLayoutAfterPaint(activeSession);
      }
      const textSnapshot = readyTextContent;
      if (!textSnapshot || !isCurrentTextSnapshot(textSnapshot)) {
        return;
      }
      if (activateEditorSession(renderTabId, textSnapshot.path)) {
        if (editorTargetLine >= 0 && editorView) moveCursorToLine(editorTargetLine);
        if (changed && activeColumn === 'preview') focusActiveEditor();
      } else if (editorContainer && (!editorSessions.has(textSnapshot.tabId) || editorSessions.get(textSnapshot.tabId)?.filePath !== textSnapshot.path)) {
        requestAnimationFrame(() => {
          if (mode !== 'editor-normal' && mode !== 'editor-insert') return;
          if (!isCurrentTextSnapshot(textSnapshot)) return;
          if (editorContainer && (!editorSessions.has(textSnapshot.tabId) || editorSessions.get(textSnapshot.tabId)?.filePath !== textSnapshot.path)) {
            initEditor(textSnapshot);
            if (activeColumn === 'preview') focusActiveEditor();
          }
        });
      } else if (editorView && editorTargetLine >= 0 && changed) {
        moveCursorToLine(editorTargetLine);
      }
      if (changed && activeColumn === 'preview' && editorView && editorFilePath === filePath) focusActiveEditor();
    } else {
      // Read editor scroll position BEFORE hiding editorContainer
      // (hidden container has zero layout, making lineBlockAtHeight unreliable)
      let pendingPreviewLine = -1;
      if (changed && editorView) {
        const vh = editorView.scrollDOM.clientHeight;
        const centerY = editorView.scrollDOM.scrollTop + vh / 2;
        const block = editorView.lineBlockAtHeight(centerY);
        if (block) {
          pendingPreviewLine = editorView.state.doc.lineAt(block.from).number - 1;
        }
      }
      if (previewWithToc) previewWithToc.style.display = '';
      if (editorContainer) editorContainer.style.display = 'none';
      hideEditorSessions();
      if (editorView) closeSearchPanel(editorView);
      if (pendingPreviewLine >= 0) {
        const targetLine = pendingPreviewLine;
        requestAnimationFrame(() => {
          const slot = getActiveSlot();
          const el = slot?.querySelector(`[data-line="${targetLine}"]`);
          if (el) el.scrollIntoView({ block: 'center', behavior: 'auto' });
        });
      }
      if (changed && codeFileDirectEdit) {
        const slot = getActiveSlot();
        if (slot) { slot.innerHTML = ''; delete slot.dataset.rendered; }
      }
      if (changed && panelElement && !tocFocused && activeColumn === 'preview') {
        panelElement.focus();
      }
    }
  });

  // Render preview when content is ready and in preview mode.
  // For fresh loads, loadFile calls renderPreview() directly with the correct
  // content — this effect handles re-renders triggered by mode transitions
  // (e.g. leaving editor mode) and file-changed events.
  $effect(() => {
    if (mode !== 'global-normal') return;
    if (!previewArea || !filePath) return;
    if (codeFileDirectEdit && content) {
      renderSimpleCodePreview();
    } else if (content || binaryContent) {
      renderPreview();
    }
  });

  function renderSimpleCodePreview() {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
    if (slot.dataset.rendered === 'true' && slot.dataset.filePath === filePath) return;
    slot.innerHTML = '';
    slot.dataset.filePath = filePath || '';
    slot.dataset.rendered = 'true';
    const pre = document.createElement('pre');
    pre.style.cssText = 'margin:0;white-space:pre-wrap;word-break:break-all;font-family:var(--font-mono);font-size:13px;line-height:1.5;color:var(--text-primary);';
    pre.textContent = content;
    slot.appendChild(pre);
  }

  export function getMode(): string { return mode; }
  export function getIsModified(): boolean { return isModified; }

  function handlePanelFocus() {
    if (outputVisible) return;
    if (mode === 'editor-normal' || mode === 'editor-insert') { focusActiveEditor(); }
    else if (mode === 'global-normal' && codeFileDirectEdit && filePath) {
      mode = 'editor-normal';
    }
  }

  export function enterEditorMode() {
    mode = 'editor-normal';
    void tick().then(() => focusActiveEditor());
  }

  export function pressTab() {
    if (!editorView || mode !== 'editor-insert') return;
    handleInsertModeTab(editorView);
  }

  export function pressShiftTab() {
    if (!editorView || mode !== 'editor-insert') return;
    handleInsertModeShiftTab(editorView);
  }

  function getPreviewRouter(): PreviewRouter {
    if (!previewRouter) {
      previewRouter = new PreviewRouter();
      previewRouter.onHeadings = (headings: TocHeading[]) => { tocHeadings = headings; tocActiveLine = -1; };
    }
    return previewRouter;
  }

  function getDirectoryPreviewer(): DirectoryPreviewer {
    if (!directoryPreviewer) { directoryPreviewer = new DirectoryPreviewer(); }
    return directoryPreviewer;
  }

  function setupScrollObserver() {
    scrollObserver?.disconnect();
    const slot = getActiveSlot();
    if (!slot || !isMarkdown) return;
    const headings = slot.querySelectorAll('h1, h2, h3, h4, h5, h6');
    if (headings.length === 0) return;
    headings.forEach((el, i) => { if (!el.id) el.id = `heading-${i}`; });
    scrollObserver = new IntersectionObserver(
      (entries) => {
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
          if (line >= 0) { tocActiveLine = line; }
        }
      },
      { root: slot, rootMargin: '-10% 0px -80% 0px', threshold: 0 }
    );
    headings.forEach((el) => { scrollObserver!.observe(el); });
  }

  function handleTocJump(line: number) {
    const slot = getActiveSlot();
    if (!slot) return;
    const heading = slot.querySelector(`[data-line="${line}"]`);
    if (heading) { heading.scrollIntoView({ behavior: 'smooth', block: 'start' }); tocActiveLine = line; }
  }

  export function isTocVisible(): boolean {
    return isMarkdown && tocHeadings.length > 0 && mode === 'global-normal' && tocOpen;
  }

  export function focusToc() { tocFocused = true; tocSidebar?.focus(); }
  export function focusContent() { tocFocused = false; if (panelElement) panelElement.focus({ preventScroll: true }); }
  export function isTocFocused(): boolean { return tocFocused; }

  function handleTocFocusChange(focused: boolean) {
    tocFocused = focused;
    if (!focused && panelElement) { panelElement.focus(); }
  }

  function isTextFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    const binaryExtensions = new Set([
      'exe','dll','so','dylib','bin','obj','o','a','lib','sys','drv',
      'rar','7z','tar','gz','bz2','xz','zst','lz4','cab',
      'mp3','wav','flac','aac','ogg','wma','m4a','opus','mid','midi',
      'mp4','mkv','avi','mov','wmv','flv','webm','m4v','mpg','mpeg','ts',
      'png','jpg','jpeg','gif','webp','bmp','ico','tiff','tif','psd','raw','cr2','nef',
      'pdf','doc','docx','xls','xlsx','ppt','pptx','odt','ods','odp',
      'ttf','otf','woff','woff2','eot',
      'db','sqlite','sqlite3','mdb','accdb','class','pyc','pyo',
      'iso','img','vhd','vhdx','qcow2',
    ]);
    return !binaryExtensions.has(ext);
  }

  const IMAGE_EXTENSIONS = new Set(['png','jpg','jpeg','gif','webp','bmp','ico','svg']);
  function isImageFile(path: string): boolean { return IMAGE_EXTENSIONS.has(path.split('.').pop()?.toLowerCase() || ''); }
  function isPdfFile(path: string): boolean { return (path.split('.').pop()?.toLowerCase() || '') === 'pdf'; }
  function isVideoFile(path: string): boolean { return isVideoFileExt(path); }
  function isDirectEditorFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return !['md', 'markdown', 'json', 'ipynb'].includes(ext);
  }
  const ARCHIVE_EXTENSIONS = new Set(['zip']);
  function isArchiveFile(path: string): boolean { return ARCHIVE_EXTENSIONS.has(path.split('.').pop()?.toLowerCase() || ''); }

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

    const loadTabId = renderTabId;
    const cached = tabEditorCache.get(loadTabId);
    if (cached && cached.filePath === path) {
      if (gen !== loadGeneration) return;
      content = cached.content;
      savedContent = cached.savedContent;
      binaryContent = cached.binaryContent;
      readyTextContent = cached.binaryContent ? null : {
        tabId: loadTabId,
        path,
        generation: gen,
        content: cached.content,
      };
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
      if (!isMarkdown) { tocHeadings = []; tocActiveLine = -1; }
      else if (cached.tocHeadings && cached.tocHeadings.length > 0) {
        if (pendingTocExpanded && pendingTocExpanded.size > 0) {
          restoreExpandedLines(cached.tocHeadings, pendingTocExpanded);
        }
        tocHeadings = cached.tocHeadings;
      }
      mode = cached.mode;
      const tCache = performance.now();
      if (cached.mode !== 'global-normal' && cached.editorCursorPos > 0) {
        pendingEditorPos = cached.editorCursorPos;
        pendingEditorScrollTop = cached.editorScrollTop;
      }
      if (!cached.content && !cached.binaryContent && cached.mode === 'global-normal') {
        renderPreview();
      }
      const tRender = performance.now();
      console.log(`[tab-perf] loadFile CACHE_HIT tab=${loadTabId} file=${fileName} cacheRestore=${(tCache-t0).toFixed(1)}ms render=${(tRender-tCache).toFixed(1)}ms total=${(tRender-t0).toFixed(1)}ms`);
      startWatching(path);
      invoke<{ size: number; modified: number }>('get_file_metadata', { path })
        .then(meta => { if (meta.modified !== cached.fileMtime && (mode === 'global-normal' || !cached.isModified)) { tabEditorCache.delete(loadTabId); if (loadTabId === renderTabId && filePath === path) loadFile(path); } })
        .catch(() => {});
      return;
    }

    // Reset content immediately so the render $effect (which fires before
    // async loadFile completes) does not render stale data from a previous file.
    content = '';
    binaryContent = null;
    readyTextContent = null;
    pendingRestoreScrollTop = -1;
    isModified = false;
    mode = 'global-normal';

    const ext = path.split('.').pop()?.toLowerCase() || '';
    isMarkdown = ext === 'md' || ext === 'markdown';
    if (!isMarkdown) { tocHeadings = []; tocActiveLine = -1; }

    // Directory
    try {
      await invoke<FileEntry[]>('read_directory', { path });
      if (gen !== loadGeneration) return;
      content = ''; binaryContent = null;
      destroyEditorSession(loadTabId);
      mode = 'global-normal';
      renderDirectoryPreview();
      stopWatching();
      return;
    } catch { /* not a directory */ }

    // Binary image
    if (isImageFile(path) && !path.toLowerCase().endsWith('.svg')) {
      try {
        if (path.toLowerCase().endsWith('.gif')) {
          const base64 = await invoke<string>('read_binary_file', { path });
          if (gen !== loadGeneration) return;
          const binary = atob(base64); const bytes = new Uint8Array(binary.length);
          for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
          binaryContent = bytes.buffer; thumbnailMeta = null;
        } else {
          const result = await invoke<{ data: string; width: number; height: number; original_size: number; is_thumbnail: boolean }>('read_image_thumbnail', { path });
          if (gen !== loadGeneration) return;
          if (result.data) {
            const binary = atob(result.data); const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
            binaryContent = bytes.buffer;
            thumbnailMeta = { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: result.is_thumbnail };
          } else {
            const base64 = await invoke<string>('read_binary_file', { path });
            if (gen !== loadGeneration) return;
            const binary = atob(base64); const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
            binaryContent = bytes.buffer;
            thumbnailMeta = { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: false };
          }
        }
        content = '[Binary Image]';
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load image:', error);
        binaryContent = null; thumbnailMeta = null; content = '';
      }
      mode = 'global-normal';
      renderPreview();
      return;
    }

    // PDF
    if (isPdfFile(path)) {
      try {
        const info = await invoke<{ page_count: number; title: string | null; author: string | null; file_size: number }>('get_pdf_info', { path });
        if (gen !== loadGeneration) return;
        pdfPageCount = info.page_count; pdfCurrentPage = 0; pdfFileSize = info.file_size; pdfTitle = info.title;
        await loadPdfPage(path, 0, gen);
        if (gen !== loadGeneration) return;
        content = '[PDF]'; mode = 'global-normal'; renderPreview();
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load PDF:', error);
        pdfPageCount = 0; pdfCurrentPage = 0; content = ''; binaryContent = null;
      }
      return;
    }

    // Archive
    if (isArchiveFile(path)) {
      content = ''; binaryContent = null;
      destroyEditorSession(loadTabId);
      mode = 'global-normal';
      renderArchivePreview(path);
      return;
    }

    // Video
    if (isVideoFile(path)) {
      try {
        const result = await invoke<VideoMeta>('get_video_thumbnail', { path });
        if (gen !== loadGeneration) return;
        videoMeta = result; binaryContent = null; content = JSON.stringify(result);
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load video thumbnail:', error);
        videoMeta = null; binaryContent = null; content = String(error);
      }
      mode = 'global-normal'; renderPreview();
      return;
    }

    // Text / binary
    const MAX_PREVIEW_SIZE = 1024 * 1024; // 1MB
    originalFileSize = 0;
    try {
      const meta = await invoke<{ size: number; modified: number }>('get_file_metadata', { path });
      if (gen !== loadGeneration) return;
      originalFileSize = meta.size; currentFileMtime = meta.modified;
    } catch { /* ignore */ }

    const usePartial = originalFileSize > MAX_PREVIEW_SIZE;
    try {
      const newContent = usePartial
        ? await invoke<string>('read_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
        : await invoke<string>('read_file', { path });
      if (gen !== loadGeneration) return;
      // Null bytes indicate binary data misread as text (from_utf8_lossy)
      if (newContent.includes('\0')) {
        try {
          const base64 = usePartial
            ? await invoke<string>('read_binary_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
            : await invoke<string>('read_binary_file', { path });
          if (gen !== loadGeneration) return;
          const binary = atob(base64); const bytes = new Uint8Array(binary.length);
          for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
          content = ''; binaryContent = bytes.buffer;
        } catch {
          if (gen !== loadGeneration) return;
          content = ''; binaryContent = null;
        }
      } else {
        content = newContent; binaryContent = null; savedContent = newContent;
        readyTextContent = { tabId: loadTabId, path, generation: gen, content: newContent };
      }
    } catch {
      if (gen !== loadGeneration) return;
      destroyEditorSession(loadTabId);
      try {
        const base64 = usePartial
          ? await invoke<string>('read_binary_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
          : await invoke<string>('read_binary_file', { path });
        if (gen !== loadGeneration) return;
        const binary = atob(base64); const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        content = ''; binaryContent = bytes.buffer;
      } catch {
        if (gen !== loadGeneration) return;
        content = ''; binaryContent = null;
      }
    }
    // Code files go directly to editor mode (no Shiki preview)
    if (readyTextContent && isDirectEditorFile(path)) {
      mode = 'editor-normal';
    } else {
      destroyEditorSession(loadTabId);
      mode = 'global-normal';
      renderPreview();
    }
    startWatching(path);
  }

  function requestTabRender(tabId: number): number {
    const version = (tabRenderVersions.get(tabId) ?? 0) + 1;
    tabRenderVersions.set(tabId, version);
    return version;
  }

  function isCurrentTabRender(tabId: number, path: string, version: number): boolean {
    return renderTabId === tabId
      && filePath === path
      && tabRenderVersions.get(tabId) === version;
  }

  async function renderPreview() {
    if (!previewArea || !filePath) return;
    requestTabRender(renderTabId);

    // PreviewRouter and MarkdownPreviewer keep instance-local mutable state.
    // Serialize work instead of dropping a render while another tab is pending;
    // the queue always renders the newest tab/file context next.
    if (renderInFlight) {
      renderQueued = true;
      return;
    }

    renderInFlight = true;
    try {
      do {
        renderQueued = false;
        await renderPreviewOnce();
      } while (renderQueued);
    } finally {
      renderInFlight = false;
      // A render request may arrive between the loop condition and finally.
      if (renderQueued) {
        renderQueued = false;
        void renderPreview();
      }
    }
  }

  async function renderPreviewOnce() {
    if (!previewArea || !filePath) return;

    const tabId = renderTabId;
    const path = filePath;
    const tabVersion = tabRenderVersions.get(tabId) ?? requestTabRender(tabId);
    const slot = getOrCreateSlot(tabId);
    showTabSlot(tabId);

    // Skip if this tab's slot already holds a fresh render of the same file.
    if (slot.dataset.rendered === 'true' && slot.dataset.filePath === path
        && slot.dataset.fileMtime === String(currentFileMtime)) {
      pendingRestoreScrollTop = -1;
      if (isMarkdown) { requestAnimationFrame(() => setupScrollObserver()); }
      if (tocFocused && tocOpen && pendingTocSelectedIndex >= 0) {
        requestAnimationFrame(() => { tocSidebar?.setSelectedTocIndex(pendingTocSelectedIndex); pendingTocSelectedIndex = -1; tocSidebar?.focus(); });
      }
      return;
    }

    slot.innerHTML = '';
    slot.dataset.filePath = path;

    const requestId = ++renderRequestId;
    if (thumbnailMeta) {
      slot.dataset.thumbWidth = String(thumbnailMeta.width);
      slot.dataset.thumbHeight = String(thumbnailMeta.height);
      slot.dataset.thumbOriginalSize = String(thumbnailMeta.originalSize);
      slot.dataset.thumbIsThumbnail = String(thumbnailMeta.isThumbnail);
    } else {
      delete slot.dataset.thumbWidth; delete slot.dataset.thumbHeight;
      delete slot.dataset.thumbOriginalSize; delete slot.dataset.thumbIsThumbnail;
    }
    if (originalFileSize > 0) { slot.dataset.originalFileSize = String(originalFileSize); }
    else { delete slot.dataset.originalFileSize; }

    const previewContent: string | ArrayBuffer = binaryContent ?? content;
    await getPreviewRouter().preview(path, previewContent, slot);
    if (requestId !== renderRequestId || !isCurrentTabRender(tabId, path, tabVersion)) return;

    slot.dataset.rendered = 'true';
    slot.dataset.fileMtime = String(currentFileMtime);

    const savedScroll2 = pendingRestoreScrollTop;
    pendingRestoreScrollTop = -1;
    if (savedScroll2 >= 0) { requestAnimationFrame(() => { slot.scrollTop = savedScroll2; }); }
    if (pendingTocExpanded && tocHeadings.length > 0) {
      restoreExpandedLines(tocHeadings, pendingTocExpanded);
      pendingTocExpanded = null; tocHeadings = [...tocHeadings];
    }
    if (tocFocused && tocOpen) {
      requestAnimationFrame(() => {
        if (pendingTocSelectedIndex >= 0) { tocSidebar?.setSelectedTocIndex(pendingTocSelectedIndex); pendingTocSelectedIndex = -1; }
        tocSidebar?.focus();
      });
    }
    if (isPdfFile(path) && pdfPageCount > 0) { addPdfInfoBar(slot); }
    if (isMarkdown) { setupScrollObserver(); }
  }

  function scrollPreview(deltaY: number, deltaX: number = 0) {
    const slot = getActiveSlot();
    if (slot) slot.scrollBy({ top: deltaY, left: deltaX, behavior: 'auto' });
  }

  export function getVisibleLine(): number {
    const slot = getActiveSlot();
    if (!slot || !content) return 0;
    const rect = slot.getBoundingClientRect();
    const el = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
    if (el) {
      const lined = el.closest('[data-line]');
      if (lined) { const line = parseInt(lined.getAttribute('data-line')!); if (!isNaN(line)) return line; }
    }
    const maxScroll = slot.scrollHeight - slot.clientHeight;
    if (maxScroll <= 0) return 0;
    return Math.round((slot.scrollTop / maxScroll) * (content.split('\n').length - 1));
  }

  function getPosAtLine(text: string, lineNumber: number): number {
    let pos = 0;
    for (let i = 0, line = 0; i < text.length && line < lineNumber; i++) {
      if (text[i] === '\n') line++;
      if (line < lineNumber) pos = i + 1;
    }
    return pos;
  }

  function moveCursorToLine(lineNumber: number) {
    if (!editorView) return;
    const targetLine = Math.min(lineNumber + 1, editorView.state.doc.lines);
    editorView.dispatch({ selection: { anchor: editorView.state.doc.line(targetLine).from } });
    scrollEditorToPos(editorView, editorView.state.selection.main.head);
    editorTargetLine = -1;
  }

  function scrollEditorToPos(view: EditorView, pos: number) {
    const session = getSessionForView(view);
    if (session) {
      void refreshEditorLayoutAfterPaint(session, () => {
        view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
      });
      return;
    }
    view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
    requestAnimationFrame(() => {
      requestAnimationFrame(() => clampEditorScroll(view));
    });
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function addPdfInfoBar(container: HTMLElement) {
    container.querySelector('.pdf-info-bar')?.remove();
    const bar = document.createElement('div');
    bar.className = 'pdf-info-bar image-info-bar';
    const info = document.createElement('span');
    info.textContent = `${pdfCurrentPage + 1}/${pdfPageCount}`;
    if (pdfFileSize > 0) info.textContent += ` · ${formatSize(pdfFileSize)}`;
    if (pdfTitle) info.textContent += ` · ${pdfTitle}`;
    bar.appendChild(info);
    const hints = document.createElement('span');
    hints.className = 'pdf-hints';
    hints.textContent = 'J/K:翻页 E:全屏';
    hints.style.color = '#666'; hints.style.fontSize = '11px';
    bar.appendChild(hints);
    container.appendChild(bar);
  }

  async function renderDirectoryPreview() {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
    if (!filePath) return;
    const requestId = ++renderRequestId;
    const previewer = getDirectoryPreviewer();
    slot.dataset.filePath = filePath;
    await previewer.render('', slot);
    if (requestId !== renderRequestId) return;
  }

  async function renderArchivePreview(path: string) {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
    const requestId = ++renderRequestId;
    slot.dataset.filePath = path;
    await getPreviewRouter().preview(path, '', slot);
    if (requestId !== renderRequestId) return;
  }

  async function loadPdfPage(path: string, page: number, gen?: number) {
    const g = gen ?? loadGeneration;
    try {
      const result = await invoke<{ data: string; width: number; height: number }>('render_pdf_page', { path, page, scale: pdfRenderScale });
      if (g !== loadGeneration) return;
      const binary = atob(result.data); const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      binaryContent = bytes.buffer; pdfCurrentPage = page;
    } catch (error) {
      if (g !== loadGeneration) return;
      console.error(`Failed to render PDF page ${page}:`, error);
    }
  }

  function initEditor(textSnapshot: TextContentSnapshot) {
    if (!editorContainer || !isCurrentTextSnapshot(textSnapshot)) return;
    if (activateEditorSession(textSnapshot.tabId, textSnapshot.path)) return;
    destroyEditorSession(textSnapshot.tabId);
    if (import.meta.env.DEV) performance.mark('vim-session-create-start');
    const tabId = textSnapshot.tabId;
    const targetPath = textSnapshot.path;
    const initialContent = textSnapshot.content;
    const host = document.createElement('div');
    host.className = 'editor-session';
    editorContainer.appendChild(host);
    const sessionLineNumberCompartment = new Compartment();
    const sessionThemeCompartment = new Compartment();
    editorFilePath = targetPath;
    const language = getLanguage(targetPath);
    const extensions = [
      highlightActiveLineGutter(), highlightSpecialChars(), history(),
      foldGutter(), drawSelection(), dropCursor(), autocompletion({ defaultKeymap: false }),
      bracketMatching(), closeBrackets(), crosshairCursor(),
      highlightActiveLine(), highlightSelectionMatches(), indentOnInput(),
      rectangularSelection(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      keymap.of([...closeBracketsKeymap, ...defaultKeymap, ...searchKeymap, ...historyKeymap, ...foldKeymap, ...editorAutocompleteKeymap, ...lintKeymap]),
      sessionLineNumberCompartment.of(lineNumbers()),
      search({ top: true }), sMatchField, EditorView.lineWrapping,
      indentUnit.of('    '),
      vim({ status: false }),
      createVimCommandHandler(
        () => ({
          save: async () => { if (batchRenameTempPath) onBatchRenameSave(content); else await saveFile(); },
          quit: () => { if (batchRenameTempPath) onBatchRenameCancel(); else if (!codeFileDirectEdit) mode = 'global-normal'; },
          forceQuit: () => { if (batchRenameTempPath) onBatchRenameCancel(); else if (!codeFileDirectEdit) { content = savedContent; isModified = false; mode = 'global-normal'; } },
          isModified: () => isModified,
        }),
        (msg) => onToast(msg)
      ),
      sessionThemeCompartment.of(getSyntaxTheme()),
      suppressNativeSelection,
      gruvboxTheme,
      EditorView.theme({
        '&': { fontFamily: "'Consolas', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Courier New', monospace" },
        '.cm-content': { fontFamily: "'Consolas', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Courier New', monospace" },
      }),
      EditorView.updateListener.of((update) => {
        const isActive = renderTabId === tabId && editorSessions.get(tabId)?.view === update.view;
        if (update.docChanged && isActive) { content = update.state.doc.toString(); isModified = content !== savedContent; }
        const cm = (update.view as any).cm;
        const vimState = cm?.state?.vim;
        if (vimState && isActive) {
          if (vimState.insertMode && mode === 'editor-normal') mode = 'editor-insert';
          else if (!vimState.insertMode && !vimState.visualMode && mode === 'editor-insert') mode = 'editor-normal';
          update.view.dom.classList.toggle('vim-visual', !!vimState.visualMode);
          update.view.dom.classList.toggle('cm-insert-selecting', !!vimState.insertMode && !update.state.selection.main.empty);
        } else {
          update.view.dom.classList.remove('vim-visual');
          update.view.dom.classList.remove('cm-insert-selecting');
        }
      }),
    ];
    if (language) extensions.push(language);
    if (targetPath.toLowerCase().endsWith('.py')) {
      extensions.push(pythonLanguage.data.of({ autocomplete: pythonCompletionSource }));
    }
    const restorePos = pendingEditorPos >= 0 ? pendingEditorPos : 0;
    const state = EditorState.create({
      doc: initialContent, extensions,
      selection: pendingEditorPos >= 0 ? { anchor: pendingEditorPos }
        : editorTargetLine >= 0 ? { anchor: getPosAtLine(initialContent, editorTargetLine) }
        : undefined,
    });
    const needsScroll = pendingEditorPos >= 0 || editorTargetLine >= 0;
    editorTargetLine = -1;
    pendingEditorPos = -1;
    const view = new EditorView({ state, parent: host });
    editorView = view;
    savedContent = view.state.doc.toString();
    isModified = false;

    // Clamp editor scroll on container resize (terminal drag, panel resize, etc.)
    const resizeObserver = new ResizeObserver(() => {
      if (editorSessions.get(tabId)?.view === view) {
        const activeSession = editorSessions.get(tabId);
        if (activeSession && isActiveEditorSession(activeSession)) {
          void refreshEditorLayoutAfterPaint(activeSession);
        } else {
          clampEditorScroll(view);
        }
      }
    });
    resizeObserver.observe(host);

    const bridge = initClipboardBridge(view);
    const session: EditorSession = {
      tabId, filePath: targetPath, host, view,
      lineNumberCompartment: sessionLineNumberCompartment,
      themeCompartment: sessionThemeCompartment,
      resizeObserver, clipboardBridge: bridge,
    };
    editorSessions.set(tabId, session);
    clipboardBridge = bridge;

    setupVimLineNumbers(sessionLineNumberCompartment, view);
    setupAllVimCommands((text) => {
      outputText = text;
      outputVisible = true;
    });
    if (pendingEditorScrollTop >= 0) {
      const savedTop = pendingEditorScrollTop;
      pendingEditorScrollTop = -1;
      void refreshEditorLayoutAfterPaint(session, () => {
        view.scrollDOM.scrollTop = savedTop;
      });
    } else if (needsScroll) {
      scrollEditorToPos(view, view.state.selection.main.head);
    } else {
      void refreshEditorLayoutAfterPaint(session);
    }

    // Watch theme changes to swap syntax highlighting
    if (!themeObserver) {
      themeObserver = new MutationObserver(() => {
        for (const session of editorSessions.values()) {
          const isLight = document.documentElement.getAttribute('data-theme') === 'light';
          session.view.dispatch({ effects: session.themeCompartment.reconfigure(isLight ? gruvboxLight : gruvboxDark) });
        }
      });
      themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    }

    view.contentDOM.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key !== 'Enter' || mode !== 'editor-insert') return;
      if (handleInsertModeEnter(view)) {
        e.preventDefault();
        e.stopPropagation();
      }
    }, true);
    if (import.meta.env.DEV) tracePerformance('vim-session-create', 'vim-session-create-start');
  }

  function codeToVimKey(event: KeyboardEvent): string | null {
    const code = event.code;
    let key = '';
    if (event.ctrlKey) key += 'C-';
    if (event.altKey) key += 'A-';
    if (event.metaKey) key += 'M-';
    if (code.startsWith('Key') && code.length === 4) key += event.shiftKey ? code[3] : code[3].toLowerCase();
    else if (code === 'Enter') key += 'Enter';
    else if (code === 'Space') key += 'Space';
    else if (code === 'Escape') key = 'Esc';
    else if (code === 'Backspace') key += 'BS';
    else if (code === 'Tab') key += 'Tab';
    else if (code === 'Delete') key += 'Del';
    else if (code.startsWith('Digit')) {
      const shifted = ')!@#$%^&*(';
      key += event.shiftKey ? shifted[parseInt(code[5])] : code[5];
    }
    else if (code.startsWith('Arrow')) key += code.slice(5);
    else if (code === 'BracketLeft') key += event.shiftKey ? '{' : '[';
    else if (code === 'BracketRight') key += event.shiftKey ? '}' : ']';
    else if (code === 'Semicolon') key += event.shiftKey ? ':' : ';';
    else if (code === 'Quote') key += event.shiftKey ? '"' : "'";
    else if (code === 'Comma') key += event.shiftKey ? '<' : ',';
    else if (code === 'Period') key += event.shiftKey ? '>' : '.';
    else if (code === 'Slash') key += event.shiftKey ? '?' : '/';
    else if (code === 'Backslash') key += '\\';
    else if (code === 'Minus') key += event.shiftKey ? '_' : '-';
    else if (code === 'Equal') key += event.shiftKey ? '+' : '=';
    else if (code === 'Backquote') key += event.shiftKey ? '~' : '`';
    else return null;
    if (key.length > 1) key = '<' + key + '>';
    return key;
  }

  let overlayCmdBuf = $state('');
  let overlayCmdActive = $state(false);
  let clipboardBridge: ClipboardBridge | null = null;
  let searchActive: boolean = $state(false);
  let searchBuf: string = $state('');
  let outputVisible: boolean = $state(false);
  let outputText: string = $state('');
  let outputExitCode: number = $state(0);

  // Tab completion state
  let completions: { name: string; is_dir: boolean }[] = [];
  let completionIndex: number = -1;
  let completionPrefix: string = '';
  let completionDir: string = '';

  async function triggerFileCompletion() {
    const cmd = overlayCmdBuf;
    // Only complete after :! prefix with a non-empty partial
    if (!cmd.startsWith('!')) return;
    const afterBang = cmd.slice(1); // text after !
    const lastSpace = afterBang.lastIndexOf(' ');
    const partial = lastSpace >= 0 ? afterBang.slice(lastSpace + 1) : '';
    if (!partial) return;

    const cwd = filePath ? filePath.split(/[/\\]/).slice(0, -1).join('\\') || 'C:\\' : 'C:\\';

    // Re-fetch if directory changed
    if (completionDir !== cwd || completions.length === 0) {
      try {
        const entries = await invoke<{ name: string; is_dir: boolean }[]>('read_directory', { path: cwd });
        completions = entries
          .filter(e => e.name.toLowerCase().startsWith(partial.toLowerCase()))
          .sort((a, b) => a.name.localeCompare(b.name));
        completionDir = cwd;
        completionPrefix = partial;
        completionIndex = 0;
      } catch {
        resetCompletion();
        return;
      }
    } else {
      // Cycle to next match
      if (completions.length > 0) {
        completionIndex = (completionIndex + 1) % completions.length;
      }
    }

    if (completions.length === 0) return;

    const completed = completions[completionIndex];
    const prefix = lastSpace >= 0 ? afterBang.slice(0, lastSpace + 1) : '';
    overlayCmdBuf = '!' + prefix + completed.name;
  }

  function resetCompletion() {
    completions = [];
    completionIndex = -1;
    completionPrefix = '';
    completionDir = '';
  }

  function executeSearch() {
    if (!editorView || !searchBuf) return;
    openSearchPanel(editorView);
    const query = new SearchQuery({ search: searchBuf, caseSensitive: false });
    editorView.dispatch({ effects: setSearchQuery.of(query) });
    findNext(editorView);
  }

  function highlightSMatches() {
    if (!editorView) return;
    const cmd = overlayCmdBuf;
    const delimMatch = cmd.match(/^(['<,'>]*)([%]?)s(.)/);
    if (delimMatch) {
      const delim = delimMatch[3];
      const esc = delim.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const re = new RegExp(`s${esc}([^${esc}]*)(?:${esc}([^${esc}]*))?(?:${esc}([ggiI]*))?`);
      const pm = cmd.match(re);
      if (pm && pm[2] !== undefined) { editorView.dom.style.setProperty('--s-replacement', JSON.stringify(pm[2])); }
      else { editorView.dom.style.removeProperty('--s-replacement'); }
    } else { editorView.dom.style.removeProperty('--s-replacement'); }
    editorView.dispatch({ effects: triggerSMatchUpdate.of() });
  }

  async function executeShellCommand(command: string) {
    const cwd = filePath ? filePath.split(/[/\\]/).slice(0, -1).join('\\') || null : null;
    outputVisible = true;
    outputText = 'Executing...';
    outputExitCode = 0;
    try {
      const result = await invoke<{ stdout: string; stderr: string; exit_code: number }>('exec_shell_command', { command, cwd });
      outputText = result.stdout;
      if (result.stderr) outputText += '\n' + result.stderr;
      outputExitCode = result.exit_code;
      if (!outputText.trim()) outputText = '(no output)';
    } catch (error) {
      outputText = String(error);
      outputExitCode = -1;
    }
  }

  function closeOutputPanel() {
    outputVisible = false;
    outputText = '';
    if (mode === 'editor-normal' && overlayElement) {
      overlayElement.focus();
    } else if (mode === 'editor-insert' && editorView) {
      editorView.focus();
    }
  }

  function handleOutputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      closeOutputPanel();
    }
  }

  function processOverlayCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (trimmed.startsWith('!')) {
      const shellCmd = trimmed.slice(1).trim();
      if (!shellCmd) { onToast('E471: Argument required'); return; }
      executeShellCommand(shellCmd);
      return;
    }
    if (trimmed === 'w' || trimmed === 'write') {
      if (batchRenameTempPath) onBatchRenameSave(content); else saveFile();
    } else if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') {
      if (batchRenameTempPath) onBatchRenameCancel();
      else { content = savedContent; isModified = false; mode = 'global-normal'; }
    } else if (trimmed === 'q' || trimmed === 'quit') {
      if (batchRenameTempPath) onBatchRenameCancel();
      else if (isModified) onToast('E37: No write since last change (add ! to override)');
      else mode = 'global-normal';
    } else if (trimmed === 'wq' || trimmed === 'x') {
      if (batchRenameTempPath) onBatchRenameSave(content);
      else saveFile().then(() => { mode = 'global-normal'; });
    } else if (trimmed === 'wqall' || trimmed === 'wqall!') {
      if (batchRenameTempPath) onBatchRenameSave(content);
      else saveFile().then(() => { mode = 'global-normal'; });
    } else if (trimmed === 'reg' || trimmed === 'registers') {
      const regOutput = getRegistersOutput();
      outputText = regOutput;
      outputVisible = true;
    } else if (editorView) {
      const cm = getCM(editorView);
      if (cm) { Vim.handleEx(cm as any, trimmed); editorView.dispatch({ effects: clearSMatch.of() }); editorView.dom.style.removeProperty('--s-replacement'); }
    }
  }

  let ctrlWPending: boolean = false;

  function handleOverlayKeydown(event: KeyboardEvent) {
    // Pass Ctrl+W window nav keys through to PanelLayout
    if (event.ctrlKey && event.key === 'w') {
      ctrlWPending = true;
      return;
    }
    if (ctrlWPending) {
      ctrlWPending = false;
      if (event.code === 'KeyH' || event.code === 'KeyL' || event.code === 'KeyJ' || event.code === 'KeyK' || event.code === 'KeyM') {
        return;
      }
    }

    event.preventDefault(); event.stopPropagation();
    if (outputVisible) {
      if (event.key === 'Enter' || event.key === 'Escape') {
        closeOutputPanel();
      }
      return;
    }
    if (overlayCmdActive) {
      if (event.key === 'Enter') { overlayCmdActive = false; processOverlayCommand(overlayCmdBuf); overlayCmdBuf = ''; setTimeout(() => { if (overlayElement && mode === 'editor-normal') overlayElement.focus(); }, 0); return; }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') { overlayCmdActive = false; overlayCmdBuf = ''; onToast(''); if (editorView) { editorView.dispatch({ effects: clearSMatch.of() }); editorView.dom.style.removeProperty('--s-replacement'); } return; }
      if (event.key === 'Backspace') { if (overlayCmdBuf.length > 0) overlayCmdBuf = overlayCmdBuf.slice(0, -1); else overlayCmdActive = false; highlightSMatches(); return; }
      if (event.key === 'Tab') { event.preventDefault(); triggerFileCompletion(); return; }
      if (event.key.length === 1) { overlayCmdBuf += event.key; resetCompletion(); highlightSMatches(); }
      return;
    }
    if (searchActive) {
      if (event.key === 'Enter') { searchActive = false; executeSearch(); return; }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') { searchActive = false; searchBuf = ''; return; }
      if (event.key === 'Backspace') { if (searchBuf.length > 0) searchBuf = searchBuf.slice(0, -1); else searchActive = false; return; }
      if (event.key.length === 1) searchBuf += event.key;
      return;
    }
    if (event.key === ':') {
      overlayCmdActive = true;
      overlayCmdBuf = '';
      if (editorView) {
        const cm = getCM(editorView);
        if (cm?.state?.vim?.visualMode) {
          overlayCmdBuf = "'<,'>";
        }
      }
      return;
    }
    if (event.key === '/' || event.key === '?') { searchActive = true; searchBuf = ''; return; }

    if (event.code === 'KeyN' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      if (editorView) { if (event.shiftKey) findPrevious(editorView); else findNext(editorView); }
      return;
    }
    if (event.key === 'Escape' && editorView) closeSearchPanel(editorView);
    const vimKey = codeToVimKey(event);
    if (!vimKey || !editorView) return;
    if ((vimKey === 'p' || vimKey === 'P') && clipboardBridge) { clipboardBridge.injectClipboard(); }
    const cm = getCM(editorView);
    if (!cm) return;
    (Vim as any).multiSelectHandleKey?.(cm, vimKey, 'user');
    const vimState = cm.state?.vim;
    if (vimState?.insertMode) { mode = 'editor-insert'; }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (mode !== 'global-normal') return;
    if (event.ctrlKey && event.code === 'KeyL' && isMarkdown && tocHeadings.length > 0) { event.preventDefault(); event.stopPropagation(); focusToc(); return; }
    if (event.code === 'KeyE' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); if (!filePath || !isTextFile(filePath)) { onToast('此文件类型不支持编辑'); return; } if (!content && originalFileSize > 0) { onToast('文件加载中，请稍候'); return; } editorTargetLine = getVisibleLine(); mode = 'editor-normal'; }
    else if (event.code === 'KeyE' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); if (!filePath) { onToast('此文件类型不支持全屏查看'); return; } onFullscreen(); }
    else if (event.code === 'KeyJ' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(40); }
    else if (event.code === 'KeyK' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(-40); }
    else if (event.code === 'KeyH' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(0, -40); }
    else if (event.code === 'KeyL' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(0, 40); }
    else if (event.code === 'KeyJ' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { if (filePath && isPdfFile(filePath) && pdfCurrentPage < pdfPageCount - 1) { event.preventDefault(); loadPdfPage(filePath, pdfCurrentPage + 1); } }
    else if (event.code === 'KeyK' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { if (filePath && isPdfFile(filePath) && pdfCurrentPage > 0) { event.preventDefault(); loadPdfPage(filePath, pdfCurrentPage - 1); } }
    else if (event.code === 'KeyG' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); const ggSlot = getActiveSlot(); if (ggSlot) ggSlot.scrollTop = 0; }
    else if (event.code === 'KeyG' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); const gSlot = getActiveSlot(); if (gSlot) gSlot.scrollTop = gSlot.scrollHeight; }
    else if (event.ctrlKey && event.code === 'KeyS') { event.preventDefault(); saveFile(); }
  }

  async function saveFile() {
    if (!filePath || !isModified) return;
    try {
      await invoke('write_file', { path: filePath, content });
      savedContent = content; isModified = false;
      // Get actual file mtime after save so metadata check doesn't
      // falsely invalidate cached content on tab switch
      const meta = await invoke<{ modified: number }>('get_file_metadata', { path: filePath });
      currentFileMtime = meta.modified;
      const cached = tabEditorCache.get(renderTabId);
      if (cached && cached.filePath === filePath) {
        cached.content = content;
        cached.savedContent = savedContent;
        cached.isModified = false;
        cached.fileMtime = meta.modified;
      }
      const saveSlot = getActiveSlot();
      if (saveSlot) { delete saveSlot.dataset.rendered; }
    } catch (error) { console.error('Failed to save file:', error); }
  }

  function getFileName(): string { if (!filePath) return ''; return filePath.split('\\').pop() || filePath.split('/').pop() || ''; }
  export function setContent(newContent: string) { content = newContent; savedContent = newContent; isModified = false; }
  export function getContent(): string { return content; }
  export function getFile(): string | null { return filePath; }
  export function getPdfInfo(): { currentPage: number; pageCount: number; filePath: string | null } { return { currentPage: pdfCurrentPage, pageCount: pdfPageCount, filePath }; }
  export function setPdfPage(page: number) { if (filePath && isPdfFile(filePath) && page >= 0 && page < pdfPageCount) { loadPdfPage(filePath, page); } }
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
    <div class="preview-with-toc" bind:this={previewWithToc} class:hidden={!filePath && !batchRenameTempPath} class:modeHidden={mode !== 'global-normal'}>
      <div class="preview-area" bind:this={previewArea} aria-hidden="true"></div>
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
    <div class="editor-area" bind:this={editorContainer} class:hidden={!filePath && !batchRenameTempPath} class:activeEditor={mode === 'editor-normal' || mode === 'editor-insert'} onclick={(e) => e.stopPropagation()} onmouseup={() => {
      // Re-focus overlay after mouse interactions (selection, click) pass through to CodeMirror.
      // pointer-events:none on the overlay lets mouse events reach CodeMirror, which steals focus.
      if (mode === 'editor-normal') {
        requestAnimationFrame(() => {
          if (overlayElement && mode === 'editor-normal' && activeColumn === 'preview') {
            focusActiveEditor();
          }
        });
      }
    }}>
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
    {#if !filePath && !batchRenameTempPath}
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

  {#if outputVisible}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="panel-output"
      tabindex="0"
      onkeydown={handleOutputKeydown}
      onclick={(e) => e.stopPropagation()}
    >
      <div class="output-content">
        <pre class="output-text">{outputText}</pre>
      </div>
      <div class="output-footer">
        <span>Press ENTER to continue</span>
        {#if outputExitCode !== 0}
          <span class="output-exitcode">exit: {outputExitCode}</span>
        {/if}
      </div>
    </div>
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

  .modified-indicator { color: var(--warning); font-size: 12px; }

  .mode-indicator {
    font-size: 10px;
    padding: 1px 6px;
    background-color: var(--accent);
    color: var(--bg-primary);
    margin-left: auto;
  }
  .mode-indicator.normal { background-color: var(--success); }
  .mode-indicator.insert { background-color: var(--warning); color: var(--bg-primary); }
  .mode-indicator.toc { background-color: var(--success); }

  .toc-toggle {
    background: none; border: 1px solid var(--border); cursor: pointer;
    font-size: 10px; padding: 1px 6px; color: var(--text-muted);
    font-family: var(--font-mono); margin-left: 2px;
  }
  .toc-toggle:hover { color: var(--text-primary); border-color: var(--text-muted); }
  .toc-toggle:not(.closed) { color: var(--bg-primary); background-color: var(--accent); border-color: var(--accent); }
  .toc-toggle.closed { color: var(--text-muted); }

  .layout-toggle {
    background: none; border: 1px solid var(--border); cursor: pointer;
    padding: 2px 4px; display: flex; align-items: center; justify-content: center;
    color: var(--text-muted); margin-left: auto;
    transition: color 0.15s ease, border-color 0.15s ease;
  }
  .layout-toggle:hover { color: var(--text-primary); border-color: var(--accent); }
  .layout-toggle.expanded { color: var(--accent); border-color: var(--accent); }

  .panel-content { flex: 1; overflow: hidden; position: relative; }

  .preview-with-toc { display: flex; width: 100%; height: 100%; }
  .preview-with-toc.modeHidden { display: none; }
  .preview-with-toc.hidden { display: none; }

  .preview-area { flex: 1; min-width: 0; height: 100%; overflow: hidden; position: relative; }

  :global(.tab-preview-slot) {
    position: absolute; inset: 0; overflow: auto;
    padding: 12px; display: flex; flex-direction: column; box-sizing: border-box;
    background: var(--bg-primary);
    font-family: var(--font-mono);
  }

  .editor-area { width: 100%; height: 100%; display: none; position: relative; }
  .editor-area.hidden { display: none; }
  .editor-area.activeEditor:not(.hidden) { display: block; }
  :global(.editor-session) { width: 100%; height: 100%; }

  .editor-overlay {
    position: absolute; top: 0; left: 0; width: 100%; height: 100%;
    z-index: 10; background: transparent; outline: none;
    pointer-events: none;
  }
  .editor-overlay.overlay-hidden { display: none; }

  .panel-cmdline {
    padding: 4px 12px; background-color: var(--bg-secondary);
    border-top: 1px solid var(--border); font-family: var(--font-mono);
    font-size: 13px; color: var(--text-primary);
  }

  .welcome {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 100%; text-align: center; color: var(--text-muted);
    max-width: 400px; margin: 0 auto; font-family: var(--font-mono);
  }
  .welcome h2 { color: var(--text-primary); margin-bottom: 8px; }
  .welcome p { margin-bottom: 24px; }
  .shortcuts { text-align: left; background-color: var(--bg-secondary); padding: 16px; width: 100%; border: 1px solid var(--border); }
  .shortcuts h3 { color: var(--text-primary); margin-bottom: 12px; font-size: 13px; }
  .shortcuts ul { list-style: none; padding: 0; }
  .shortcuts li { padding: 3px 0; font-size: 12px; color: var(--text-secondary); }
  .shortcuts kbd { background-color: var(--bg-tertiary); padding: 1px 4px; font-family: var(--font-mono); font-size: 11px; border: 1px solid var(--border); }

  :global(.cm-editor) { height: 100%; font-family: var(--font-mono); }
  :global(.cm-editor ::selection) { background-color: var(--bg-active); color: var(--text-primary); }
  :global(.cm-panel) { display: none !important; }
  :global(.cm-searchMatch) { background-color: #fabd2f55 !important; outline: 1px solid #fabd2f88 !important; border-radius: 2px; }
  :global(.cm-searchMatch-selected) { background-color: #fabd2faa !important; outline: 1px solid #fabd2fcc !important; border-radius: 2px; }
  :global(.cm-sMatch) { background-color: #b8bb2644; outline: 1px solid #b8bb2688; border-radius: 2px; }
  :global(.cm-sMatch-replace) { background-color: #b8bb2644; outline: 1px solid #b8bb2688; border-radius: 2px; font-size: 0; color: transparent; }
  :global(.cm-sMatch-replace::after) { content: var(--s-replacement, ''); font-size: initial; color: #83a598; font-style: italic; }

  :global(.dir-list) { font-family: var(--font-mono); font-size: 13px; min-width: 0; }
  :global(.dir-entry) { display: flex; align-items: center; padding: 2px 0; gap: 6px; min-width: 0; }
  :global(.entry-name) { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--file-color); }
  :global(.entry-name.is-dir) { color: var(--dir-color); font-weight: 500; }
  :global(.entry-size) { flex-shrink: 0; color: var(--text-muted); font-size: 12px; min-width: 60px; text-align: right; }
  :global(.entry-size.dir) { visibility: hidden; }

  :global(.archive-header) { display: flex; align-items: center; gap: 8px; padding: 8px 0 12px; border-bottom: 1px solid var(--border); margin-bottom: 8px; }
  :global(.archive-icon) { font-size: 16px; }
  :global(.archive-name) { font-weight: 600; color: var(--text-primary); font-size: 14px; }
  :global(.archive-meta) { color: var(--text-muted); font-size: 12px; margin-left: auto; }

  :global(.preview-empty) { color: var(--text-muted); text-align: center; padding: 24px; font-size: 13px; }
  :global(.preview-unsupported) { color: var(--text-muted); text-align: center; padding: 24px; font-size: 13px; }
  :global(.preview-code) { white-space: pre-wrap; word-break: break-all; }
  :global(.preview-code pre) { white-space: pre-wrap; word-break: break-all; }
  :global(.preview-plain) { white-space: pre-wrap; word-break: break-all; }
  :global(.preview-hex) { font-family: var(--font-mono); font-size: 13px; }
  :global(.hex-notice) { padding: 8px 12px; background: #3c3836; color: #d79921; font-size: 12px; margin-bottom: 8px; border-radius: 4px; }
  :global(.hex-dump) { margin: 0; line-height: 1.5; color: var(--text-secondary); white-space: pre-wrap; word-break: break-all; }
  :global(.hex-dump code) { font-family: var(--font-mono); font-size: 12px; }

  :global(.preview-image) { display: flex; align-items: center; justify-content: center; flex: 1; min-height: 0; }
  :global(.image-info-bar) { display: flex; align-items: center; justify-content: space-between; padding: 4px 12px; background-color: var(--bg-secondary); border-top: 1px solid var(--border); color: var(--text-muted); font-size: 11px; flex-shrink: 0; }
  :global(.image-view-original) { background: none; border: 1px solid var(--border); color: var(--text-primary); padding: 1px 6px; cursor: pointer; font-size: 11px; font-family: var(--font-mono); }
  :global(.image-view-original:hover) { background-color: var(--bg-hover); }
  :global(.image-view-original:disabled) { opacity: 0.5; cursor: default; }

  :global(.preview-pdf-page) { display: flex; align-items: center; justify-content: center; flex: 1; min-height: 0; }
  :global(.pdf-info-bar) { display: flex; align-items: center; justify-content: space-between; }

  /* Markdown Preview — Terminal style */
  :global(.preview-markdown) { font-family: var(--font-mono); font-size: 14px; line-height: 1.7; color: var(--text-primary); }

  :global(.preview-markdown h1), :global(.preview-markdown h2), :global(.preview-markdown h3),
  :global(.preview-markdown h4), :global(.preview-markdown h5), :global(.preview-markdown h6) {
    font-family: var(--font-mono); font-weight: 700; line-height: 1.3;
    margin-top: 1.6em; margin-bottom: 0.6em; color: var(--accent);
  }
  :global(.preview-markdown h1) { font-size: 1.8em; padding-bottom: 0.3em; border-bottom: 2px solid var(--border); }
  :global(.preview-markdown h2) { font-size: 1.5em; padding-bottom: 0.25em; border-bottom: 1px solid var(--border); }
  :global(.preview-markdown h3) { font-size: 1.25em; }
  :global(.preview-markdown h4) { font-size: 1.1em; color: var(--text-secondary); }
  :global(.preview-markdown h5) { font-size: 1em; color: var(--text-secondary); }
  :global(.preview-markdown h6) { font-size: 0.9em; color: var(--text-muted); }
  :global(.preview-markdown h1:first-child), :global(.preview-markdown h2:first-child),
  :global(.preview-markdown h3:first-child) { margin-top: 0; }

  :global(.preview-markdown p) { margin-top: 0; margin-bottom: 1em; }
  :global(.preview-markdown strong) { font-weight: 700; color: var(--text-primary); }
  :global(.preview-markdown em) { font-style: italic; color: var(--text-primary); }
  :global(.preview-markdown del) { text-decoration: line-through; color: var(--text-muted); }
  :global(.preview-markdown a) { color: var(--accent); text-decoration: none; border-bottom: 1px dashed var(--accent); transition: border-bottom-style 0.15s; }
  :global(.preview-markdown a:hover) { border-bottom-style: solid; }
  :global(.preview-markdown hr) { border: none; border-top: 1px dashed var(--border); margin: 2em 0; }
  :global(.preview-markdown ul), :global(.preview-markdown ol) { padding-left: 2em; margin-bottom: 1em; }
  :global(.preview-markdown li) { margin-bottom: 0.3em; }
  :global(.preview-markdown li > p) { margin-bottom: 0.3em; }
  :global(.preview-markdown ul.contains-task-list) { list-style: none; padding-left: 0.5em; }
  :global(.preview-markdown .task-list-item) { display: flex; align-items: baseline; gap: 0.5em; }
  :global(.preview-markdown .task-list-item input[type="checkbox"]) {
    appearance: none; -webkit-appearance: none; width: 14px; height: 14px;
    border: 1px solid var(--border); background: var(--bg-primary);
    cursor: default; flex-shrink: 0; position: relative; top: 2px;
  }
  :global(.preview-markdown .task-list-item input[type="checkbox"]:checked) { background: var(--accent); border-color: var(--accent); }
  :global(.preview-markdown .task-list-item input[type="checkbox"]:checked::after) { content: '✓'; position: absolute; top: -2px; left: 1px; font-size: 11px; color: var(--bg-primary); font-weight: bold; }
  :global(.preview-markdown blockquote) { margin: 1em 0; padding: 0.5em 1em; border-left: 3px solid var(--accent); background: var(--bg-secondary); color: var(--text-secondary); }
  :global(.preview-markdown blockquote p:last-child) { margin-bottom: 0; }
  :global(.preview-markdown code) { font-family: var(--font-mono); font-size: 0.9em; padding: 0.15em 0.4em; background: var(--bg-tertiary); border-radius: 3px; color: var(--warning); }
  :global(.preview-markdown pre) { margin: 1em 0; border-radius: 4px; white-space: pre-wrap; word-break: break-all; border: 1px solid var(--border); }
  :global(.preview-markdown pre code) { display: block; padding: 1em; font-size: 0.85em; line-height: 1.6; background: none; color: inherit; border-radius: 0; background-color: transparent !important; }
  :global(.preview-markdown table) { width: 100%; border-collapse: collapse; margin: 1em 0; font-size: 0.9em; }
  :global(.preview-markdown thead th) { background: var(--bg-secondary); font-weight: 600; text-align: left; padding: 0.6em 0.8em; border: 1px solid var(--border); }
  :global(.preview-markdown tbody td) { padding: 0.5em 0.8em; border: 1px solid var(--border); }
  :global(.preview-markdown tbody tr:nth-child(even)) { background: var(--bg-secondary); }
  :global(.preview-markdown tbody tr:hover) { background: var(--bg-hover); }
  :global(.preview-markdown img) { max-width: 100%; border-radius: 4px; margin: 0.5em 0; }
  :global(.preview-markdown dl) { margin: 1em 0; }
  :global(.preview-markdown dt) { font-weight: 700; margin-top: 0.5em; }
  :global(.preview-markdown dd) { margin-left: 2em; color: var(--text-secondary); }
  :global(.preview-markdown .math-placeholder) { font-family: var(--font-mono); font-style: italic; color: var(--text-muted); background: var(--bg-tertiary); padding: 0.1em 0.3em; border-radius: 3px; border: 1px dashed var(--border); }
  :global(.preview-markdown .katex-display) { margin: 1em 0; overflow-x: visible; text-align: center; }
  :global(.preview-markdown eq), :global(.preview-markdown eqn) { display: inline; }
  :global(.preview-markdown section.eqno), :global(.preview-markdown section:not(.eqno)) { display: block; text-align: center; margin: 1em 0; }
  :global(.preview-markdown .mermaid-container) { margin: 1em 0; text-align: center; overflow-x: auto; }
  :global(.preview-markdown .mermaid-container svg) { max-width: 100%; height: auto; }
  :global(.preview-markdown pre.mermaid-error) { border-left: 3px solid var(--error, #e74c3c); }

  /* Output panel */
  .panel-output {
    display: flex;
    flex-direction: column;
    max-height: 200px;
    background-color: var(--bg-primary);
    border-top: 2px solid var(--accent);
  }
  .output-content {
    flex: 1;
    overflow: auto;
    padding: 8px 12px;
    min-height: 0;
  }
  .output-text {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    user-select: text;
    cursor: text;
  }
  .output-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }
  .output-exitcode {
    color: var(--warning);
  }

  /* Notebook Preview (.ipynb) */
  :global(.preview-ipynb) {
    font-family: var(--font-mono);
  }
  :global(.ipynb-meta) {
    font-size: 11px;
    color: var(--text-muted);
    padding: 4px 0 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 12px;
  }
  :global(.ipynb-cell) {
    margin-bottom: 4px;
    border: 1px solid var(--border);
    background-color: var(--bg-secondary);
  }
  :global(.ipynb-cell-header) {
    padding: 2px 8px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
  }
  :global(.ipynb-badge) {
    font-size: 10px;
    padding: 0 5px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--bg-primary);
  }
  :global(.ipynb-badge-markdown) { background-color: #458588; }
  :global(.ipynb-badge-code) { background-color: var(--accent); }
  :global(.ipynb-badge-raw) { background-color: var(--text-muted); }
  :global(.ipynb-cell-body) {
    padding: 8px 12px;
    color: var(--text-primary);
    font-size: 13px;
    line-height: 1.6;
  }
  :global(.ipynb-cell-body h1), :global(.ipynb-cell-body h2), :global(.ipynb-cell-body h3),
  :global(.ipynb-cell-body h4), :global(.ipynb-cell-body h5), :global(.ipynb-cell-body h6) {
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--accent);
    margin: 0.6em 0 0.3em;
  }
  :global(.ipynb-cell-body h1) { font-size: 1.4em; }
  :global(.ipynb-cell-body h2) { font-size: 1.25em; }
  :global(.ipynb-cell-body h3) { font-size: 1.1em; }
  :global(.ipynb-cell-body h1:first-child), :global(.ipynb-cell-body h2:first-child),
  :global(.ipynb-cell-body h3:first-child) { margin-top: 0; }
  :global(.ipynb-cell-body p) { margin: 0.4em 0; }
  :global(.ipynb-cell-body code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    padding: 0.1em 0.3em;
    background: var(--bg-tertiary);
    border-radius: 2px;
    color: var(--warning);
  }
  :global(.ipynb-cell-body pre) {
    margin: 0.4em 0;
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow-x: auto;
  }
  :global(.ipynb-cell-body pre code) {
    padding: 0;
    background: none;
    color: var(--text-primary);
    font-size: 0.85em;
    line-height: 1.5;
  }
  :global(.ipynb-cell-body ul), :global(.ipynb-cell-body ol) {
    padding-left: 1.5em;
    margin: 0.3em 0;
  }
  :global(.ipynb-cell-body a) {
    color: var(--accent);
    text-decoration: none;
  }
  :global(.ipynb-cell-body table) {
    border-collapse: collapse;
    width: 100%;
    margin: 0.4em 0;
    font-size: 0.9em;
  }
  :global(.ipynb-cell-body th), :global(.ipynb-cell-body td) {
    padding: 4px 8px;
    border: 1px solid var(--border);
    text-align: left;
  }
  :global(.ipynb-cell-body th) { background: var(--bg-tertiary); }
  :global(.ipynb-cell-body img) { max-width: 100%; }
  :global(.ipynb-cell-body blockquote) {
    margin: 0.4em 0;
    padding: 4px 12px;
    border-left: 3px solid var(--accent);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }
  :global(.ipynb-cell-input) {
    padding: 4px 8px 0;
  }
  :global(.ipynb-exec-label) {
    font-size: 10px;
    color: var(--text-muted);
    font-weight: 600;
  }
  :global(.ipynb-code) {
    margin-top: 2px;
  }
  :global(.ipynb-code .shiki) {
    border-radius: 3px;
    border: 1px solid var(--border);
    padding: 8px 12px;
    font-size: 0.85em;
    line-height: 1.5;
    overflow-x: auto;
  }
  :global(.ipynb-code .shiki code) {
    font-family: var(--font-mono);
    counter-reset: step;
  }
  :global(.ipynb-outputs) {
    border-top: 1px solid var(--border);
    padding: 4px 8px;
  }
  :global(.ipynb-output) {
    margin-bottom: 4px;
  }
  :global(.ipynb-output .ipynb-exec-label) {
    display: block;
    margin-bottom: 2px;
  }
  :global(.ipynb-output pre) {
    margin: 2px 0;
    padding: 6px 10px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    font-size: 0.85em;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    overflow-x: auto;
  }
  :global(.ipynb-output-img) {
    max-width: 100%;
    border-radius: 3px;
    border: 1px solid var(--border);
  }
  :global(.ipynb-output-error pre) {
    color: var(--error, #e74c3c);
  }
  :global(.ipynb-error-banner) {
    padding: 8px 12px;
    background: var(--error, #e74c3c);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 8px;
  }
  :global(.ipynb-cell-raw pre) {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-secondary);
    font-size: 0.9em;
  }

  /* JSON Preview */
  :global(.preview-json) {
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
  }
  :global(.json-toggle) {
    cursor: pointer;
    user-select: none;
    color: var(--text-muted);
    margin-right: 2px;
  }
  :global(.json-toggle:hover) { color: var(--accent); }
  :global(.json-content.collapsed) { display: none; }
  :global(.json-key) { color: var(--accent); }
  :global(.json-string) { color: #b8bb26; }
  :global(.json-number) { color: #fe8019; }
  :global(.json-boolean) { color: #d3869b; }
  :global(.json-null) { color: #928374; }
  :global(.json-bracket) { color: var(--text-secondary); }
  :global(.json-item) { padding-left: 20px; border-left: 1px solid var(--border); }
</style>
