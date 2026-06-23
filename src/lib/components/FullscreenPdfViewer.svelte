<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface TextMatch {
    x: number;
    y: number;
    width: number;
    height: number;
  }

  interface PdfSearchResult {
    page: number;
    matches: TextMatch[];
  }

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
  let viewportEl: HTMLDivElement | undefined = $state(undefined);
  let overlayEl: HTMLDivElement | undefined = $state(undefined);

  // Current page image data (fetched from backend)
  let pageData: { data: string; width: number; height: number } | null = $state(null);
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
  let searchResults: PdfSearchResult[] = $state([]);
  let currentMatchIndex = $state(0);
  let currentMatchPage = $state(0);
  let isSearching = $state(false);
  let searchStatus = $state('');

  // Preload cache
  let pageCache: Map<number, { data: string; width: number; height: number }> = new Map();
  let preloadingPages: Set<number> = new Set();

  const PAN_STEP = 100;
  const ZOOM_STEP = 0.25;
  const MIN_SCALE = 0.1;
  const RENDER_SCALE = 1.5;
  const PRELOAD_RANGE = 2;

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Fetch page data from backend
  async function fetchPage(pageNum: number): Promise<{ data: string; width: number; height: number }> {
    const cached = pageCache.get(pageNum);
    if (cached) return cached;

    const result = await invoke<{ data: string; width: number; height: number }>('render_pdf_page', {
      path: pdfPath,
      page: pageNum,
      scale: RENDER_SCALE,
    });
    pageCache.set(pageNum, result);
    return result;
  }

  // Draw page data onto canvas
  function drawToCanvas(data: { data: string; width: number; height: number }) {
    if (!canvasEl) return;

    const binary = atob(data.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const blob = new Blob([bytes.buffer], { type: 'image/png' });
    const url = URL.createObjectURL(blob);

    const img = new Image();
    img.onload = () => {
      if (!canvasEl) { URL.revokeObjectURL(url); return; }

      canvasEl.width = data.width;
      canvasEl.height = data.height;
      pageWidth = data.width;
      pageHeight = data.height;

      const ctx = canvasEl.getContext('2d');
      if (!ctx) { URL.revokeObjectURL(url); return; }

      ctx.clearRect(0, 0, data.width, data.height);
      ctx.drawImage(img, 0, 0);
      URL.revokeObjectURL(url);

      drawSearchHighlights(ctx, data.height);

      // Fit to screen
      if (viewportEl) {
        const vw = viewportEl.clientWidth;
        const vh = viewportEl.clientHeight;
        scale = Math.min(vw / data.width, vh / data.height, 1);
        translateX = 0;
        translateY = 0;
      }
    };
    img.onerror = () => {
      hasError = true;
      errorMessage = 'Failed to load page image';
      isLoading = false;
      URL.revokeObjectURL(url);
    };
    img.src = url;
  }

  // When canvasEl becomes available AND we have page data, draw
  $effect(() => {
    const canvas = canvasEl;
    const data = pageData;
    if (canvas && data) {
      drawToCanvas(data);
    }
  });

  async function goToPage(pageNum: number) {
    if (pageNum < 0 || pageNum >= totalPages) return;
    currentPage = pageNum;
    isLoading = true;
    hasError = false;

    try {
      const data = await fetchPage(pageNum);
      pageData = data;
      isLoading = false;
    } catch (error) {
      console.error(`Failed to render PDF page ${pageNum}:`, error);
      hasError = true;
      errorMessage = `Failed to render page ${pageNum}`;
      isLoading = false;
    }

    preloadPages(pageNum);
  }

  async function preloadPages(currentPageNum: number) {
    for (let i = -PRELOAD_RANGE; i <= PRELOAD_RANGE; i++) {
      const targetPage = currentPageNum + i;
      if (targetPage < 0 || targetPage >= totalPages || targetPage === currentPageNum) continue;
      if (pageCache.has(targetPage) || preloadingPages.has(targetPage)) continue;

      preloadingPages.add(targetPage);
      try {
        await fetchPage(targetPage);
      } catch (e) {
        // ignore preload errors
      } finally {
        preloadingPages.delete(targetPage);
      }
    }
  }

  // Search
  function drawSearchHighlights(ctx: CanvasRenderingContext2D, pageHeight: number) {
    const pageMatches = searchResults.find(r => r.page === currentPage);
    if (!pageMatches || pageMatches.matches.length === 0) return;

    for (let i = 0; i < pageMatches.matches.length; i++) {
      const match = pageMatches.matches[i];
      const isActive = currentPage === currentMatchPage && i === currentMatchIndex;

      ctx.fillStyle = isActive ? 'rgba(255, 165, 0, 0.5)' : 'rgba(255, 255, 0, 0.35)';

      const x = match.x * RENDER_SCALE;
      const y = pageHeight - (match.y + match.height) * RENDER_SCALE;
      const w = match.width * RENDER_SCALE;
      const h = match.height * RENDER_SCALE;

      ctx.fillRect(x, y, w, h);
    }
  }

  async function performSearch() {
    if (!searchQuery.trim()) {
      searchResults = [];
      searchStatus = '';
      return;
    }

    isSearching = true;
    searchStatus = 'Searching...';

    try {
      const results = await invoke<PdfSearchResult[]>('search_pdf_text', {
        path: pdfPath,
        query: searchQuery.trim(),
      });

      searchResults = results;
      const totalMatches = results.reduce((sum, r) => sum + r.matches.length, 0);

      if (totalMatches === 0) {
        searchStatus = 'No results';
        currentMatchIndex = 0;
        currentMatchPage = 0;
      } else {
        searchStatus = `${totalMatches} matches`;
        currentMatchPage = results[0].page;
        currentMatchIndex = 0;
        if (currentMatchPage !== currentPage) {
          await goToPage(currentMatchPage);
        } else {
          // Redraw with highlights
          if (pageData) drawToCanvas(pageData);
        }
      }
    } catch (error) {
      console.error('Search failed:', error);
      searchStatus = 'Search failed';
    } finally {
      isSearching = false;
    }
  }

  function navigateMatch(direction: 'next' | 'prev') {
    if (searchResults.length === 0) return;

    const allMatches: { page: number; index: number }[] = [];
    for (const result of searchResults) {
      for (let i = 0; i < result.matches.length; i++) {
        allMatches.push({ page: result.page, index: i });
      }
    }

    let currentPos = allMatches.findIndex(
      m => m.page === currentMatchPage && m.index === currentMatchIndex
    );

    if (currentPos === -1) {
      currentPos = 0;
    } else if (direction === 'next') {
      currentPos = (currentPos + 1) % allMatches.length;
    } else {
      currentPos = (currentPos - 1 + allMatches.length) % allMatches.length;
    }

    const target = allMatches[currentPos];
    currentMatchPage = target.page;
    currentMatchIndex = target.index;

    if (target.page !== currentPage) {
      goToPage(target.page);
    } else if (pageData) {
      drawToCanvas(pageData);
    }
  }

  function closeSearch() {
    showSearch = false;
    searchQuery = '';
    searchResults = [];
    searchStatus = '';
    currentMatchIndex = 0;
    currentMatchPage = 0;
    if (pageData) drawToCanvas(pageData);
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
        if (event.shiftKey) navigateMatch('prev');
        else if (searchResults.length > 0) navigateMatch('next');
        else performSearch();
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

  onMount(() => {
    requestAnimationFrame(() => overlayEl?.focus());
    goToPage(initialPage);
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="pdf-viewer-overlay"
  bind:this={overlayEl}
  onkeydown={handleKeydown}
  onclick={() => overlayEl?.focus()}
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
            if (searchResults.length > 0) navigateMatch(e.shiftKey ? 'prev' : 'next');
            else performSearch();
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
      <canvas bind:this={canvasEl} style={canvasStyle} class="pdf-canvas"></canvas>
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
