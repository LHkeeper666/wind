import { invoke } from '@tauri-apps/api/core';
import type PdfPreviewPanel from '$lib/components/PdfPreviewPanel.svelte';
import type PdfTocSidebar from '$lib/components/PdfTocSidebar.svelte';
import type { PdfPageDimensions, PdfOutlineItem } from '$lib/utils/pdf-shared';
import { fetchPdfOutline } from '$lib/utils/pdf-shared';
import { logError } from '$lib/utils/log';

export interface PdfCacheData {
  pdfCurrentPage: number;
  pdfPageCount: number;
  pdfPageDimensions: PdfPageDimensions[];
  pdfOutline: PdfOutlineItem[];
  pdfTocOpen: boolean;
  pdfTocFocused: boolean;
  pdfTocExpanded: number[];
  pdfTocSelectedIndex: number;
  pdfScale: number;
  pdfScrollTop: number;
}

interface PdfStateDeps {
  getFilePath: () => string | null;
  onToast: (msg: string) => void;
}

export interface PdfStateAPI {
  // Getters
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

  // Setters
  setPage: (page: number) => void;
  setTocFocused: (focused: boolean) => void;

  // Actions
  togglePdfToc: () => void;
  jumpToPdfPage: (page: number) => void;
  jumpToPage: (page: number, y: number) => void;
  focusPanel: () => boolean;
  focusToc: (silent?: boolean) => void;
  reset: () => void;
  applyPendingScroll: () => void;

  // Load integration
  loadPdfInfo: (path: string, gen: number) => Promise<boolean>;

  // Cache integration
  toCacheSnapshot: () => PdfCacheData;
  fromCacheSnapshot: (data: PdfCacheData) => void;
}

export function createPdfState(deps: PdfStateDeps): PdfStateAPI {
  let pdfPageCount: number = $state(0);
  let pdfCurrentPage: number = $state(0);
  let pdfFileSize: number = $state(0);
  let pdfTitle: string | null = $state(null);
  let pdfPageDimensions: PdfPageDimensions[] = $state([]);
  let pdfOutline: PdfOutlineItem[] = $state([]);
  let pdfTocOpen: boolean = $state(false);
  let pdfTocFocused: boolean = $state(false);
  let pdfTocExpanded: number[] = $state([]);
  let pdfTocSelectedIndex: number = $state(-1);
  let pdfScale: number = 0;
  let tocSidebarStatePending = false;
  let pdfTocSidebar: PdfTocSidebar | undefined = $state(undefined);
  let pdfPreviewPanel: PdfPreviewPanel | undefined = $state(undefined);

  // Track generation to discard stale async results
  let lastLoadGen: number = 0;
  let pendingPdfScrollTop: number = -1;
  // Invalidates in-flight focus retries when focus moves elsewhere.
  let focusRequest: number = 0;

  function togglePdfToc() {
    if (pdfOutline.length === 0) return;
    pdfTocOpen = !pdfTocOpen;
    if (pdfTocOpen) {
      setTimeout(() => pdfTocSidebar?.focus(), 0);
    }
  }

  function jumpToPdfPage(page: number) {
    pdfPreviewPanel?.scrollToPage(page);
  }

  function jumpToPage(page: number, y: number) {
    if (page >= 0 && page < pdfPageCount) {
      pdfPreviewPanel?.scrollToPage(page, y);
    }
  }

  function applyTocSidebarState() {
    if (!tocSidebarStatePending || !pdfTocSidebar) return;
    pdfTocSidebar.setExpandedIndexes(pdfTocExpanded);
    pdfTocSidebar.setSelectedIndex(pdfTocSelectedIndex);
    tocSidebarStatePending = false;
  }

  function applyPdfScale() {
    if (pdfScale > 0 && pdfPreviewPanel) pdfPreviewPanel.setScale(pdfScale);
  }

  function focusPanel(): boolean {
    focusRequest++;
    return pdfPreviewPanel?.focusPanel() ?? false;
  }

  function focusToc(silent = false) {
    const request = ++focusRequest;
    pdfTocFocused = true;
    if (!silent) deps.onToast('Focus: TOC');
    // The sidebar may not be mounted yet on the tick right after a tab restore.
    // Retry for a few frames, then fall back to the preview so focus does not
    // stay on an element belonging to the previous tab.
    let attempts = 0;
    const tryFocus = () => {
      if (request !== focusRequest) return;
      if (pdfTocSidebar?.focus()) return;
      if (++attempts >= 10) {
        pdfTocFocused = false;
        pdfPreviewPanel?.focusPanel();
        return;
      }
      requestAnimationFrame(tryFocus);
    };
    requestAnimationFrame(tryFocus);
  }

  function reset() {
    pdfPageCount = 0;
    pdfCurrentPage = 0;
    pdfFileSize = 0;
    pdfTitle = null;
    pdfPageDimensions = [];
    pdfOutline = [];
    pdfTocOpen = false;
    pdfTocFocused = false;
    pdfTocExpanded = [];
    pdfTocSelectedIndex = -1;
    pdfScale = 0;
    pdfPreviewPanel?.resetScale();
    tocSidebarStatePending = false;
    focusRequest++;
  }

  async function loadPdfInfo(path: string, gen: number): Promise<boolean> {
    lastLoadGen = gen;
    try {
      const info = await invoke<{
        page_count: number;
        title: string | null;
        author: string | null;
        file_size: number;
        page_dimensions: PdfPageDimensions[];
      }>('get_pdf_info', { path });
      if (gen !== lastLoadGen) return false;
      pdfPageCount = info.page_count;
      pdfCurrentPage = 0;
      pdfFileSize = info.file_size;
      pdfTitle = info.title;
      pdfPageDimensions = info.page_dimensions;
      pdfTocExpanded = [];
      pdfTocSelectedIndex = -1;
      pdfScale = 0;
      pdfPreviewPanel?.resetScale();
      tocSidebarStatePending = true;
      applyTocSidebarState();

      // Fetch outline (non-blocking, don't fail on outline errors)
      fetchPdfOutline(path)
        .then(outline => {
          if (gen === lastLoadGen) {
            pdfOutline = outline;
            if (outline.length > 0) pdfTocOpen = true;
          }
        })
        .catch(e => {
          logError('pdf', `outline fetch failed: ${e}`);
          if (gen === lastLoadGen) pdfOutline = [];
        });
      return true;
    } catch (error) {
      if (gen !== lastLoadGen) return false;
      logError('pdf', `Failed to load PDF: ${error}`);
      pdfPageCount = 0;
      pdfCurrentPage = 0;
      pdfPageDimensions = [];
      pdfOutline = [];
      return false;
    }
  }

  function toCacheSnapshot(): PdfCacheData {
    if (pdfTocSidebar) {
      pdfTocExpanded = pdfTocSidebar.getExpandedIndexes();
      pdfTocSelectedIndex = pdfTocSidebar.getSelectedIndex();
    }
    if (pdfPreviewPanel) pdfScale = pdfPreviewPanel.getScale();
    return {
      pdfCurrentPage,
      pdfPageCount,
      pdfPageDimensions: [...pdfPageDimensions],
      pdfOutline: [...pdfOutline],
      pdfTocOpen,
      pdfTocFocused,
      pdfTocExpanded: [...pdfTocExpanded],
      pdfTocSelectedIndex,
      pdfScale,
      pdfScrollTop: pdfPreviewPanel?.getScrollTop() ?? 0,
    };
  }

  function fromCacheSnapshot(data: PdfCacheData) {
    pdfCurrentPage = data.pdfCurrentPage;
    pdfPageCount = data.pdfPageCount;
    pdfPageDimensions = data.pdfPageDimensions ?? [];
    pdfOutline = data.pdfOutline ?? [];
    pdfTocOpen = data.pdfTocOpen ?? false;
    pdfTocFocused = data.pdfTocFocused ?? false;
    pdfTocExpanded = data.pdfTocExpanded ?? [];
    pdfTocSelectedIndex = data.pdfTocSelectedIndex ?? -1;
    pdfScale = data.pdfScale ?? 0;
    tocSidebarStatePending = true;
    applyTocSidebarState();
    applyPdfScale();
    focusRequest++;
    pendingPdfScrollTop = data.pdfScrollTop ?? 0;
  }

  function applyPendingScroll() {
    if (pendingPdfScrollTop < 0) return;
    const top = pendingPdfScrollTop;
    pendingPdfScrollTop = -1;
    // Double rAF ensures PdfPreviewPanel has completed initScale and layout
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        pdfPreviewPanel?.setScrollTop(top);
      });
    });
  }

  return {
    getPdfPageCount: () => pdfPageCount,
    getPdfCurrentPage: () => pdfCurrentPage,
    getPdfFileSize: () => pdfFileSize,
    getPdfTitle: () => pdfTitle,
    getPdfPageDimensions: () => pdfPageDimensions,
    getPdfOutline: () => pdfOutline,
    getPdfTocOpen: () => pdfTocOpen,
    getPdfTocFocused: () => pdfTocFocused,

    get pdfPreviewPanel() { return pdfPreviewPanel; },
    set pdfPreviewPanel(v: PdfPreviewPanel | undefined) {
      pdfPreviewPanel = v;
      if (v) applyPdfScale();
    },
    get pdfTocSidebar() { return pdfTocSidebar; },
    set pdfTocSidebar(v: PdfTocSidebar | undefined) {
      pdfTocSidebar = v;
      if (v) applyTocSidebarState();
    },

    setPage(page: number) { pdfCurrentPage = page; },
    setTocFocused(focused: boolean) {
      if (!focused) focusRequest++;
      pdfTocFocused = focused;
    },

    togglePdfToc,
    jumpToPdfPage,
    jumpToPage,
    focusPanel,
    focusToc,
    reset,
    applyPendingScroll,
    loadPdfInfo,
    toCacheSnapshot,
    fromCacheSnapshot,
  };
}