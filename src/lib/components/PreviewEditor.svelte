<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy, tick, untrack } from 'svelte';
  import { get } from 'svelte/store';
  import { layout } from '$lib/stores/layout';
  import type { VideoMeta, TocHeading } from '$lib/previewers';
  import { isTextFile, isImageFile, isPdfFile, isVideoFile, isArchiveFile } from '$lib/utils/file-types';
  import { EditorView } from 'codemirror';
  import { closeSearchPanel } from '@codemirror/search';
  import type { ClipboardBridge } from '$lib/utils/clipboard-bridge';
  import { createPdfState } from '$lib/composables/pdf-state.svelte';
  import { createMarkdownTocState } from '$lib/composables/markdown-toc-state.svelte';
  import { createFileWatcher } from '$lib/composables/file-watcher.svelte';
  import PdfPreviewPanel from './PdfPreviewPanel.svelte';
  import PdfTocSidebar from './PdfTocSidebar.svelte';
  import { getEditorIndentPolicy } from '$lib/utils/editor-indent-policy';
  import { handleInsertModeShiftTab, handleInsertModeTab } from '$lib/utils/editor-text-keys';
  import {
    type TextContentSnapshot,
    TabCacheManager,
    collectExpandedLines,
    restoreExpandedLines,
  } from '$lib/utils/tab-cache';
  import { isDirectEditorFile } from '$lib/utils/file-loader';
  import {
    loadArchiveDirectory, loadArchiveFile, loadDirectory,
    loadImage, loadVideo, loadTextOrBinary,
  } from '$lib/utils/file-loaders';
  import TextEditorHost from './TextEditorHost.svelte';
  import VimOverlay from './VimOverlay.svelte';
  import PreviewPane from './PreviewPane.svelte';
  import type { FileEntry } from '$lib/types/file-explorer';

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
    selectedEntryIsDir = null,
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
    selectedEntryIsDir?: boolean | null;
  } = $props();

  // During t+n/t+p the selected tab is previewed before activeTabId commits.
  // Keep preview state keyed by that target tab instead of the origin tab.
  let renderTabId = $derived(previewTabId ?? currentTabId);

  const pdfState = createPdfState({
    getFilePath: () => filePath,
    onToast,
  });

  const tocState = createMarkdownTocState({
    getPanelElement: () => panelElement,
  });

  const fileWatcher = createFileWatcher({
    getFilePath: () => filePath,
    getMode: () => mode,
    getRenderTabId: () => renderTabId,
    onFileChanged: async (changedPath: string) => {
      if (!filePath) return;
      const a = changedPath.replace(/\//g, '\\').toLowerCase();
      const b = filePath.replace(/\//g, '\\').toLowerCase();
      if (a !== b) return;
      if (mode !== 'global-normal') return;
      tabCache.delete(renderTabId);
      previewPane?.clearActiveSlot();
      await loadFile(filePath);
    },
  });

  let content: string = $state('');
  let savedContent: string = $state('');
  let binaryContent: ArrayBuffer | null = $state(null);
  let thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null = $state(null);
  let videoMeta: VideoMeta | null = $state(null);
  let originalFileSize: number = $state(0);
  let isModified: boolean = $state(false);
  let mode: 'global-normal' | 'editor-normal' | 'editor-insert' = $state('global-normal');
  let panelElement: HTMLElement | undefined = $state(undefined);
  let editorView: EditorView | undefined = $state(undefined);
  let editorFilePath: string | null = $state(null);
  let archiveEditPath: string | null = $state(null);
  let archiveEditInternalPath: string | null = $state(null);
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
  let textEditorHost: TextEditorHost | undefined = $state(undefined);
  let vimOverlay: VimOverlay | undefined = $state(undefined);
  let previewPane: PreviewPane | undefined = $state(undefined);
  let renderTrigger: number = $state(0);
  let editorInitInFlight: boolean = false;
  let suppressModeEffect: boolean = false;

  function destroyEditorSession(tabId: number): void {
    textEditorHost?.destroySession(tabId);
  }

  function activateEditorSession(tabId: number, path: string): boolean {
    return textEditorHost?.activateSession(tabId, path) ?? false;
  }

  function hideEditorSessions(): void {
    textEditorHost?.hideAllSessions();
  }

  function focusActiveEditor(): void {
    // Focus VimOverlay's overlay which has the keydown handler,
    // NOT TextEditorHost's overlay (which has no keydown handler).
    if (mode === 'editor-normal') {
      const overlay = vimOverlay?.getOverlayElement();
      if (overlay) {
        overlay.focus({ preventScroll: true });
        return;
      }
    }
    textEditorHost?.focusActiveEditor();
  }

  export function focusActiveInput(): void {
    if (!vimOverlay?.getOutputVisible()) focusActiveEditor();
  }

  export function prepareTabFocus(
    tabId: number,
    path: string | null,
    targetMode: 'editor-normal' | 'editor-insert' | 'global-normal',
  ): boolean {
    if (!path || targetMode === 'global-normal') return false;
    // Suppress mode effect to prevent infinite loop when mode actually changes:
    // activateEditorSession updates editorView → triggers mode effect → calls activateEditorSession again
    const willChangeMode = mode !== targetMode;
    if (willChangeMode) suppressModeEffect = true;
    if (!activateEditorSession(tabId, path)) {
      suppressModeEffect = false;
      return false;
    }
    mode = targetMode;
    // suppressModeEffect is cleared by the mode effect (only set if mode actually changed)
    focusActiveEditor();
    return true;
  }

  // TOC state managed by tocState composable
  let pendingTocExpanded: Set<number> | null = null;
  let isMarkdown: boolean = $state(false);
  let isDirectory: boolean = $state(false);
  // File will go to editor (independent of content load state) — suppress preview rendering
  let directEdit = $derived(!!filePath && !binaryContent && !isDirectory && isDirectEditorFile(filePath));
  let editorTargetLine: number = -1;
  let pendingEditorPos: number = -1;
  let pendingEditorScrollTop: number = -1;
  let pendingRestoreScrollRatio: number = $state(-1);

  // Per-tab editor state cache
  const tabCache = new TabCacheManager();
  let pendingRestoreScrollTop: number = -1;
  let pendingTocSelectedIndex: number = -1;

  export function clearTabCache(tabId: number) {
    tabCache.delete(tabId);
    previewPane?.clearSlot(tabId);
    destroyEditorSession(tabId);
  }

  export function cacheTabState(tabId: number) {
    if (!filePath) return;
    tabCache.set(tabId, {
      filePath, content, savedContent, binaryContent,
      mode,
      editorCursorPos: editorView?.state.selection.main.head ?? 0,
      editorScrollTop: editorView?.scrollDOM.scrollTop ?? 0,
      previewScrollTop: previewPane?.getScrollTop() ?? 0,
      isModified, ...pdfState.toCacheSnapshot(), fileMtime: fileWatcher.getFileMtime(),
      ...tocState.toCacheSnapshot(() => previewPane?.getTocSelectedIndex() ?? -1),
    });
  }

  export function deactivateTab() {
    // Visual cleanup only: reset mode so the editor is hidden during tab
    // switch. editorView is NOT destroyed here — teardown is loadFile's job.
    // Clear content to prevent the render effect from firing with stale content
    // when mode changes to 'global-normal'. Without this, the render effect would
    // start an async render with old content, clearing the slot and causing a blank
    // flash + scroll position loss when loadFile subsequently sets new content.
    content = '';
    binaryContent = null;
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
      previewScrollTop: previewPane?.getScrollTop() ?? 0,
      isModified, pdfCurrentPage: pdfState.getPdfCurrentPage(),
      tocOpen: tocState.getTocOpen(), tocExpandedLines: collectExpandedLines(tocState.getTocHeadings()),
    };
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

  // Re-focus overlay after mouse selection (mouseup may fire outside editor panel)
  $effect(() => {
    if (mode !== 'editor-normal') return;
    const overlay = vimOverlay?.getOverlayElement();
    if (!overlay) return;
    function handleDocMouseUp() {
      requestAnimationFrame(() => {
        const ov = vimOverlay?.getOverlayElement();
        if (ov && mode === 'editor-normal' && activeColumn === 'preview' && document.activeElement !== ov) {
          focusActiveEditor();
        }
      });
    }
    document.addEventListener('mouseup', handleDocMouseUp);
    return () => document.removeEventListener('mouseup', handleDocMouseUp);
  });

  $effect(() => {
    const overlay = vimOverlay?.getOverlayElement();
    if (!overlay) return;
    const blockComposition = (event: CompositionEvent) => { event.preventDefault(); event.stopPropagation(); };
    overlay.addEventListener('compositionstart', blockComposition, true);
    overlay.addEventListener('compositionend', blockComposition, true);
    return () => {
      overlay.removeEventListener('compositionstart', blockComposition, true);
      overlay.removeEventListener('compositionend', blockComposition, true);
    };
  });

  // Auto-focus output panel when it becomes visible
  $effect(() => {
    if (vimOverlay?.getOutputVisible()) {
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
      // Reset isDirectory immediately so the auto-render effect doesn't use
      // stale state from a previous tab (e.g. directory → file switch).
      isDirectory = false;
      previewPane?.prepareForLoad();
      loadFile(filePath);
    }
  });

  fileWatcher.setup();

  onDestroy(() => {
    // Editor sessions are destroyed by TextEditorHost's onDestroy
    // Preview resources are destroyed by PreviewPane's onDestroy
    fileWatcher.teardown();
  });

  let prevMode: string = mode;
  $effect(() => {
    const m = mode;
    const changed = m !== prevMode;
    prevMode = m;
    if (m === 'editor-normal' || m === 'editor-insert') {
      // Clear preview slot content to prevent stale preview flash during mode transition
      previewPane?.clearActiveSlot();
      // activateEditorSession/initEditor update editorView state which would
      // re-trigger this effect. Use untrack() to prevent the dependency.
      const activeSession = textEditorHost?.getActiveEditorSession();
      if (activeSession && textEditorHost?.isActiveSession(activeSession)) {
        void textEditorHost?.refreshLayout(activeSession);
      }
      const textSnapshot = readyTextContent;
      if (!textSnapshot || !isCurrentTextSnapshot(textSnapshot)) {
        return;
      }
      if (untrack(() => activateEditorSession(renderTabId, textSnapshot.path))) {
        pendingEditorPos = -1;
        pendingEditorScrollTop = -1;
        if (editorTargetLine >= 0 && editorView) { textEditorHost?.moveCursorToLine(editorTargetLine); editorTargetLine = -1; }
        if (changed && activeColumn === 'preview') focusActiveEditor();
      } else if (!textEditorHost?.hasSession(textSnapshot.tabId) || textEditorHost?.getEditorSession(textSnapshot.tabId)?.filePath !== textSnapshot.path) {
        requestAnimationFrame(() => {
          if (mode !== 'editor-normal' && mode !== 'editor-insert') return;
          if (!isCurrentTextSnapshot(textSnapshot)) return;
          if (!textEditorHost?.hasSession(textSnapshot.tabId) || textEditorHost?.getEditorSession(textSnapshot.tabId)?.filePath !== textSnapshot.path) {
            untrack(() => initEditor(textSnapshot));
            pendingEditorPos = -1;
            pendingEditorScrollTop = -1;
            if (activeColumn === 'preview') focusActiveEditor();
          }
        });
      } else if (editorView && editorTargetLine >= 0 && changed) {
        textEditorHost?.moveCursorToLine(editorTargetLine);
        editorTargetLine = -1;
      }
      if (changed && activeColumn === 'preview' && editorView && textEditorHost?.getEditorFilePath() === filePath) focusActiveEditor();
    } else {
      editorInitInFlight = false; // safety reset
      suppressModeEffect = false; // safety reset
      // Read editor scroll position BEFORE hiding editor container
      // (hidden container has zero layout, making scroll measurements unreliable)
      let editorScrollRatio = -1;
      if (changed && editorView) {
        const scrollDOM = editorView.scrollDOM;
        const scrollHeight = scrollDOM.scrollHeight - scrollDOM.clientHeight;
        if (scrollHeight > 0) {
          editorScrollRatio = scrollDOM.scrollTop / scrollHeight;
        }
      }
      hideEditorSessions();
      if (editorView) closeSearchPanel(editorView);
      // Restore preview scroll position using editor scroll ratio.
      // For all files (markdown, code, etc.), use the editor's current scroll position
      // at exit time. PreviewPane applies the ratio after async render completes.
      if (changed && editorScrollRatio >= 0) {
        pendingRestoreScrollRatio = editorScrollRatio;
      }
      if (changed && codeFileDirectEdit) {
        previewPane?.clearActiveSlot();
      }
      if (changed && panelElement && !tocState.getTocFocused() && activeColumn === 'preview') {
        panelElement.focus();
      }
    }
  });

  export function getMode(): string { return mode; }
  export function getIsModified(): boolean { return isModified; }

  function handlePanelFocus() {
    if (vimOverlay?.getOutputVisible()) return;
    if (mode === 'global-normal' && filePath && isPdfFile(filePath)) {
      pdfState.focusPanel();
      return;
    }
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
    handleInsertModeTab(editorView, { indentPolicy: getEditorIndentPolicy(editorFilePath || filePath) });
  }

  export function pressShiftTab() {
    if (!editorView || mode !== 'editor-insert') return;
    handleInsertModeShiftTab(editorView, { indentPolicy: getEditorIndentPolicy(editorFilePath || filePath) });
  }

  export function isTocVisible(): boolean {
    return mode === 'global-normal' && (
      previewPane?.isTocVisible() === true
      || (isPdfFile(filePath || '') && pdfState.getPdfOutline().length > 0 && pdfState.getPdfTocOpen())
    );
  }

  export function focusToc() {
    if (isPdfFile(filePath || '')) {
      pdfState.focusToc();
      return;
    }
    tocState.setTocFocused(true);
    onToast('Focus: TOC');
    previewPane?.focusToc();
  }
  export function focusContent() {
    tocState.setTocFocused(false);
    pdfState.setTocFocused(false);
    onToast('Focus: PREVIEW');
    if (isPdfFile(filePath || '')) pdfState.focusPanel();
    else if (panelElement) panelElement.focus({ preventScroll: true });
  }
  export function isTocFocused(): boolean { return tocState.getTocFocused() || pdfState.getPdfTocFocused(); }

  // handleTocFocusChange is now tocState.handleTocFocusChange


  async function loadFile(path: string) {
    const gen = ++loadGeneration;
    const fileName = path.split(/[/\\]/).pop() || path;

    // Check if we're in archive mode
    const layoutVal = get(layout);
    const archiveStateVal = layoutVal.archiveState;
    const browsingArchive = archiveStateVal !== null;

    // Track archive context for save operations
    if (browsingArchive && archiveStateVal) {
      archiveEditPath = archiveStateVal.archivePath;
      archiveEditInternalPath = path;
    } else {
      archiveEditPath = null;
      archiveEditInternalPath = null;
    }

    const loadTabId = renderTabId;
    const cached = tabCache.get(loadTabId);
    if (cached && cached.filePath === path && !browsingArchive) {
      if (gen !== loadGeneration) return;
      // Bring cached slot to front FIRST to hide old tab's content immediately.
      // Slot retains its rendered content and scroll position from the previous visit.
      previewPane?.prepareForLoad();
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
      pdfState.fromCacheSnapshot(cached);
      tocState.fromCacheSnapshot(cached);
      fileWatcher.setFileMtime(cached.fileMtime);
      pendingTocSelectedIndex = cached.tocSelectedIndex;
      pendingTocExpanded = new Set(cached.tocExpandedLines);
      pendingRestoreScrollTop = cached.previewScrollTop;
      const ext = path.split('.').pop()?.toLowerCase() || '';
      isMarkdown = ext === 'md' || ext === 'markdown';
      if (!isMarkdown) { tocState.reset(); }
      else if (cached.tocHeadings && cached.tocHeadings.length > 0) {
        if (pendingTocExpanded && pendingTocExpanded.size > 0) {
          restoreExpandedLines(cached.tocHeadings, pendingTocExpanded);
        }
      }
      mode = cached.mode;
      if (cached.mode !== 'global-normal' && cached.editorCursorPos > 0) {
        pendingEditorPos = cached.editorCursorPos;
        pendingEditorScrollTop = cached.editorScrollTop;
      }
      if (!cached.content && !cached.binaryContent && cached.mode === 'global-normal') {
        renderTrigger++;
      }
      // Restore preview scroll position (slot content is already cached).
      if (isPdfFile(path)) {
        // PDF: use double-rAF via applyPendingScroll to wait for PdfPreviewPanel initScale
        pdfState.applyPendingScroll();
      } else if (cached.previewScrollTop > 0) {
        const savedScroll = cached.previewScrollTop;
        requestAnimationFrame(() => {
          const slot = previewPane?.getActiveSlot();
          slot?.scrollTo(0, savedScroll);
        });
      }
      fileWatcher.startWatching(path);
      invoke<{ size: number; modified: number }>('get_file_metadata', { path })
        .then(meta => { if (meta.modified !== cached.fileMtime && (mode === 'global-normal' || !cached.isModified)) { tabCache.delete(loadTabId); if (loadTabId === renderTabId && filePath === path) loadFile(path); } })
        .catch(() => {});
      return;
    }

    // Bring new tab's slot to front FIRST to hide old tab's content immediately.
    previewPane?.prepareForLoad();
    // Reset content so the render $effect does not render stale data from a previous file.
    content = '';
    binaryContent = null;
    readyTextContent = null;
    previewPane?.bumpContentGeneration();
    pendingRestoreScrollTop = -1;
    isModified = false;
    isDirectory = false;
    fileWatcher.setFileMtime(0);
    mode = 'global-normal';
    // Clear slot content for fresh load (no cached content to display).
    previewPane?.clearActiveSlot();

    const ext = path.split('.').pop()?.toLowerCase() || '';
    isMarkdown = ext === 'md' || ext === 'markdown';
    if (!isMarkdown) { tocState.reset(); }

    // Archive file: read from archive
    const checkGen = () => gen === loadGeneration;
    const ctx = { gen, loadTabId, path, archivePath: archiveStateVal?.archivePath ?? null, archiveInternalPath: path, selectedEntryIsDir };

    if (browsingArchive && archiveStateVal) {
      try {
        if (selectedEntryIsDir) {
          const result = await loadArchiveDirectory(ctx, checkGen);
          if ('aborted' in result) return;
          content = ''; binaryContent = null;
          destroyEditorSession(loadTabId);
          mode = 'global-normal';
          previewPane?.prepareForLoad();
          const slot = previewPane?.getActiveSlot();
          if (slot) { slot.dataset.filePath = path; slot.innerHTML = result.html; }
          fileWatcher.stopWatching();
          return;
        }

        const result = await loadArchiveFile(ctx, checkGen);
        if ('aborted' in result) return;
        content = result.content;
        binaryContent = result.binaryContent;
        readyTextContent = result.readyTextContent;
        if (result.isBinary) {
          destroyEditorSession(loadTabId);
          mode = 'global-normal';
          renderTrigger++;
        } else if (readyTextContent && isDirectEditorFile(path)) {
          editorInitInFlight = true;
          if (!activateEditorSession(loadTabId, path)) initEditor(readyTextContent);
          mode = 'editor-normal';
        } else {
          destroyEditorSession(loadTabId);
          mode = 'global-normal';
          renderTrigger++;
        }
        fileWatcher.stopWatching();
        return;
      } catch (e) {
        content = `[Error reading archive file: ${e}]`;
        binaryContent = null; readyTextContent = null;
        destroyEditorSession(loadTabId);
        mode = 'global-normal'; renderTrigger++;
        fileWatcher.stopWatching();
        return;
      }
    }

    // Directory
    try {
      const result = await loadDirectory(ctx, checkGen);
      if (!('aborted' in result)) {
        content = ''; binaryContent = null;
        destroyEditorSession(loadTabId);
        mode = 'global-normal'; isDirectory = true; renderTrigger++;
        fileWatcher.stopWatching();
        return;
      }
    } catch { /* not a directory */ }

    // Binary image
    if (isImageFile(path) && !path.toLowerCase().endsWith('.svg')) {
      const result = await loadImage(ctx, checkGen);
      if ('aborted' in result) return;
      content = result.content; binaryContent = result.binaryContent; thumbnailMeta = result.thumbnailMeta;
      mode = 'global-normal'; renderTrigger++;
      return;
    }

    // PDF
    if (isPdfFile(path)) {
      content = ''; binaryContent = null; mode = 'global-normal';
      previewPane?.clearActiveSlot();
      await pdfState.loadPdfInfo(path, gen);
      return;
    }

    // Archive (top-level)
    if (isArchiveFile(path)) {
      content = ''; binaryContent = null;
      destroyEditorSession(loadTabId);
      mode = 'global-normal'; renderTrigger++;
      previewPane?.renderArchivePreview(path);
      return;
    }

    // Video
    if (isVideoFile(path)) {
      const result = await loadVideo(ctx, checkGen);
      if ('aborted' in result) return;
      videoMeta = result.videoMeta; binaryContent = result.binaryContent; content = result.content;
      mode = 'global-normal'; renderTrigger++;
      return;
    }

    // Text / binary
    {
      const result = await loadTextOrBinary(ctx, checkGen);
      if ('aborted' in result) return;
      content = result.content; binaryContent = result.binaryContent;
      readyTextContent = result.readyTextContent;
      originalFileSize = result.originalFileSize;
      savedContent = result.savedContent;
      fileWatcher.setFileMtime(result.fileMtime);
    }

    // Code files go directly to editor mode (no Shiki preview)
    if (readyTextContent && isDirectEditorFile(path)) {
      editorInitInFlight = true;
      if (!activateEditorSession(loadTabId, path)) initEditor(readyTextContent);
      mode = 'editor-normal';
    } else {
      destroyEditorSession(loadTabId);
      mode = 'global-normal'; renderTrigger++;
    }
    fileWatcher.startWatching(path);
  }

  function scrollPreview(deltaY: number, deltaX: number = 0) {
    previewPane?.scrollPreview(deltaY, deltaX);
  }

  export function getVisibleLine(): number {
    return previewPane?.getVisibleLine() ?? 0;
  }

  function moveCursorToLine(lineNumber: number) {
    textEditorHost?.moveCursorToLine(lineNumber);
    editorTargetLine = -1;
  }

  function scrollEditorToPos(view: EditorView, pos: number) {
    textEditorHost?.scrollEditorToPos(view, pos);
  }

  function initEditor(textSnapshot: TextContentSnapshot) {
    if (!isCurrentTextSnapshot(textSnapshot)) return;
    textEditorHost?.initEditor(textSnapshot);
  }

  let clipboardBridge: ClipboardBridge | null = null;

  function handleKeydown(event: KeyboardEvent) {
    if (mode !== 'global-normal') return;
    // PDF mode: let PdfPreviewPanel handle all non-modifier keys
    if (filePath && isPdfFile(filePath) && !event.ctrlKey && !event.altKey && !event.metaKey) return;
    if (event.ctrlKey && event.code === 'KeyL' && isMarkdown && tocState.getTocHeadings().length > 0) { event.preventDefault(); event.stopPropagation(); focusToc(); return; }
    if (event.code === 'KeyE' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); if (!filePath || !isTextFile(filePath)) { onToast('此文件类型不支持编辑'); return; } if (!content && originalFileSize > 0) { onToast('文件加载中，请稍候'); return; } editorTargetLine = getVisibleLine(); mode = 'editor-normal'; }
    else if (event.code === 'KeyE' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); if (!filePath) { onToast('此文件类型不支持全屏查看'); return; } onFullscreen(); }
    else if (event.code === 'KeyJ' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(40); }
    else if (event.code === 'KeyK' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(-40); }
    else if (event.code === 'KeyH' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(0, -40); }
    else if (event.code === 'KeyL' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); scrollPreview(0, 40); }
    else if (event.code === 'KeyG' && !event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); const ggSlot = previewPane?.getActiveSlot(); if (ggSlot) ggSlot.scrollTop = 0; }
    else if (event.code === 'KeyG' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey) { event.preventDefault(); const gSlot = previewPane?.getActiveSlot(); if (gSlot) gSlot.scrollTop = gSlot.scrollHeight; }
    else if (event.ctrlKey && event.code === 'KeyS') { event.preventDefault(); saveFile(); }
  }

  async function saveFile() {
    if (!filePath || !isModified) return;
    try {
      if (archiveEditPath && archiveEditInternalPath) {
        // Save to archive (ZIP only)
        const encoder = new TextEncoder();
        const bytes = Array.from(encoder.encode(content));
        await invoke('archive_write_file', {
          archivePath: archiveEditPath,
          internalPath: archiveEditInternalPath,
          content: bytes,
        });
      } else {
        await invoke('write_file', { path: filePath, content });
        // Get actual file mtime after save so metadata check doesn't
        // falsely invalidate cached content on tab switch
        const meta = await invoke<{ modified: number }>('get_file_metadata', { path: filePath });
        fileWatcher.setFileMtime(meta.modified);
        const cached = tabCache.get(renderTabId);
        if (cached && cached.filePath === filePath) {
          cached.content = content;
          cached.savedContent = savedContent;
          cached.isModified = false;
          cached.fileMtime = meta.modified;
        }
      }
      savedContent = content; isModified = false;
      const saveSlot = previewPane?.getActiveSlot();
      if (saveSlot) { delete saveSlot.dataset.rendered; }
    } catch (error) { console.error('Failed to save file:', error); }
  }

  function getFileName(): string { if (!filePath) return ''; return filePath.split('\\').pop() || filePath.split('/').pop() || ''; }
  export function setContent(newContent: string) { content = newContent; savedContent = newContent; isModified = false; }
  export function getContent(): string { return content; }
  export function getFile(): string | null { return filePath; }
  export function getPdfInfo(): { currentPage: number; pageCount: number; filePath: string | null } { return { currentPage: pdfState.getPdfCurrentPage(), pageCount: pdfState.getPdfPageCount(), filePath }; }
  export function togglePdfToc() { pdfState.togglePdfToc(); }
  export function jumpToPdfPage(page: number) { pdfState.jumpToPdfPage(page); }
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
      <span class="mode-indicator" class:insert={mode === 'editor-insert'} class:normal={mode === 'editor-normal'} class:toc={mode === 'global-normal' && (tocState.getTocFocused() || pdfState.getPdfTocFocused())}>
      {#if mode === 'global-normal' && (tocState.getTocFocused() || pdfState.getPdfTocFocused())}TOC{:else if mode === 'global-normal'}PREVIEW{:else if mode === 'editor-normal'}NORMAL{:else}INSERT{/if}
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
    {#if isMarkdown && tocState.getTocHeadings().length > 0 && mode === 'global-normal'}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <button
        class="toc-toggle"
        class:closed={!tocState.getTocOpen()}
        onclick={() => { tocState.setTocOpen(!tocState.getTocOpen()); }}
        title={tocState.getTocOpen() ? 'Hide outline' : 'Show outline'}
      >
        ☰
      </button>
    {/if}
    {#if filePath && isPdfFile(filePath) && pdfState.getPdfOutline().length > 0 && mode === 'global-normal'}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <button
        class="toc-toggle"
        class:closed={!pdfState.getPdfTocOpen()}
        onclick={() => pdfState.togglePdfToc()}
        title={pdfState.getPdfTocOpen() ? 'Hide outline' : 'Show outline'}
      >
        ☰
      </button>
    {/if}
  </div>

  <div class="panel-content">
    {#if filePath && isPdfFile(filePath) && mode === 'global-normal'}
      <div class="pdf-with-toc">
        <PdfPreviewPanel
          bind:this={pdfState.pdfPreviewPanel}
          pdfPath={filePath}
          pageDimensions={pdfState.getPdfPageDimensions()}
          pageCount={pdfState.getPdfPageCount()}
          fileSize={pdfState.getPdfFileSize()}
          title={pdfState.getPdfTitle()}
          onPageChange={(page: number) => { pdfState.setPage(page); }}
          onFullscreen={() => onFullscreen()}
        />
        {#if pdfState.getPdfOutline().length > 0 && pdfState.getPdfTocOpen()}
          <PdfTocSidebar
            bind:this={pdfState.pdfTocSidebar}
            outline={pdfState.getPdfOutline()}
            currentPage={pdfState.getPdfCurrentPage()}
            pageDimensions={pdfState.getPdfPageDimensions()}
            onJump={(page, y) => pdfState.jumpToPage(page, y)}
            onFocusChange={(focused) => { pdfState.setTocFocused(focused); }}
            onExit={() => focusContent()}
          />
        {/if}
      </div>
    {/if}
    <div class="preview-pane-wrapper" class:hidden={(!filePath && !batchRenameTempPath) || (filePath && isPdfFile(filePath) && mode === 'global-normal')} class:modeHidden={mode !== 'global-normal'}>
    <PreviewPane
      bind:this={previewPane}
      filePath={filePath}
      content={content}
      binaryContent={binaryContent}
      originalFileSize={originalFileSize}
      thumbnailMeta={thumbnailMeta}
      currentFileMtime={fileWatcher.getFileMtime()}
      isMarkdown={isMarkdown}
      tocHeadings={tocState.getTocHeadings()}
      tocActiveLine={tocState.getTocActiveLine()}
      tocFocused={tocState.getTocFocused()}
      tocOpen={tocState.getTocOpen()}
      pendingTocExpanded={pendingTocExpanded}
      pendingTocSelectedIndex={pendingTocSelectedIndex}
      pendingRestoreScrollTop={pendingRestoreScrollTop}
      renderTabId={renderTabId}
      codeFileDirectEdit={codeFileDirectEdit}
      directEdit={directEdit}
      isDirectory={isDirectory}
      renderTrigger={renderTrigger}
      mode={mode}
      onTocJump={(_line) => {}}
      onTocFocusChange={tocState.handleTocFocusChange}
      onRenderComplete={() => {
        // Apply pending scroll ratio after async render completes.
        // pendingRestoreScrollRatio is set by the mode effect when exiting editor mode.
        if (pendingRestoreScrollRatio >= 0) {
          const ratio = pendingRestoreScrollRatio;
          pendingRestoreScrollRatio = -1;
          requestAnimationFrame(() => {
            const slot = previewPane?.getActiveSlot();
            if (slot) {
              const maxScroll = slot.scrollHeight - slot.clientHeight;
              if (maxScroll > 0) slot.scrollTop = ratio * maxScroll;
            }
          });
        }
      }}
      onTocHeadingsChange={tocState.onHeadingsChange}
      onTocActiveLineChange={tocState.onActiveLineChange}
    />
    </div>
    <TextEditorHost
      bind:this={textEditorHost}
      tabId={renderTabId}
      filePath={filePath}
      content={content}
      savedContent={savedContent}
      mode={mode}
      pendingEditorPos={pendingEditorPos}
      pendingEditorScrollTop={pendingEditorScrollTop}
      editorTargetLine={editorTargetLine}
      activeColumn={activeColumn}
      batchRenameTempPath={batchRenameTempPath}
      isDirectEditorFile={codeFileDirectEdit}
      onModeChange={(m) => { mode = m; }}
      onContentChange={(c) => { content = c; }}
      onSavedContentChange={(c) => { savedContent = c; }}
      onModifiedChange={(m) => { isModified = m; }}
      onClipboardBridgeChange={(b) => { clipboardBridge = b; }}
      onEditorViewChange={(v) => { editorView = v; }}
      onSaveFile={() => saveFile()}
      onToast={(msg) => onToast(msg)}
      onBatchRenameSave={onBatchRenameSave}
      onBatchRenameCancel={onBatchRenameCancel}
    />
    <VimOverlay
      bind:this={vimOverlay}
      mode={mode}
      editorView={editorView}
      clipboardBridge={clipboardBridge}
      filePath={filePath}
      content={content}
      savedContent={savedContent}
      isModified={isModified}
      batchRenameTempPath={batchRenameTempPath}
      onModeChange={(m) => { mode = m; }}
      onContentChange={(c) => { content = c; }}
      onSavedContentChange={(c) => { savedContent = c; }}
      onModifiedChange={(m) => { isModified = m; }}
      onToast={(msg) => onToast(msg)}
      onBatchRenameSave={onBatchRenameSave}
      onBatchRenameCancel={onBatchRenameCancel}
      onSaveFile={() => saveFile()}
    />
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

  .preview-pane-wrapper { width: 100%; height: 100%; }
  .preview-pane-wrapper.modeHidden { display: none; }
  .preview-pane-wrapper.hidden { display: none; }

  .pdf-with-toc { display: flex; width: 100%; height: 100%; }

  :global(.editor-session) { width: 100%; height: 100%; }

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

	/* Frontmatter block */
	:global(.preview-markdown .frontmatter-block) {
		margin-bottom: 1.5em;
		padding-bottom: 0.8em;
		border-bottom: 1px dashed var(--border);
		font-size: 0.82em;
		line-height: 1.6;
		opacity: 0.75;
	}
	:global(.preview-markdown .fm-row) {
		/* compact single line */
	}
	:global(.preview-markdown .fm-key) {
		color: var(--accent);
		font-weight: 500;
	}
	:global(.preview-markdown .fm-sep) {
		color: var(--text-muted);
		margin: 0 0.3em;
	}
	:global(.preview-markdown .fm-value) {
		color: var(--text-secondary);
	}

	/* Indented table wrapper */
	:global(.preview-markdown .table-indent-wrapper) {
		display: block;
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
