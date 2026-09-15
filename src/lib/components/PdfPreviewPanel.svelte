<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    type PdfPageData,
    type PdfPageDimensions,
    type PdfSearchState,
    type PdfLinkAnnotation,
    PdfPageCache,
    fetchPdfPage,
    drawSearchHighlights,
    searchPdfText,
    navigateSearchMatch,
    RENDER_SCALE,
    cumulativePageOffsets,
    pageAtScrollPosition,
    fetchPdfPageLinks,
    clearLinkCache,
    PdfRenderScheduler,
  } from '$lib/utils/pdf-shared';

  let {
    pdfPath,
    pageDimensions = [],
    pageCount = 0,
    fileSize = 0,
    title = null,
    onPageChange = (_page: number) => {},
    onFullscreen = () => {},
  }: {
    pdfPath: string;
    pageDimensions?: PdfPageDimensions[];
    pageCount?: number;
    fileSize?: number;
    title?: string | null;
    onPageChange?: (page: number) => void;
    onFullscreen?: () => void;
  } = $props();

  // State
  let currentPage = $state(0);
  let totalPages = $derived(pageCount);
  let scale = $state(1);
  let scaleInitialized = false;
  let isLoading = $state(true);
  let hasError = $state(false);
  let errorMessage = $state('');
  let hasFocus = $state(false);
  let isHovered = $state(false);

  // DOM refs
  let containerEl: HTMLDivElement | undefined = $state(undefined);
  let scrollEl: HTMLDivElement | undefined = $state(undefined);

  // Search
  let showSearch = $state(false);
  let searchQuery = $state('');
  let searchInput: HTMLInputElement | undefined = $state(undefined);
  let searchState: PdfSearchState = $state({ results: [], currentMatchPage: 0, currentMatchIndex: 0 });
  let isSearching = $state(false);
  let searchStatus = $state('');

  // Cache
  let pageCache = new PdfPageCache(80 * 1024 * 1024);
  let lowResCache = new PdfPageCache(16 * 1024 * 1024); // low-res progressive cache
  let renderScheduler = new PdfRenderScheduler();
  let linkCache = new Map<number, PdfLinkAnnotation[]>();
  let resizeObserver: ResizeObserver | undefined;

  // Track canvas elements for reactive size updates
  let canvasRefs = new Map<number, HTMLCanvasElement>();

  // Track which pages have canvas elements rendered
  let renderedPages = new Set<number>();
  // Track in-flight work separately so a low-res task cannot hide a missing high-res upgrade.
  let lowRenderingPages = new Set<number>();
  let highRenderingPages = new Set<number>();
  // Track which pages have link overlays loaded
  let loadedLinks = new Set<number>();
  // Request cancellation: incremented on viewport changes to skip stale renders
  let renderGeneration = 0;
  let lastRenderRangeStart = -1;
  let lastRenderRangeEnd = -1;
  let renderRetryScheduled = false;
  let jumpToken = 0;
  let programmaticScroll = false;
  let jumpOverflowAnchor = '';

  const BUFFER_PAGES = 3;
  const SCROLL_STEP = 40;
  const ZOOM_STEP = 0.15;
  const PRELOAD_PAGES = 5; // pages beyond buffer to pre-fetch into cache
  const LOW_RES_SCALE = 0.5; // fast low-res for progressive rendering
  const LOW_RES_PRELOAD = 15; // wider preload range for low-res
  const MIN_SCALE = 0.3;
  const MAX_SCALE = 5.0;

  function scheduleRenderRetry() {
    if (renderRetryScheduled) return;
    renderRetryScheduled = true;
    requestAnimationFrame(() => {
      renderRetryScheduled = false;
      updatePageRendering();
    });
  }

  let viewportWidth = $state(0);
  let pageTrackWidth = $derived.by(() => {
    const widestPage = pageDimensions.length > 0 ? Math.max(...pageDimensions.map(d => d.width * scale)) : 0;
    return Math.max(viewportWidth, widestPage);
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Page slot heights based on dimensions and scale
  let pageOffsets = $derived(cumulativePageOffsets(pageDimensions, scale));
  let totalHeight = $derived(pageOffsets.length > 0 ? pageOffsets[pageOffsets.length - 1] : 0);

  // Initialize scale to fit first page width
  function initScale() {
    if (scaleInitialized || !scrollEl || pageDimensions.length === 0) return;
    const vw = scrollEl.clientWidth;
    if (vw === 0) return;
    // Fit the widest page to viewport width
    const maxPageW = Math.max(...pageDimensions.map(d => d.width));
    scale = Math.min(vw / maxPageW, 1);
    scaleInitialized = true;
  }

  // Update current page from scroll position
  function updateCurrentPage() {
    if (!scrollEl || pageDimensions.length === 0) return;
    const newPage = pageAtScrollPosition(scrollEl.scrollTop, scrollEl.clientHeight, pageOffsets);
    if (newPage !== currentPage && newPage >= 0 && newPage < totalPages) {
      currentPage = newPage;
      onPageChange(newPage);
    }
  }

  // Draw a canvas from page data and insert into slot
  function drawCanvasIntoSlot(slot: HTMLElement, pageNum: number, data: PdfPageData, tag?: string, generation?: number) {
    const t0 = performance.now();
    if (generation !== undefined) slot.dataset.renderGeneration = String(generation);
    const canvas = document.createElement('canvas');
    canvas.className = 'pdf-canvas';

    // Draw page content
    const binary = atob(data.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const mimeType = data.format === 'jpeg' ? 'image/jpeg' : 'image/png';
    const blob = new Blob([bytes.buffer], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const tDecode = performance.now();

    const img = new Image();
    img.onload = () => {
      if (generation !== undefined && generation !== renderGeneration) {
        URL.revokeObjectURL(url);
        if (slot.dataset.renderGeneration === String(generation)) {
          renderedPages.delete(pageNum);
          canvasRefs.delete(pageNum);
        }
        return;
      }
      const incomingQuality = tag?.includes('high') ? 2 : 1;
      const existingQuality = Number(slot.dataset.renderQuality || '0');
      if (incomingQuality < existingQuality) {
        URL.revokeObjectURL(url);
        return;
      }
      const tLoad = performance.now();
      console.log(`[pdf-perf] img.onload p${pageNum} ${tag || ''}`);
      canvas.width = data.width;
      canvas.height = data.height;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.clearRect(0, 0, data.width, data.height);
        ctx.drawImage(img, 0, 0);
        drawSearchHighlights(ctx, pageNum, data.height, searchState);
      }
      URL.revokeObjectURL(url);

      const dim = pageDimensions[pageNum];

      // Wrap canvas + overlay in a container so overlay aligns with canvas
      const wrapper = document.createElement('div');
      wrapper.className = 'pdf-page-wrapper';
      wrapper.style.position = 'relative';
      if (dim) {
        wrapper.style.width = `${dim.width * scale}px`;
        wrapper.style.height = `${dim.height * scale}px`;
      }
      wrapper.appendChild(canvas);
      slot.innerHTML = '';
      slot.appendChild(wrapper);
      slot.dataset.renderQuality = String(incomingQuality);

      // Track canvas ref for reactive size updates
      canvasRefs.set(pageNum, canvas);
      const tDraw = performance.now();
      console.log(`[pdf-perf] draw p${pageNum} ${tag || ''} decode=${(tDecode - t0).toFixed(1)}ms imgLoad=${(tLoad - tDecode).toFixed(1)}ms canvasDraw=${(tDraw - tLoad).toFixed(1)}ms total=${(tDraw - t0).toFixed(1)}ms size=${data.width}x${data.height}`);
      loadLinkOverlay(pageNum, wrapper, data); // overlay attached to wrapper for correct alignment
    };
    img.onerror = (e) => {
      URL.revokeObjectURL(url);
      if (generation !== undefined && generation !== renderGeneration) return;
      const incomingQuality = tag?.includes('high') ? 2 : 1;
      if (incomingQuality < Number(slot.dataset.renderQuality || '0')) return;
      console.error(`[pdf-perf] img.onerror p${pageNum} ${tag || ''} blobUrl=${url.substring(0, 30)} dataSize=${data.data.length}`);
      slot.innerHTML = `<div class="pdf-page-error">Page ${pageNum + 1}</div>`;
    };
    img.src = url;
  }

  // Render a single page using cached data (no Tauri invoke, synchronous draw)
  function renderPageFromCache(pageNum: number, slot: HTMLElement, data: PdfPageData, generation = renderGeneration, tag = 'cache') {
    renderedPages.add(pageNum);
    drawCanvasIntoSlot(slot, pageNum, data, tag, generation);
  }

  function queueHighRender(pageNum: number, slot: HTMLElement, priority: 0 | 1 | 2, generation: number) {
    const highCached = pageCache.get(pageNum);
    if (highCached) {
      if (slot.dataset.renderQuality !== '2') renderPageFromCache(pageNum, slot, highCached, generation, 'high-res-cached');
      return;
    }
    if (highRenderingPages.has(pageNum)) return;

    highRenderingPages.add(pageNum);
    console.log(`[pdf-scheduler] enqueue page=${pageNum} variant=high priority=${priority} gen=${generation}`);
    let stale = false;
    renderScheduler.enqueue(
      () => {
        console.log(`[pdf-scheduler] start page=${pageNum} variant=high gen=${generation}`);
        return fetchPdfPage(pdfPath, pageNum, pageCache, undefined, () => generation === renderGeneration);
      },
      priority,
      generation,
    ).then((data) => {
      if (generation !== renderGeneration) {
        stale = true;
        console.log(`[pdf-scheduler] discard page=${pageNum} variant=high gen=${generation}`);
        return;
      }
      lowResCache.delete(pageNum);
      renderedPages.add(pageNum);
      console.log(`[pdf-scheduler] commit page=${pageNum} variant=high gen=${generation}`);
      drawCanvasIntoSlot(slot, pageNum, data, 'high-res', generation);
    }).catch((error) => {
      stale = generation !== renderGeneration;
      if (!stale) console.error(`Failed to render high-res page ${pageNum}:`, error);
    }).finally(() => {
      highRenderingPages.delete(pageNum);
      if (stale) scheduleRenderRetry();
    });
  }

  function queueLowRender(pageNum: number, slot: HTMLElement, priority: 0 | 1 | 2, generation: number) {
    if (pageCache.has(pageNum) || lowResCache.has(pageNum) || lowRenderingPages.has(pageNum)) return;

    lowRenderingPages.add(pageNum);
    console.log(`[pdf-scheduler] enqueue page=${pageNum} variant=low priority=${priority} gen=${generation}`);
    let stale = false;
    renderScheduler.enqueue(
      () => {
        console.log(`[pdf-scheduler] start page=${pageNum} variant=low gen=${generation}`);
        return fetchPdfPage(pdfPath, pageNum, lowResCache, LOW_RES_SCALE, () => generation === renderGeneration);
      },
      priority,
      generation,
    ).then((data) => {
      if (generation !== renderGeneration || pageCache.has(pageNum)) {
        stale = generation !== renderGeneration;
        return;
      }
      renderedPages.add(pageNum);
      console.log(`[pdf-scheduler] commit page=${pageNum} variant=low gen=${generation}`);
      drawCanvasIntoSlot(slot, pageNum, data, 'low-res', generation);
      queueHighRender(pageNum, slot, priority, generation);
    }).catch((error) => {
      stale = generation !== renderGeneration;
      if (!stale) console.error(`Failed to render low-res page ${pageNum}:`, error);
    }).finally(() => {
      lowRenderingPages.delete(pageNum);
      if (stale) scheduleRenderRetry();
    });
  }

  function renderPage(pageNum: number, slot: HTMLElement, priority: 0 | 1 | 2 = 1, generation = renderGeneration) {
    const highCached = pageCache.get(pageNum);
    if (highCached) {
      if (slot.dataset.renderQuality !== '2') renderPageFromCache(pageNum, slot, highCached, generation, 'high-res-cached');
      return;
    }

    const lowCached = lowResCache.get(pageNum);
    if (lowCached && !renderedPages.has(pageNum)) {
      renderPageFromCache(pageNum, slot, lowCached, generation, 'low-res-cached');
    }
    if (lowCached) queueHighRender(pageNum, slot, priority, generation);
    else queueLowRender(pageNum, slot, priority, generation);
  }

  // Load and render link annotation overlay
  async function loadLinkOverlay(pageNum: number, slot: HTMLElement, data: PdfPageData) {
    if (loadedLinks.has(pageNum)) return;
    loadedLinks.add(pageNum);

    try {
      const links = await fetchPdfPageLinks(pdfPath, pageNum);
      if (links.length === 0) { console.log(`[pdf] links p${pageNum}: 0`); return; }
      console.log(`[pdf] links p${pageNum}: ${links.length} items`, links[0]);

      const dim = pageDimensions[pageNum];
      if (!dim) return;

      const overlay = document.createElement('div');
      overlay.className = 'link-overlay';
      overlay.style.position = 'absolute';
      overlay.style.top = '0';
      overlay.style.left = '0';
      overlay.style.width = '100%';
      overlay.style.height = '100%';

      for (const link of links) {
        if (link.is_external && !link.url) continue;

        const rect = document.createElement('div');
        rect.className = 'link-rect';
        // PDF coords: origin at bottom-left. CSS: origin at top-left.
        rect.style.position = 'absolute';
        rect.style.left = `${(link.rect[0] / dim.width) * 100}%`;
        rect.style.bottom = `${(link.rect[1] / dim.height) * 100}%`;
        rect.style.width = `${((link.rect[2] - link.rect[0]) / dim.width) * 100}%`;
        rect.style.height = `${((link.rect[3] - link.rect[1]) / dim.height) * 100}%`;

        rect.addEventListener('click', (e) => {
          e.stopPropagation();
          if (link.is_external && link.url) {
            window.open(link.url, '_blank');
          } else {
            jumpToLink(link);
          }
        });

        overlay.appendChild(rect);
      }

      slot.appendChild(overlay);
    } catch {
      // ignore link loading errors
    }
  }

  // Jump to a link target
  function jumpToLink(link: PdfLinkAnnotation) {
    if (!scrollEl) return;
    schedulePageJump(link.target_page, link.target_y);
  }

  function getPageScrollTop(pageNum: number, pdfY = 0): number {
    if (!scrollEl || pageNum < 0 || pageNum >= pageDimensions.length) return 0;
    const slot = scrollEl.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
    const pageTop = slot
      ? slot.offsetTop
      : pageOffsets[pageNum] || 0;
    const pageHeight = pageDimensions[pageNum]?.height || 0;
    const hasExplicitY = Number.isFinite(pdfY) && pdfY > 0 && pdfY < pageHeight;
    const yOffset = hasExplicitY ? (pageHeight - pdfY) * scale : 0;
    const target = Math.max(0, pageTop + yOffset);
    console.log(`[pdf-nav] measure page=${pageNum} slotTop=${pageTop} yOffset=${yOffset.toFixed(1)} scale=${scale.toFixed(2)} target=${target.toFixed(1)}`);
    return target;
  }

  function schedulePageJump(pageNum: number, pdfY = 0) {
    if (!scrollEl || pageNum < 0 || pageNum >= pageDimensions.length) return;
    const token = ++jumpToken;
    renderGeneration++;
    renderScheduler.invalidate(renderGeneration);
    if (!programmaticScroll) jumpOverflowAnchor = scrollEl.style.overflowAnchor;
    scrollEl.style.overflowAnchor = 'none';
    programmaticScroll = true;

    requestAnimationFrame(() => requestAnimationFrame(() => {
      if (!scrollEl || token !== jumpToken) return;
      scrollEl.scrollTop = getPageScrollTop(pageNum, pdfY);
      requestAnimationFrame(() => {
        if (!scrollEl || token !== jumpToken) return;
        scrollEl.scrollTop = getPageScrollTop(pageNum, pdfY);
        programmaticScroll = false;
        scrollEl.style.overflowAnchor = jumpOverflowAnchor;
      });
    }));
  }

  function cancelPendingJump() {
    if (!programmaticScroll) return;
    jumpToken++;
    programmaticScroll = false;
    if (scrollEl) scrollEl.style.overflowAnchor = jumpOverflowAnchor;
  }

  // Destroy a page canvas (keep the slot)
  function destroyPage(pageNum: number, slot: HTMLElement) {
    renderedPages.delete(pageNum);
    lowRenderingPages.delete(pageNum);
    highRenderingPages.delete(pageNum);
    loadedLinks.delete(pageNum);
    canvasRefs.delete(pageNum);
    slot.innerHTML = '';
    delete slot.dataset.renderQuality;
    delete slot.dataset.renderGeneration;
  }

  // Find the first page whose bottom offset > position (binary search)
  function findPageAtOffset(position: number): number {
    let lo = 0, hi = pageOffsets.length - 2;
    while (lo < hi) {
      const mid = (lo + hi) >>> 1;
      if (pageOffsets[mid + 1] <= position) lo = mid + 1;
      else hi = mid;
    }
    return Math.max(0, lo);
  }

  // Render/destroy pages based on viewport + buffer, and preload beyond buffer
  function updatePageRendering() {
    if (!scrollEl || pageOffsets.length < 2) return;

    const t0 = performance.now();

    const ch = scrollEl.clientHeight;
    const st = scrollEl.scrollTop;

    // Calculate visible range (viewport ± BUFFER_PAGES)
    const bufPx = BUFFER_PAGES * ch;
    const rangeStart = Math.max(0, findPageAtOffset(st - bufPx));
    const rangeEnd = Math.min(totalPages - 1, findPageAtOffset(st + ch + bufPx));
    const visibleStart = Math.max(0, findPageAtOffset(st));
    const visibleEnd = Math.min(totalPages - 1, findPageAtOffset(st + ch));
    if (rangeStart !== lastRenderRangeStart || rangeEnd !== lastRenderRangeEnd) {
      renderGeneration++;
      lastRenderRangeStart = rangeStart;
      lastRenderRangeEnd = rangeEnd;
    }
    const generation = renderGeneration;
    const pinnedPages = Array.from({ length: rangeEnd - rangeStart + 1 }, (_, index) => rangeStart + index);
    pageCache.setPinnedPages(pinnedPages);
    lowResCache.setPinnedPages(pinnedPages);
    renderScheduler.invalidate(generation);

    // Destroy pages outside buffer
    let destroyed = 0;
    for (const pageNum of renderedPages) {
      if (pageNum < rangeStart || pageNum > rangeEnd) {
        const slot = scrollEl.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
        if (slot) { destroyPage(pageNum, slot); destroyed++; }
      }
    }

    // Draw cached pages instantly, then render uncached from center outward (most visible first)
    const center = findPageAtOffset(st + ch / 2);
    let cacheHits = 0;
    let uncachedCount = 0;
    // Build page order: center, center-1, center+1, center-2, center+2, ...
    const renderOrder: number[] = [];
    for (let d = 0; ; d++) {
      const lo = center - d;
      const hi = center + d;
      if (lo < rangeStart && hi > rangeEnd) break;
      if (lo >= rangeStart) renderOrder.push(lo);
      if (hi > lo && hi <= rangeEnd) renderOrder.push(hi);
    }

    for (const i of renderOrder) {
      const slot = scrollEl.querySelector(`.page-slot[data-page="${i}"]`) as HTMLElement | null;
      if (!slot) continue;

      const cached = pageCache.get(i);
      if (cached) {
        if (slot.dataset.renderQuality !== '2') renderPageFromCache(i, slot, cached, generation, 'high-res-cached');
        cacheHits++;
        continue;
      }
      const lowCached = lowResCache.get(i);
      if (lowCached) {
        if (!renderedPages.has(i)) renderPageFromCache(i, slot, lowCached, generation, 'low-res-cached');
        cacheHits++;
      }

      const priority = i >= visibleStart && i <= visibleEnd ? 0 : (i >= rangeStart && i <= rangeEnd ? 1 : 2);
      renderPage(i, slot, priority, generation);
      uncachedCount++;
    }

    const t1 = performance.now();
    console.log(`[pdf-perf] updatePageRendering: range=[${rangeStart}-${rangeEnd}] cacheHits=${cacheHits} uncached=${uncachedCount} destroyed=${destroyed} gen=${renderGeneration} time=${(t1 - t0).toFixed(1)}ms scrollTop=${st.toFixed(0)}`);

    // Defer preloads so visible pages' high-res fetches get Mutex priority.
    // renderPage() awaits low-res then starts high-res; without the delay,
    // ~30 preload requests would queue ahead of visible pages' high-res fetches.
    const mid = Math.floor((rangeStart + rangeEnd) / 2);
    setTimeout(() => {
      if (generation !== renderGeneration) return;
      // Preload low-res for wide range
      const lowStart = Math.max(0, rangeStart - LOW_RES_PRELOAD);
      const lowEnd = Math.min(totalPages - 1, rangeEnd + LOW_RES_PRELOAD);
      for (let page = lowStart; page <= lowEnd; page++) {
        if (!pageCache.has(page) && !lowResCache.has(page)
          && !lowRenderingPages.has(page) && !lowResCache.isPreloading(page)) {
          lowResCache.markPreloading(page);
          renderScheduler.enqueue(() => fetchPdfPage(pdfPath, page, lowResCache, LOW_RES_SCALE, () => generation === renderGeneration), 2, generation)
            .catch(() => {}).finally(() => lowResCache.unmarkPreloading(page));
        }
      }

      // Preload high-res ONLY outside buffer
      const hiPreStart = Math.max(0, rangeStart - PRELOAD_PAGES);
      const hiPreEnd = Math.min(totalPages - 1, rangeEnd + PRELOAD_PAGES);
      if (hiPreStart < rangeStart) {
        for (let page = hiPreStart; page < rangeStart; page++) {
          if (!pageCache.has(page) && !highRenderingPages.has(page) && !pageCache.isPreloading(page)) {
            pageCache.markPreloading(page);
            renderScheduler.enqueue(() => fetchPdfPage(pdfPath, page, pageCache, undefined, () => generation === renderGeneration), 2, generation)
              .catch(() => {}).finally(() => pageCache.unmarkPreloading(page));
          }
        }
      }
      if (hiPreEnd > rangeEnd) {
        for (let page = rangeEnd + 1; page <= hiPreEnd; page++) {
          if (!pageCache.has(page) && !highRenderingPages.has(page) && !pageCache.isPreloading(page)) {
            pageCache.markPreloading(page);
            renderScheduler.enqueue(() => fetchPdfPage(pdfPath, page, pageCache, undefined, () => generation === renderGeneration), 2, generation)
              .catch(() => {}).finally(() => pageCache.unmarkPreloading(page));
          }
        }
      }
    }, 100);
  }

  // Re-render all visible pages (e.g., after search state change)
  function rerenderVisiblePages() {
    renderedPages.clear();
    lowRenderingPages.clear();
    highRenderingPages.clear();
    loadedLinks.clear();
    const slots = scrollEl?.querySelectorAll('.page-slot') as NodeListOf<HTMLElement> | undefined;
    if (!slots) return;
    slots.forEach((slot) => {
      const pageNum = parseInt(slot.dataset.page || '-1');
      if (pageNum < 0) return;
      slot.innerHTML = '';
      delete slot.dataset.renderQuality;
      delete slot.dataset.renderGeneration;
      const inView = pageOffsets[pageNum + 1] > (scrollEl?.scrollTop || 0) - BUFFER_PAGES * (scrollEl?.clientHeight || 0)
        && pageOffsets[pageNum] < (scrollEl?.scrollTop || 0) + (scrollEl?.clientHeight || 0) + BUFFER_PAGES * (scrollEl?.clientHeight || 0);
      if (inView) {
        const highCached = pageCache.get(pageNum);
        if (highCached) {
          renderPageFromCache(pageNum, slot, highCached, renderGeneration, 'high-res-cached');
        } else {
          const lowCached = lowResCache.get(pageNum);
          if (lowCached) renderPageFromCache(pageNum, slot, lowCached, renderGeneration, 'low-res-cached');
          renderPage(pageNum, slot);
        }
      }
    });
  }

  // Update CSS dimensions of all rendered canvases/wrappers to match current scale
  function updateCanvasSizes() {
    for (const [pageNum, canvas] of canvasRefs) {
      const dim = pageDimensions[pageNum];
      if (!dim) continue;
      const wrapper = canvas.parentElement as HTMLElement | null;
      if (wrapper && wrapper.classList.contains('pdf-page-wrapper')) {
        wrapper.style.width = `${dim.width * scale}px`;
        wrapper.style.height = `${dim.height * scale}px`;
      }
    }
  }

  // Reactive: sync canvas/wrapper sizes whenever scale changes
  $effect(() => {
    // Depend on scale — triggers re-run when zoom changes
    const s = scale;
    // Run after Svelte flushes DOM (page-slot heights already updated)
    updateCanvasSizes();
  });

  // Zoom — only update CSS, no canvas destroy/re-render
  function applyZoom(newScale: number) {
    if (!scrollEl || pageDimensions.length === 0) return;
    const oldScale = scale;
    newScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, newScale));
    if (newScale === oldScale) return;

    // Preserve scroll center position
    const center = scrollEl.scrollTop + scrollEl.clientHeight / 2;
    const ratio = center / totalHeight;

    scale = newScale;
    // Canvas/wrapper sizes are updated reactively via $effect on scale

    // Restore scroll position after DOM update
    requestAnimationFrame(() => {
      if (!scrollEl) return;
      const newCenter = ratio * totalHeight;
      scrollEl.scrollTop = newCenter - scrollEl.clientHeight / 2;
      // Render any newly visible pages (due to size change)
      updatePageRendering();
    });
  }

  // Keyboard handler
  function handleKeydown(event: KeyboardEvent) {
    if (showSearch) {
      if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); closeSearch(); return; }
      if (event.key === 'Enter') { event.preventDefault(); event.stopPropagation(); if (searchState.results.length > 0) navigateMatch(event.shiftKey ? 'prev' : 'next'); else performSearch(); return; }
      event.stopPropagation();
      return;
    }

    if (event.ctrlKey && event.code === 'KeyS') return;
    if (event.ctrlKey && event.code === 'KeyL') return;

    // Ctrl+=/- for zoom
    if (event.ctrlKey && (event.code === 'Equal' || event.code === 'NumpadAdd')) {
      event.preventDefault(); event.stopPropagation();
      applyZoom(scale + ZOOM_STEP);
      return;
    }
    if (event.ctrlKey && (event.code === 'Minus' || event.code === 'NumpadSubtract')) {
      event.preventDefault(); event.stopPropagation();
      applyZoom(scale - ZOOM_STEP);
      return;
    }

    // Shift+E for fullscreen
    if (event.shiftKey && event.code === 'KeyE') {
      event.preventDefault(); event.stopPropagation(); onFullscreen(); return;
    }

    switch (event.code) {
      case 'KeyJ': cancelPendingJump(); event.preventDefault(); event.stopPropagation(); scrollEl?.scrollBy(0, SCROLL_STEP); break;
      case 'KeyK': cancelPendingJump(); event.preventDefault(); event.stopPropagation(); scrollEl?.scrollBy(0, -SCROLL_STEP); break;
      case 'KeyH':
        if (scrollEl && scrollEl.scrollWidth > scrollEl.clientWidth) { cancelPendingJump(); event.preventDefault(); event.stopPropagation(); scrollEl.scrollBy(-SCROLL_STEP, 0); }
        break;
      case 'KeyL':
        if (scrollEl && scrollEl.scrollWidth > scrollEl.clientWidth) { cancelPendingJump(); event.preventDefault(); event.stopPropagation(); scrollEl.scrollBy(SCROLL_STEP, 0); }
        break;
      case 'KeyG':
        cancelPendingJump(); event.preventDefault(); event.stopPropagation();
        if (scrollEl) scrollEl.scrollTop = event.shiftKey ? scrollEl.scrollHeight : 0;
        break;
      case 'KeyN':
        event.preventDefault(); event.stopPropagation();
        if (searchState.results.length > 0) navigateMatch(event.shiftKey ? 'prev' : 'next');
        break;
      case 'Slash':
        event.preventDefault(); event.stopPropagation();
        showSearch = true;
        setTimeout(() => searchInput?.focus(), 0);
        break;
    }
  }

  // Mouse wheel: scroll or Ctrl+zoom
  function handleWheel(e: WheelEvent) {
    if (!hasFocus && !isHovered) return;

    if (e.ctrlKey) {
      e.preventDefault();
      e.stopPropagation();
      const delta = e.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP;
      applyZoom(scale + delta);
    }
    // Non-Ctrl wheel: let native scroll handle it
    else cancelPendingJump();
  }

  // Search
  async function performSearch() {
    if (!searchQuery.trim()) { searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = ''; return; }
    isSearching = true; searchStatus = 'Searching...';
    try {
      const results = await searchPdfText(pdfPath, searchQuery);
      const totalMatches = results.reduce((sum, r) => sum + r.matches.length, 0);
      if (totalMatches === 0) { searchState = { results, currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = 'No results'; }
      else {
        searchState = { results, currentMatchPage: results[0].page, currentMatchIndex: 0 };
        searchStatus = `${totalMatches} matches`;
        scrollToPage(results[0].page);
        // Re-render to show highlights
        requestAnimationFrame(() => rerenderVisiblePages());
      }
    } catch { searchStatus = 'Search failed'; }
    finally { isSearching = false; }
  }

  function navigateMatch(direction: 'next' | 'prev') {
    const target = navigateSearchMatch(direction, searchState);
    if (!target) return;
    searchState = { ...searchState, currentMatchPage: target.page, currentMatchIndex: target.index };
    scrollToPage(target.page);
    requestAnimationFrame(() => rerenderVisiblePages());
  }

  function closeSearch() {
    showSearch = false; searchQuery = '';
    searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = '';
    requestAnimationFrame(() => rerenderVisiblePages());
  }

  // Scroll to a specific page top
  export function scrollToPage(pageNum: number, pdfY = 0) {
    if (!scrollEl || pageNum < 0 || pageNum >= totalPages) return;
    lastRenderRangeStart = -1;
    lastRenderRangeEnd = -1;
    schedulePageJump(pageNum, pdfY);
  }

  // Focus management
  export function focusPanel() { containerEl?.focus(); }

  // Reload when pdfPath changes
  let lastPath = '';
  $effect(() => {
    const path = pdfPath;
    if (path && path !== lastPath) {
      lastPath = path;
      scaleInitialized = false;
      pageCache.clear();
      lowResCache.clear();
      clearLinkCache();
      renderedPages.clear();
      lowRenderingPages.clear();
      highRenderingPages.clear();
      loadedLinks.clear();
      canvasRefs.clear();
      renderGeneration++;
      lastRenderRangeStart = -1;
      lastRenderRangeEnd = -1;
      renderScheduler.clear();
      invoke('clear_pdf_cache').catch(() => {});
      isLoading = true;
      hasError = false;
      isLoading = false;
    }
  });

  // Init scale and render when both pageDimensions and scrollEl are available
  $effect(() => {
    const dims = pageDimensions;
    const el = scrollEl;
    if (dims.length > 0 && el) {
      viewportWidth = el.clientWidth;
      // Double rAF ensures browser has completed layout
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          initScale();
          updatePageRendering();
        });
      });
    }
  });

  onMount(() => {
    resizeObserver = new ResizeObserver(() => {
      if (scrollEl) viewportWidth = scrollEl.clientWidth;
      initScale();
      requestAnimationFrame(() => {
        updatePageRendering();
      });
    });
    if (scrollEl) resizeObserver.observe(scrollEl);
  });

  // Set up scroll listener and observer when scrollEl becomes available
  $effect(() => {
    const el = scrollEl;
    if (el) {
      // Scroll listener
      let ticking = false;
      const handler = () => {
        if (!ticking) {
          requestAnimationFrame(() => {
            updateCurrentPage();
            updatePageRendering();
            ticking = false;
          });
          ticking = true;
        }
      };
      el.addEventListener('scroll', handler, { passive: true });
      // Resize observer
      if (resizeObserver) resizeObserver.observe(el);
      return () => el.removeEventListener('scroll', handler);
    }
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    pageCache.clear();
    lowResCache.clear();
    lowRenderingPages.clear();
    highRenderingPages.clear();
    renderScheduler.clear();
    invoke('clear_pdf_cache').catch(() => {});
    clearLinkCache();
  });

  const position = $derived(`${currentPage + 1}/${totalPages}`);
  const zoomPercent = $derived(`${Math.round(scale * 100)}%`);
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="pdf-preview-panel"
  bind:this={containerEl}
  tabindex="0"
  onfocus={() => hasFocus = true}
  onblur={() => hasFocus = false}
  onmouseenter={() => isHovered = true}
  onmouseleave={() => isHovered = false}
  onkeydown={handleKeydown}
  onwheel={handleWheel}
  role="application"
  aria-label="PDF Preview Panel"
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
          if (e.key === 'Escape') { e.stopPropagation(); closeSearch(); }
          if (e.key === 'Enter') { e.stopPropagation(); if (searchState.results.length > 0) navigateMatch(e.shiftKey ? 'prev' : 'next'); else performSearch(); }
          e.stopPropagation();
        }}
      />
      {#if searchStatus}
        <span class="pdf-search-status">{searchStatus}</span>
      {/if}
    </div>
  {/if}

  {#if isLoading && pageDimensions.length === 0}
    <div class="pdf-scroll-area">
      <div class="pdf-status">Loading...</div>
    </div>
  {:else if hasError}
    <div class="pdf-scroll-area">
      <div class="pdf-status error">{errorMessage}</div>
    </div>
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="pdf-scroll-area" bind:this={scrollEl} onclick={() => containerEl?.focus()}>
      <div class="pdf-page-track" style="width: {pageTrackWidth}px;">
        {#each { length: totalPages } as _, i}
          <div
            class="page-slot"
            data-page={i}
            style="height: {(pageDimensions[i]?.height || 0) * scale}px;"
          ></div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="pdf-info-bar">
    <span class="pdf-info-page">{position}</span>
    <span class="pdf-info-zoom">{zoomPercent}</span>
    {#if fileSize > 0}
      <span class="pdf-info-size">{formatSize(fileSize)}</span>
    {/if}
    <span class="pdf-info-hints">j/k滚动 Ctrl+=/-缩放 /搜索 E全屏</span>
  </div>
</div>

<style>
  .pdf-preview-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    outline: none;
    position: relative;
    overflow: hidden;
    background-color: var(--bg-primary);
  }

  .pdf-search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    z-index: 10;
    font-family: var(--font-mono);
  }

  .pdf-search-input {
    flex: 1;
    max-width: 300px;
    padding: 3px 8px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
    outline: none;
  }

  .pdf-search-input:focus {
    border-color: var(--border-focus);
  }

  .pdf-search-status {
    color: var(--text-muted);
    font-size: 11px;
  }

  .pdf-scroll-area {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
    position: relative;
  }

  .pdf-page-track {
    min-height: 100%;
  }

  :global(.page-slot) {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    overflow: visible;
    position: relative;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-secondary);
  }

  :global(.pdf-canvas) {
    user-select: none;
    -webkit-user-drag: none;
    display: block;
    width: 100%;
    height: 100%;
  }

  :global(.pdf-page-wrapper) {
    flex-shrink: 0;
  }

  :global(.pdf-page-loading) {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 12px;
    font-family: var(--font-mono);
    background-color: var(--bg-secondary);
  }

  :global(.pdf-page-error) {
    color: var(--text-muted);
    font-size: 13px;
    font-family: var(--font-mono);
    padding: 20px;
    text-align: center;
  }

  :global(.link-overlay) {
    pointer-events: none;
    z-index: 5;
  }

  :global(.link-rect) {
    pointer-events: auto;
    cursor: pointer;
    background-color: transparent;
    border: none;
  }

  :global(.link-rect:hover) {
    background-color: rgba(100, 149, 237, 0.15);
    outline: 1px solid rgba(100, 149, 237, 0.4);
  }

  .pdf-status {
    color: var(--text-muted);
    font-size: 13px;
    font-family: var(--font-mono);
    padding: 20px;
    text-align: center;
  }

  .pdf-status.error {
    color: var(--error);
  }

  .pdf-info-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 3px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 11px;
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  .pdf-info-page {
    color: var(--text-primary);
  }

  .pdf-info-zoom {
    color: var(--text-muted);
  }

  .pdf-info-size {
    color: var(--text-muted);
  }

  .pdf-info-hints {
    margin-left: auto;
    color: var(--text-muted);
    font-size: 10px;
  }
</style>
