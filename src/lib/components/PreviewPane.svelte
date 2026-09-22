<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { PreviewRouter } from '$lib/previewers';
  import type { TocHeading } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import TocSidebar from './TocSidebar.svelte';
  import { restoreExpandedLines } from '$lib/utils/tab-cache';

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
    onTocJump = (line: number) => {},
    onTocFocusChange = (focused: boolean) => {},
    onRenderComplete = () => {},
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
    onTocJump: (line: number) => void;
    onTocFocusChange: (focused: boolean) => void;
    onRenderComplete: () => void;
  } = $props();

  let previewArea: HTMLElement | undefined = $state(undefined);
  let previewWithToc: HTMLElement | undefined = $state(undefined);
  let tocSidebar: TocSidebar | undefined = $state(undefined);
  let previewRouter: PreviewRouter | undefined;
  let directoryPreviewer: DirectoryPreviewer | undefined;
  let renderRequestId: number = 0;
  let renderInFlight: boolean = false;
  let renderQueued: boolean = false;
  let scrollObserver: IntersectionObserver | undefined;

  // Per-tab persistent preview slots
  const tabSlots = new Map<number, HTMLDivElement>();
  let currentTabId: number = 0;

  export function setTabId(tabId: number) {
    currentTabId = tabId;
  }

  export function getActiveSlot(): HTMLDivElement | undefined {
    return tabSlots.get(currentTabId);
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
      slot.style.zIndex = id === tabId ? '1' : '0';
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

  export function requestTabRender(tabId: number): number {
    return (tabSlots.get(tabId)?.dataset.renderVersion ? parseInt(tabSlots.get(tabId)!.dataset.renderVersion!) : 0) + 1;
  }

  export function isCurrentTabRender(tabId: number, path: string, version: number): boolean {
    const slot = tabSlots.get(tabId);
    return slot?.dataset.renderVersion === String(version) && slot?.dataset.filePath === path;
  }

  export async function renderPreview() {
    if (!previewArea || !filePath) return;
    requestTabRender(currentTabId);

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

    const tabId = currentTabId;
    const path = filePath;
    const tabVersion = requestTabRender(tabId);
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
    slot.dataset.renderVersion = String(tabVersion);

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
    if (isMarkdown) { setupScrollObserver(); }
    onRenderComplete();
  }

  export function renderSimpleCodePreview() {
    const slot = getOrCreateSlot(currentTabId);
    showTabSlot(currentTabId);
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
    const slot = getOrCreateSlot(currentTabId);
    showTabSlot(currentTabId);
    if (!filePath) return;
    const requestId = ++renderRequestId;
    const previewer = getDirectoryPreviewer();
    slot.dataset.filePath = filePath;
    await previewer.render('', slot);
    if (requestId !== renderRequestId) return;
  }

  export async function renderArchivePreview(path: string) {
    const slot = getOrCreateSlot(currentTabId);
    showTabSlot(currentTabId);
    const requestId = ++renderRequestId;
    slot.dataset.filePath = path;
    await getPreviewRouter().preview(path, '', slot);
    if (requestId !== renderRequestId) return;
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
          if (line >= 0) { tocActiveLine = line; }
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
    if (heading) { heading.scrollIntoView({ behavior: 'smooth', block: 'start' }); tocActiveLine = line; }
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
  }
</style>