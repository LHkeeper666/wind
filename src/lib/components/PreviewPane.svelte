<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { PreviewRouter } from '$lib/previewers';
  import type { TocHeading } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import TocSidebar from './TocSidebar.svelte';
  import { restoreExpandedLines } from '$lib/utils/tab-cache';
  import { isDirectEditorFile } from '$lib/utils/file-types';

  let {
    filePath = null,
    content = '',
    binaryContent = null,
    originalFileSize = 0,
    thumbnailMeta = null as { width: number; height: number; originalSize: number; isThumbnail: boolean } | null,
    currentFileMtime = 0,
    isMarkdown = false,
    tocHeadings = [] as TocHeading[],
    tocActiveLine = -1,
    tocFocused = false,
    tocOpen = true,
    pendingTocExpanded = null as Set<number> | null,
    pendingTocSelectedIndex = -1,
    pendingRestoreScrollTop = -1,
    renderTabId = 0,
    codeFileDirectEdit = false,
    directEdit = false,
    isDirectory = false,
    renderTrigger = 0,
    mode = 'global-normal' as 'global-normal' | 'editor-normal' | 'editor-insert',
    onTocJump = (line: number) => {},
    onTocFocusChange = (focused: boolean) => {},
    onRenderComplete = () => {},
    onTocHeadingsChange = (_headings: TocHeading[]) => {},
    onTocActiveLineChange = (_line: number) => {},
  }: {
    filePath: string | null;
    content: string;
    binaryContent: ArrayBuffer | null;
    originalFileSize: number;
    thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null;
    currentFileMtime: number;
    isMarkdown: boolean;
    tocHeadings: TocHeading[];
    tocActiveLine: number;
    tocFocused: boolean;
    tocOpen: boolean;
    pendingTocExpanded: Set<number> | null;
    pendingTocSelectedIndex: number;
    pendingRestoreScrollTop: number;
    renderTabId: number;
    codeFileDirectEdit: boolean;
    directEdit: boolean;
    isDirectory: boolean;
    renderTrigger: number;
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    onTocJump?: (line: number) => void;
    onTocFocusChange?: (focused: boolean) => void;
    onRenderComplete?: () => void;
    onTocHeadingsChange?: (headings: TocHeading[]) => void;
    onTocActiveLineChange?: (line: number) => void;
  } = $props();

  let previewArea: HTMLElement | undefined = $state(undefined);
  let previewWithToc: HTMLElement | undefined = $state(undefined);
  let tocSidebar: TocSidebar | undefined = $state(undefined);
  let previewRouter: PreviewRouter | undefined;
  let directoryPreviewer: DirectoryPreviewer | undefined;
  let _renderRequestId: number = 0;
  let _contentGeneration: number = 0;
  let renderInFlight: boolean = false;
  let renderQueued: boolean = false;
  let scrollObserver: IntersectionObserver | undefined;

  // Track isDirectory staleness: when filePath changes, isDirectory may be stale
  // from a previous tab. Only render directory preview if isDirectory was set for THIS path.
  let _lastIsDirPath: string = '';
  let _lastIsDirValue: boolean = false;

  // Track content staleness for binary files (images/videos).
  // When filePath changes, binaryContent may still hold the previous file's data.
  // Skip render until binaryContent is cleared and reloaded with fresh data.
  let _lastBinaryContentRef: ArrayBuffer | null = null;

  // Per-tab persistent preview slots
  const tabSlots = new Map<number, HTMLDivElement>();

  export function getActiveSlot(): HTMLDivElement | undefined {
    return tabSlots.get(renderTabId);
  }

  export function getOrCreateSlot(tabId: number): HTMLDivElement {
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

  export function showTabSlot(tabId: number) {
    for (const [id, slot] of tabSlots) {
      const newZ = id === tabId ? '1' : '0';
      slot.style.zIndex = newZ;
    }
  }

  export function clearSlot(tabId: number) {
    const slot = tabSlots.get(tabId);
    if (slot) {
      slot.remove();
      tabSlots.delete(tabId);
    }
  }

  export function clearAllSlots() {
    for (const slot of tabSlots.values()) {
      slot.remove();
    }
    tabSlots.clear();
  }

  export function bumpContentGeneration() {
    _contentGeneration++;
  }

  export function clearActiveSlot() {
    const slot = tabSlots.get(renderTabId);
    if (slot) {
      slot.innerHTML = '';
      delete slot.dataset.rendered;
      delete slot.dataset.filePath;
      delete slot.dataset.fileMtime;
    }
  }

  export function getScrollTop(): number {
    return tabSlots.get(renderTabId)?.scrollTop ?? 0;
  }

  export function prepareForLoad() {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
  }

  export function getPreviewArea(): HTMLElement | undefined {
    return previewArea;
  }

  function getPreviewRouter(): PreviewRouter {
    if (!previewRouter) {
      previewRouter = new PreviewRouter();
      previewRouter.onHeadings = (headings: TocHeading[]) => {
        tocHeadings = headings;
        tocActiveLine = -1;
        onTocHeadingsChange(headings);
        onTocActiveLineChange(-1);
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

  function requestTabRender(tabId: number): number {
    return (tabSlots.get(tabId)?.dataset.renderVersion ? parseInt(tabSlots.get(tabId)!.dataset.renderVersion!) : 0) + 1;
  }

  function isCurrentTabRender(tabId: number, path: string, version: number): boolean {
    const slot = tabSlots.get(tabId);
    return slot?.dataset.renderVersion === String(version) && slot?.dataset.filePath === path;
  }

  // Auto-render effect: triggers when content/mode/tab changes
  let _prevRenderKey: string = '';
  let _prevFilePathForContent: string | null = null;
  $effect(() => {
    const contentFingerprint = content ? `${content.length}:${content.slice(0, 80)}` : '';
    const key = `${renderTrigger}:${mode}:${filePath}:${renderTabId}:${codeFileDirectEdit}:${directEdit}:${isDirectory}:${contentFingerprint}:${!!binaryContent}`;
    if (key === _prevRenderKey) return;
    _prevRenderKey = key;

    // Track which path isDirectory was set for.
    // When filePath changes (tab switch), isDirectory may be stale from a different tab.
    // We record the path that isDirectory=true was associated with so we can detect staleness.
    if (isDirectory !== _lastIsDirValue) {
      _lastIsDirPath = isDirectory ? (filePath ?? '') : '';
      _lastIsDirValue = isDirectory;
    }

    // When filePath changes, reset binary content staleness tracker.
    // binaryContent may still hold data from the previous file;
    // skip rendering until binaryContent is cleared and reloaded with fresh data.
    if (filePath !== _prevFilePathForContent) {
      _prevFilePathForContent = filePath;
      _lastBinaryContentRef = null;
    }

    if (mode !== 'global-normal') return;
    if (directEdit) return; // Code files go directly to editor, never render preview
    if (!previewArea || !filePath) return;

    if (isDirectory) {
      // Guard: only render directory preview if isDirectory was set for THIS filePath.
      // Prevents stale isDirectory=true from a previous tab triggering a directory render
      // for a file path that isn't actually a directory.
      if (filePath !== _lastIsDirPath) return;
      void renderDirectoryPreview();
    } else if (codeFileDirectEdit && content) {
      renderSimpleCodePreview();
    } else if (content || binaryContent) {
      // Guard: if filePath changed but binaryContent is still from the previous file,
      // skip render to prevent stale image/video flash.
      // binaryContent will be cleared by the load effect and re-rendered when new data arrives.
      if (binaryContent && binaryContent === _lastBinaryContentRef) return;
      _lastBinaryContentRef = binaryContent;
      void renderPreview();
    }
  });

  // Show correct tab slot when renderTabId changes
  $effect(() => {
    const id = renderTabId;
    if (tabSlots.size > 0) showTabSlot(id);
  });

  export async function renderPreview() {
    if (!previewArea || !filePath) return;
    requestTabRender(renderTabId);

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
    const snapContent = content;
    const snapBinary = binaryContent;
    const snapMtime = currentFileMtime;
    const snapGen = _contentGeneration;
    const tabVersion = requestTabRender(tabId);
    const slot = getOrCreateSlot(tabId);
    showTabSlot(tabId);

    const slotRendered = slot.dataset.rendered === 'true';
    const slotPath = slot.dataset.filePath;
    const slotMtime = slot.dataset.fileMtime;

    // Skip if this tab's slot already holds a fresh render of the same file.
    if (slotRendered && slotPath === path
        && slotMtime === String(snapMtime)) {
      const savedScroll = pendingRestoreScrollTop;
      pendingRestoreScrollTop = -1;
      if (savedScroll >= 0) { requestAnimationFrame(() => { slot.scrollTop = savedScroll; }); }
      if (isMarkdown) { requestAnimationFrame(() => setupScrollObserver()); }
      if (tocFocused && tocOpen && pendingTocSelectedIndex >= 0) {
        requestAnimationFrame(() => { tocSidebar?.setSelectedTocIndex(pendingTocSelectedIndex); pendingTocSelectedIndex = -1; tocSidebar?.focus(); });
      }
      return;
    }

    slot.innerHTML = '';
    slot.dataset.filePath = path;
    slot.dataset.renderVersion = String(tabVersion);

    const requestId = ++_renderRequestId;
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

    const previewContent: string | ArrayBuffer = snapBinary ?? snapContent;
    await getPreviewRouter().preview(path, previewContent, slot);
    if (requestId !== _renderRequestId || !isCurrentTabRender(tabId, path, tabVersion)) return;
    // Discard if loadFile started a new load during the async render
    if (snapGen !== _contentGeneration) return;

    slot.dataset.rendered = 'true';
    slot.dataset.fileMtime = String(snapMtime);

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
    if (isMarkdown) { setupScrollObserver(); }
    onRenderComplete();
  }

  export function renderSimpleCodePreview() {
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

  export async function renderDirectoryPreview() {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
    if (!filePath) return;
    const requestId = ++_renderRequestId;
    const previewer = getDirectoryPreviewer();
    slot.dataset.filePath = filePath;
    await previewer.render('', slot);
    if (requestId !== _renderRequestId) return;
  }

  export async function renderArchivePreview(path: string) {
    const slot = getOrCreateSlot(renderTabId);
    showTabSlot(renderTabId);
    const requestId = ++_renderRequestId;
    slot.dataset.filePath = path;
    await getPreviewRouter().preview(path, '', slot);
    if (requestId !== _renderRequestId) return;
  }

  export function scrollPreview(deltaY: number, deltaX: number = 0) {
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
          if (line >= 0) {
            tocActiveLine = line;
            onTocActiveLineChange(line);
          }
        }
      },
      { root: slot, rootMargin: '-10% 0px -80% 0px', threshold: 0 }
    );
    headings.forEach((el) => { scrollObserver!.observe(el); });
  }

  export function handleTocJump(line: number) {
    const slot = getActiveSlot();
    if (!slot) return;
    const heading = slot.querySelector(`[data-line="${line}"]`);
    if (heading) { heading.scrollIntoView({ behavior: 'smooth', block: 'start' }); tocActiveLine = line; onTocActiveLineChange(line); }
    onTocJump(line);
  }

  export function isTocVisible(): boolean {
    return (isMarkdown && tocHeadings.length > 0 && tocOpen);
  }

  export function focusToc() {
    tocFocused = true;
    tocSidebar?.focus();
    onTocFocusChange(true);
  }

  export function focusContent() {
    tocFocused = false;
    onTocFocusChange(false);
    if (previewWithToc) {
      const slot = getActiveSlot();
      if (slot) slot.focus();
    }
  }

  export function isTocFocused(): boolean {
    return tocFocused;
  }

  export function getTocSelectedIndex(): number {
    return tocSidebar?.getSelectedIndex() ?? -1;
  }

  function handleTocFocusChangeInternal(focused: boolean) {
    tocFocused = focused;
    onTocFocusChange(focused);
  }

  onMount(() => {
    // Initialize preview area
  });

  onDestroy(() => {
    previewRouter?.dispose();
    scrollObserver?.disconnect();
    clearAllSlots();
  });
</script>

<div class="preview-with-toc" bind:this={previewWithToc}>
  <div class="preview-area" bind:this={previewArea} aria-hidden="true"></div>
  {#if isMarkdown && tocHeadings.length > 0 && tocOpen}
    <TocSidebar
      bind:this={tocSidebar}
      headings={tocHeadings}
      activeLine={tocActiveLine}
      onJump={handleTocJump}
      onFocusChange={handleTocFocusChangeInternal}
    />
  {/if}
</div>

<style>
  .preview-with-toc {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .preview-with-toc.hidden {
    display: none;
  }

  .preview-with-toc.modeHidden {
    display: none;
  }

  .preview-area {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .preview-area :global(.tab-preview-slot) {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    overflow: auto;
    padding: 8px 16px;
    box-sizing: border-box;
    background-color: var(--bg-primary);
  }
</style>