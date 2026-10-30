<script lang="ts">
import { onMount, onDestroy, tick } from 'svelte';
  import { formatSize } from '$lib/utils/file-types';
  import { logError } from '$lib/utils/log';
  import {
    type PdfPageData,
    type PdfSearchState,
    PdfTextLayoutCache,
    renderPdfTextLayer,
    shouldExecutePdfSearch,
    copyPdfTextSelection,
    PdfPageCache,
    fetchPdfPage,
    preloadPdfPages,
    drawPageWithHighlights,
    drawSearchHighlights,
    searchPdfText,
    navigateSearchMatch,
    getTotalMatchCount,
    RENDER_SCALE,
  } from '$lib/utils/pdf-shared';

  let {
    pdfPath,
    initialPage = 0,
    pageCount = 0,
    fileSize = 0,
    onClose = () => {},
  }: {
    pdfPath: string;
    initialPage?: number;
    pageCount?: number;
    fileSize?: number;
    onClose?: () => void;
  } = $props();

  // Page state
  let currentPage = $state(initialPage);
  let totalPages = $state(pageCount);

  // Canvas & viewport
  let canvasEl: HTMLCanvasElement | undefined = $state(undefined);
  let textLayerEl: HTMLDivElement | undefined = $state(undefined);
  let viewportEl: HTMLDivElement | undefined = $state(undefined);
  let overlayEl: HTMLDivElement | undefined = $state(undefined);

  // Current page image data (fetched from backend)
  let pageData: PdfPageData | null = $state(null);
  let pageWidth = $state(0);
  let pageHeight = $state(0);
  let isLoading = $state(true);
  let hasError = $state(false);
  let errorMessage = $state('');

  // Zoom/pan
  let scale = $state(1);
  let translateX = $state(0);
  let translateY = $state(0);

  // Search
  let showSearch = $state(false);
  let searchQuery = $state('');
  let searchInput: HTMLInputElement | undefined = $state(undefined);
  let searchState: PdfSearchState = $state({ results: [], currentMatchPage: 0, currentMatchIndex: 0 });
  let isSearching = $state(false);
  let searchStatus = $state('');
  let executedQuery = '';
  let searchRequest = 0;
  let pageRequest = 0;

  // Preload cache
  let pageCache = new PdfPageCache(16 * 1024 * 1024);
  let textCache = new PdfTextLayoutCache(4);

  const PAN_STEP = 100;
  const ZOOM_STEP = 0.25;
  const MIN_SCALE = 0.1;

  // Draw page data onto canvas with search highlights
  async function drawToCanvas(data: PdfPageData) {
    if (!canvasEl) return;
    try {
      await drawPageWithHighlights(canvasEl, data, searchState, currentPage);
      pageWidth = data.width;
      pageHeight = data.height;

      // Fit to screen
      if (viewportEl) {
        const vw = viewportEl.clientWidth;
        const vh = viewportEl.clientHeight;
        scale = Math.min(vw / data.width, vh / data.height, 1);
        translateX = 0;
        translateY = 0;
      }
    } catch {
      hasError = true;
      errorMessage = 'Failed to load page image';
      isLoading = false;
    }
  }

  async function goToPage(pageNum: number) {
    if (pageNum < 0 || pageNum >= totalPages) return;
    const request = ++pageRequest;
    currentPage = pageNum;
    isLoading = true;
    hasError = false;

    try {
      const data = await fetchPdfPage(pdfPath, pageNum, pageCache);
      if (request !== pageRequest) return;
      pageData = data;
      isLoading = false;
      await tick();
      await drawToCanvas(data);
      void loadText(pageNum, request);
    } catch (error) {
      if (request !== pageRequest) return;
      logError('FullscreenPdfViewer', `Failed to render PDF page ${pageNum}: ${error}`);
      hasError = true;
      errorMessage = `Failed to render page ${pageNum}`;
      isLoading = false;
    }

    preloadPdfPages(pdfPath, pageNum, totalPages, pageCache);
  }

  async function loadText(pageNum: number, request: number) {
    try {
      const layout = await textCache.get(pdfPath, pdfPath, pageNum);
      if (request === pageRequest && textLayerEl && pageData) {
        renderPdfTextLayer(textLayerEl, layout, pageData.height / RENDER_SCALE, RENDER_SCALE);
      }
    } catch { /* Raster rendering remains available. */ }
  }

  function positionMatch(pageNum: number, index: number) {
    const match = searchState.results.find(result => result.page === pageNum)?.matches[index];
    if (!match || !viewportEl || !pageData) return;
    const x = (match.x + match.width / 2) * RENDER_SCALE;
    const y = pageData.height - (match.y + match.height / 2) * RENDER_SCALE;
    translateX = (viewportEl.clientWidth / 2 - (x - pageData.width / 2) * scale - viewportEl.clientWidth / 2) / scale;
    translateY = (viewportEl.clientHeight / 2 - (y - pageData.height / 2) * scale - viewportEl.clientHeight / 2) / scale;
  }

  // Search
  async function performSearch() {
    const query = searchQuery.trim();
    const request = ++searchRequest;
    if (!query) {
      executedQuery = '';
      searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 };
      searchStatus = '';
      if (pageData && canvasEl) void drawPageWithHighlights(canvasEl, pageData, searchState, currentPage);
      return;
    }

    isSearching = true;
    searchStatus = 'Searching...';

    try {
      const results = await searchPdfText(pdfPath, query);
      if (request !== searchRequest) return;
      executedQuery = query;
      const totalMatches = results.reduce((sum, r) => sum + r.matches.length, 0);

      if (totalMatches === 0) {
        searchState = { results, currentMatchPage: 0, currentMatchIndex: 0 };
        searchStatus = 'No results';
      } else {
        searchState = {
          results,
          currentMatchPage: results[0].page,
          currentMatchIndex: 0,
        };
        searchStatus = `${totalMatches} matches`;
        if (results[0].page !== currentPage) {
          await goToPage(results[0].page);
        } else if (pageData && canvasEl) {
          await drawPageWithHighlights(canvasEl, pageData, searchState, currentPage);
        }
        if (request === searchRequest) positionMatch(results[0].page, 0);
      }
    } catch (error) {
      if (request !== searchRequest) return;
      logError('FullscreenPdfViewer', `Search failed: ${error}`);
      searchStatus = 'Search failed';
    } finally {
      if (request === searchRequest) isSearching = false;
    }
  }

  async function navigateMatch(direction: 'next' | 'prev') {
    const target = navigateSearchMatch(direction, searchState);
    if (!target) return;

    searchState = {
      ...searchState,
      currentMatchPage: target.page,
      currentMatchIndex: target.index,
    };

    if (target.page !== currentPage) {
      await goToPage(target.page);
    } else if (pageData && canvasEl) {
      await drawPageWithHighlights(canvasEl, pageData, searchState, currentPage);
    }
    positionMatch(target.page, target.index);
  }

  function submitSearch(direction: 'next' | 'prev') {
    if (shouldExecutePdfSearch(searchQuery, executedQuery, isSearching)) void performSearch();
    else void navigateMatch(direction);
  }

  function closeSearch() {
    showSearch = false;
    searchQuery = '';
    executedQuery = '';
    searchRequest++;
    searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 };
    searchStatus = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (showSearch) {
      if (event.key === 'Escape') {
        event.preventDefault();
        closeSearch();
        return;
      }
      if (event.key === 'Enter') {
        event.preventDefault();
        submitSearch(event.shiftKey ? 'prev' : 'next');
        return;
      }
      return;
    }

    if (event.key === 'Escape' || (event.key === 'q' && !event.ctrlKey)) {
      event.preventDefault();
      onClose();
      return;
    }

    if (event.key === '/') {
      event.preventDefault();
      showSearch = true;
      setTimeout(() => searchInput?.focus(), 0);
      return;
    }

    if (event.key === 'n' && !event.ctrlKey) {
      event.preventDefault();
      navigateMatch('next');
      return;
    }

    if (event.key === 'N' && !event.ctrlKey) {
      event.preventDefault();
      navigateMatch('prev');
      return;
    }

    if (event.ctrlKey) {
      switch (event.key) {
        case 'j': event.preventDefault(); translateY -= PAN_STEP; break;
        case 'k': event.preventDefault(); translateY += PAN_STEP; break;
        case 'h': event.preventDefault(); translateX += PAN_STEP; break;
        case 'l': event.preventDefault(); translateX -= PAN_STEP; break;
      }
      return;
    }

    switch (event.key) {
      case 'j': event.preventDefault(); if (currentPage < totalPages - 1) goToPage(currentPage + 1); break;
      case 'k': event.preventDefault(); if (currentPage > 0) goToPage(currentPage - 1); break;
      case 'h': event.preventDefault(); scale = Math.max(MIN_SCALE, scale - ZOOM_STEP); break;
      case 'l': event.preventDefault(); scale += ZOOM_STEP; break;
      case 'g': event.preventDefault(); goToPage(0); break;
      case 'G': event.preventDefault(); goToPage(totalPages - 1); break;
    }
  }

  const canvasStyle = $derived(
    `transform: scale(${scale}) translate(${translateX}px, ${translateY}px); transform-origin: center center;`
  );

  const fileName = $derived(pdfPath.split(/[/\\]/).pop() || pdfPath);
  const position = $derived(`${currentPage + 1}/${totalPages}`);
  const zoomPercent = $derived(`${Math.round(scale * 100)}%`);

  onDestroy(() => { pageCache.clear(); textCache.clear(); pageRequest++; searchRequest++; });

  onMount(() => {
    const handleCopy = (event: ClipboardEvent) => copyPdfTextSelection(event, overlayEl);
    document.addEventListener('copy', handleCopy);
    requestAnimationFrame(() => overlayEl?.focus());
    goToPage(initialPage);
    return () => document.removeEventListener('copy', handleCopy);
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="pdf-viewer-overlay"
  bind:this={overlayEl}
  onkeydown={handleKeydown}
  onclick={() => { if (window.getSelection()?.isCollapsed) overlayEl?.focus(); }}
  role="dialog"
  aria-label="Fullscreen PDF Viewer"
  tabindex="-1"
>
  {#if showSearch}
    <div class="pdf-search-bar">
      <input
        type="text"
        class="pdf-search-input"
        placeholder="Search text..."
        bind:value={searchQuery}
        bind:this={searchInput}
        onkeydown={(e) => {
          if (e.key === 'Escape') closeSearch();
          if (e.key === 'Enter') {
            submitSearch(e.shiftKey ? 'prev' : 'next');
          }
        }}
      />
      {#if searchStatus}
        <span class="pdf-search-status">{searchStatus}</span>
      {/if}
    </div>
  {/if}

  <div class="pdf-viewport" bind:this={viewportEl}>
    {#if isLoading}
      <div class="pdf-viewer-status">Loading...</div>
    {:else if hasError}
      <div class="pdf-viewer-status error">{errorMessage}</div>
    {:else}
      <div class="pdf-page-surface" style={canvasStyle}>
        <canvas bind:this={canvasEl} class="pdf-canvas"></canvas>
        <div bind:this={textLayerEl} class="pdf-text-layer"></div>
      </div>
    {/if}
  </div>

  <div class="pdf-footer">
    <span class="pdf-footer-name">{fileName}</span>
    {#if !isLoading && !hasError && pageWidth > 0}
      <span class="pdf-footer-meta">{pageWidth}×{pageHeight}</span>
    {/if}
    {#if fileSize > 0}
      <span class="pdf-footer-meta">{formatSize(fileSize)}</span>
    {/if}
    <span class="pdf-footer-meta">{position}</span>
    <span class="pdf-footer-meta">{zoomPercent}</span>
    <span class="pdf-footer-hints">j/k:翻页 h/l:缩放 g/G:首末页 /:搜索 Esc:关闭</span>
  </div>
</div>

<style>
  .pdf-viewer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    background-color: #000;
    display: flex;
    flex-direction: column;
    outline: none;
  }

  .pdf-search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 16px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    z-index: 10;
    font-family: var(--font-mono);
  }

  .pdf-search-input {
    flex: 1;
    max-width: 400px;
    padding: 4px 10px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-mono);
    outline: none;
  }

  .pdf-search-input:focus {
    border-color: var(--border-focus);
  }

  .pdf-search-status {
    color: var(--text-muted);
    font-size: 12px;
  }

  .pdf-viewport {
    flex: 1;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pdf-canvas {
    max-width: none;
    max-height: none;
    user-select: none;
    -webkit-user-drag: none;
  }
  .pdf-page-surface { position: relative; flex: none; }
  .pdf-page-surface .pdf-text-layer { position: absolute; inset: 0; pointer-events: none; user-select: text; }
  .pdf-page-surface :global(.pdf-text-character) { position: absolute; display: block; color: transparent; white-space: pre; line-height: 1; overflow: hidden; pointer-events: auto; user-select: text; }
  .pdf-page-surface :global(.pdf-text-character::selection) { background: rgba(70, 130, 220, 0.45); }
  .pdf-page-surface :global(.pdf-text-whitespace) { position: absolute; width: 0; height: 0; overflow: hidden; color: transparent; white-space: pre; pointer-events: none; }

  .pdf-viewer-status {
    color: var(--text-muted);
    font-size: 14px;
    font-family: var(--font-mono);
  }

  .pdf-viewer-status.error {
    color: var(--error);
  }

  .pdf-footer {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 4px 16px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 12px;
    font-family: var(--font-mono);
  }

  .pdf-footer-name {
    color: var(--text-primary);
  }

  .pdf-footer-meta {
    color: var(--text-muted);
  }

  .pdf-footer-hints {
    margin-left: auto;
    color: var(--text-muted);
    font-size: 11px;
  }
</style>
