<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { formatSize } from '$lib/utils/file-types';
  import { logInfo, logError } from '$lib/utils/log';
  import { PDF_ZOOM_KEY_STEP, wheelZoomDelta, nextZoomScale, advanceZoomScale } from '$lib/utils/pdf-zoom';
  import {
    type PdfPageDimensions,
    type PdfSearchState,
    type PdfLinkAnnotation,
    PdfTextLayoutCache,
    renderPdfTextLayer,
    shouldExecutePdfSearch,
    copyPdfTextSelection,
    searchPdfText,
    navigateSearchMatch,
    cumulativePageOffsets,
    pageAtScrollPosition,
    fetchPdfPageLinks,
    clearLinkCache,
    PdfRenderScheduler,
    PdfTileCache,
    type PdfTileData,
    type PdfTileSpec,
    cappedPdfDevicePixelRatio,
    fetchPdfTile,
    getPdfTileSpecs,
    getPdfBaseSpec,
    pdfTileViewportDistance,
    PDF_MIN_RENDER_DENSITY,
    quantizePdfZoom,
  } from '$lib/utils/pdf-shared';

  let {
    pdfPath,
    pageDimensions = [],
    pageCount = 0,
    fileSize = 0,
    title = null,
    onPageChange = (_page: number) => {},
    onFullscreen = () => {},
    active = true,
    panelDragging = false,
  }: {
    pdfPath: string;
    pageDimensions?: PdfPageDimensions[];
    pageCount?: number;
    fileSize?: number;
    title?: string | null;
    onPageChange?: (page: number) => void;
    onFullscreen?: () => void;
    active?: boolean;
    panelDragging?: boolean;
  } = $props();

  // State
  let currentPage = $state(0);
  let totalPages = $derived(pageCount);
  let scale = $state(1);
  let scaleInitialized = false;
  let pendingRestoreScale = 0;
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
  let executedQuery = '';
  let searchRequest = 0;

  // Cache
  let tileCache = new PdfTileCache();
  let renderScheduler = new PdfRenderScheduler();
  let linkCache = new Map<number, PdfLinkAnnotation[]>();
  let textCache = new PdfTextLayoutCache();
  let loadedText = new Map<number, string>();
  let selectingText = false;
  let textPointerStart: { x: number; y: number } | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let observedWidth = -1;
  let resizeFrame: number | undefined;
  let resizeTimer: ReturnType<typeof setTimeout> | undefined;
  let resizing = false;
  let destroyed = false;

  function invalidateViewport() {
    renderGeneration++;
    renderScheduler.invalidate(renderGeneration);
  }

  function finishResize() {
    if (resizeTimer !== undefined) clearTimeout(resizeTimer);
    resizeTimer = undefined;
    if (panelDragging || destroyed) return;
    resizing = false;
    updatePageRendering();
  }

  function observeWidth() {
    if (!scrollEl) return;
    const width = scrollEl.clientWidth;
    if (width === observedWidth) return;
    observedWidth = width;
    if (resizeFrame !== undefined) return;
    resizeFrame = requestAnimationFrame(() => {
      resizeFrame = undefined;
      if (!scrollEl || scrollEl.clientWidth === viewportWidth) return;
      viewportWidth = scrollEl.clientWidth;
      initScale();
      if (!resizing) invalidateViewport();
      resizing = true;
      if (resizeTimer !== undefined) clearTimeout(resizeTimer);
      resizeTimer = setTimeout(finishResize, 180);
    });
  }

  let renderedPages = new Set<number>();
  // Track which pages have link overlays loaded
  let loadedLinks = new Set<number>();
  // Request cancellation: incremented on viewport changes to skip stale renders
  let renderGeneration = 0;
  let renderRetryScheduled = false;
  let jumpToken = 0;
  let programmaticScroll = false;
  let jumpOverflowAnchor = '';
  let zoomSettled = false;
  let zoomRenderTimer: ReturnType<typeof setTimeout> | undefined;
  let pathRevision = '';
  let tileViewSignature = '';
  let lastScrollTop = 0;
  let lastScrollTime = 0;
  let scrollVelocity = 0;
  let isScrolling = false;
  let keyScrollActive = false;
  let scrollSettledTimer: ReturnType<typeof setTimeout> | undefined;
  let renderScaleKey = '';
  let peakQueueLength = 0;
  let peakCacheBytes = 0;
  let lastPeakLogTime = 0;
  let renderingTiles = new Set<string>();
  let renderedTileData = new Map<string, PdfTileData>();
  let requiredVisibleHighTileKeys = new Set<string>();
  let completedVisibleHighTileKeys = new Set<string>();
  let failedVisibleHighTileKeys = new Set<string>();
  let visibleHighRetryCounts = new Map<string, number>();
  let visibleHighGeneration = -1;
  let lowBatch: { generation: number; pending: Set<string>; timer: ReturnType<typeof setTimeout> } | undefined;
  let lowBatchStartedGeneration = -1;

  function releaseLowBatch() {
    const batch = lowBatch;
    if (!batch) return;
    clearTimeout(batch.timer);
    for (const canvas of scrollEl?.querySelectorAll<HTMLCanvasElement>('.pdf-tile-canvas[data-tile-quality="low"]') || []) {
      canvas.style.visibility = '';
    }
    lowBatch = undefined;
    if (active && batch.generation === renderGeneration) requestAnimationFrame(updatePageRendering);
  }

  type ZoomAnchor = {
    page: number;
    pdfX: number;
    pdfY: number;
    viewportX: number;
    viewportY: number;
  };

  type ZoomSession = {
    anchor: ZoomAnchor;
    targetScale: number;
    rafId: number | undefined;
    settleTimer: ReturnType<typeof setTimeout> | undefined;
    overflowAnchor: string;
    lastFrameTime: number;
    inputPending: boolean;
  };

  let zoomSession: ZoomSession | undefined;

  const BUFFER_PAGES = 1;
  const SCROLL_STEP = 40;
  const MIN_SCALE = 0.3;
  const MAX_SCALE = 5.0;

  function scheduleRenderRetry() {
    if (!active) return;
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

  // Page slot heights based on dimensions and scale
  let pageOffsets = $derived(cumulativePageOffsets(pageDimensions, scale));
  let totalHeight = $derived(pageOffsets.length > 0 ? pageOffsets[pageOffsets.length - 1] : 0);

  // Initialize scale to fit first page width
  function initScale() {
    if (scaleInitialized || !scrollEl || pageDimensions.length === 0) return;
    // A tab restore may set the scale before this runs; keep it instead of
    // fitting the page, otherwise the restored zoom is discarded.
    if (pendingRestoreScale > 0) {
      scale = pendingRestoreScale;
      scaleInitialized = true;
      return;
    }
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

  function tileStyle(canvas: HTMLElement, data: PdfTileData) {
    canvas.style.left = `${(data.tileX / data.scale) * scale}px`;
    canvas.style.top = `${(data.tileY / data.scale) * scale}px`;
    canvas.style.width = `${(data.width / data.scale) * scale}px`;
    canvas.style.height = `${(data.height / data.scale) * scale}px`;
  }

  function drawTileHighlights(context: CanvasRenderingContext2D, data: PdfTileData) {
    const pageMatches = searchState.results.find(result => result.page === data.page);
    const pageHeight = pageDimensions[data.page]?.height;
    if (!pageMatches || !pageHeight) return;
    for (let index = 0; index < pageMatches.matches.length; index++) {
      const match = pageMatches.matches[index];
      context.fillStyle = data.page === searchState.currentMatchPage && index === searchState.currentMatchIndex
        ? 'rgba(255, 165, 0, 0.5)'
        : 'rgba(255, 255, 0, 0.35)';
      context.fillRect(
        match.x * data.scale - data.tileX,
        (pageHeight - match.y - match.height) * data.scale - data.tileY,
        match.width * data.scale,
        match.height * data.scale,
      );
    }
  }

  function getTileLayer(pageNum: number, slot: HTMLElement): HTMLElement {
    let wrapper = slot.querySelector<HTMLElement>(':scope > .pdf-page-wrapper');
    if (!wrapper) {
      slot.innerHTML = '';
      wrapper = document.createElement('div');
      wrapper.className = 'pdf-page-wrapper';
      wrapper.style.position = 'relative';
      const layer = document.createElement('div');
      layer.className = 'pdf-tile-layer';
      wrapper.appendChild(layer);
      slot.appendChild(wrapper);
    }
    const dim = pageDimensions[pageNum];
    if (dim) {
      wrapper.style.width = `${dim.width * scale}px`;
      wrapper.style.height = `${dim.height * scale}px`;
    }
    if (!isScrolling && !keyScrollActive) {
      loadLinkOverlay(pageNum, wrapper);
      loadTextLayer(pageNum, wrapper);
    }
    return wrapper.querySelector<HTMLElement>('.pdf-tile-layer')!;
  }

  async function loadTextLayer(pageNum: number, wrapper: HTMLElement) {
    const revision = pathRevision;
    const dim = pageDimensions[pageNum];
    if (!dim || loadedText.has(pageNum)) return;
    loadedText.set(pageNum, revision);
    try {
      const layout = await textCache.get(revision, pdfPath, pageNum);
      if (!wrapper.isConnected || pathRevision !== revision || loadedText.get(pageNum) !== revision) return;
      const layer = document.createElement('div');
      layer.className = 'pdf-text-layer';
      layer.dataset.page = String(pageNum);
      layer.dataset.renderScale = String(scale);
      renderPdfTextLayer(layer, layout, dim.height, scale);
      layer.addEventListener('pointerdown', event => {
        textPointerStart = { x: event.clientX, y: event.clientY };
      });
      layer.addEventListener('click', event => {
        if (!textPointerStart || Math.hypot(event.clientX - textPointerStart.x, event.clientY - textPointerStart.y) > 4 || !window.getSelection()?.isCollapsed) return;
        const link = [...wrapper.querySelectorAll<HTMLElement>('.link-rect')]
          .find(rect => {
            const bounds = rect.getBoundingClientRect();
            return event.clientX >= bounds.left && event.clientX <= bounds.right && event.clientY >= bounds.top && event.clientY <= bounds.bottom;
          });
        link?.click();
      });
      wrapper.appendChild(layer);
    } catch {
      loadedText.delete(pageNum);
    }
  }

  function updateTextLayers() {
    for (const layer of scrollEl?.querySelectorAll<HTMLElement>('.pdf-text-layer') || []) {
      const renderedAt = Number(layer.dataset.renderScale) || scale;
      layer.style.transformOrigin = 'top left';
      layer.style.transform = `scale(${scale / renderedAt})`;
    }
  }

  function appendTile(
    layer: HTMLElement,
    data: PdfTileData,
    generation: number,
    onInserted?: () => void,
    onFailed?: () => void,
  ) {
    const tileKey = data.key || `${data.page}:${data.scale}:${data.tileX}:${data.tileY}`;
    const existing = Array.from(layer.querySelectorAll<HTMLElement>('.pdf-tile-canvas'))
      .find(canvas => canvas.dataset.tileKey === tileKey);
    if (existing) {
      tileStyle(existing, data);
      onInserted?.();
      return;
    }
    if (data.key?.includes(':base:')) {
      const binary = atob(data.data);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index++) bytes[index] = binary.charCodeAt(index);
      const url = URL.createObjectURL(new Blob([bytes.buffer], { type: data.format === 'jpeg' ? 'image/jpeg' : 'image/png' }));
      const base = new Image();
      base.className = 'pdf-tile-canvas';
      base.dataset.tileKey = tileKey;
      base.dataset.tileQuality = 'low';
      base.style.zIndex = '1';
      tileStyle(base, data);
      base.onload = () => {
        URL.revokeObjectURL(url);
        if (generation !== renderGeneration || !layer.isConnected) return;
        const highScale = Number(renderScaleKey.split(':').pop()).toFixed(3);
        for (const old of layer.querySelectorAll<HTMLElement>('.pdf-tile-canvas')) {
          const key = old.dataset.tileKey || '';
          if (key !== tileKey && !key.startsWith(`${pathRevision}:${data.page}:${highScale}:`)) {
            renderedTileData.delete(key);
            old.remove();
          }
        }
        layer.appendChild(base);
        renderedTileData.set(tileKey, data);
        onInserted?.();
      };
      base.onerror = () => { URL.revokeObjectURL(url); onFailed?.(); };
      base.src = url;
      return;
    }
    const canvas = document.createElement('canvas');
    canvas.className = 'pdf-tile-canvas';
    canvas.dataset.tileKey = tileKey;
    canvas.dataset.tileQuality = data.lowRes ? 'low' : 'high';
    canvas.style.zIndex = data.lowRes ? '1' : '2';
    canvas.width = data.width;
    canvas.height = data.height;
    tileStyle(canvas, data);
    const binary = atob(data.data);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index++) bytes[index] = binary.charCodeAt(index);
    const blob = new Blob([bytes.buffer], { type: data.format === 'jpeg' ? 'image/jpeg' : 'image/png' });
    const url = URL.createObjectURL(blob);
    const image = new Image();
    image.onload = () => {
      URL.revokeObjectURL(url);
      if (generation !== renderGeneration || !layer.isConnected) return;
      const context = canvas.getContext('2d');
      if (!context) return;
      context.drawImage(image, 0, 0);
      drawTileHighlights(context, data);
      layer.appendChild(canvas);
      renderedTileData.set(tileKey, data);
      onInserted?.();
    };
    image.onerror = () => {
      URL.revokeObjectURL(url);
      onFailed?.();
    };
    image.src = url;
  }

  function queueTile(
    spec: PdfTileSpec,
    layer: HTMLElement,
    priority: number,
    generation: number,
    onInserted?: () => void,
  ) {
    const retryVisibleHighTile = () => {
      if (!onInserted || generation !== renderGeneration || visibleHighGeneration !== generation) return;
      const attempts = visibleHighRetryCounts.get(spec.key) || 0;
      if (attempts >= 1) {
        failedVisibleHighTileKeys.add(spec.key);
        return;
      }
      visibleHighRetryCounts.set(spec.key, attempts + 1);
      setTimeout(() => queueTile(spec, layer, priority, generation, onInserted), 50);
    };
    const cached = tileCache.get(spec.key);
    if (cached) {
      appendTile(layer, cached, generation, onInserted, retryVisibleHighTile);
      return;
    }
    if (renderingTiles.has(spec.key)) { renderScheduler.reprioritize(spec.key, priority); return; }
    renderingTiles.add(spec.key);
    const revision = pathRevision;
    const scaleKey = renderScaleKey;
    renderScheduler.enqueue(
      spec.key,
      () => fetchPdfTile(pdfPath, spec, tileCache, () => revision === pathRevision && scaleKey === renderScaleKey),
      priority,
    ).then(data => {
      if (generation === renderGeneration) appendTile(layer, data, generation, onInserted, retryVisibleHighTile);
    }).catch(error => {
      if (generation === renderGeneration) {
        logError('PdfPreviewPanel', `Failed to render PDF tile ${spec.key}: ${error}`);
        if (spec.lowRes && lowBatch?.generation === generation) {
          lowBatch.pending.delete(spec.key);
          if (lowBatch.pending.size === 0) requestAnimationFrame(releaseLowBatch);
        }
        retryVisibleHighTile();
      }
    }).finally(() => {
      renderingTiles.delete(spec.key);
      if (generation !== renderGeneration) scheduleRenderRetry();
    });
  }

  function updateTileLayouts() {
    for (const wrapper of scrollEl?.querySelectorAll<HTMLElement>('.pdf-page-wrapper') || []) {
      const pageNum = Number(wrapper.parentElement?.dataset.page);
      const dim = pageDimensions[pageNum];
      if (!dim) continue;
      wrapper.style.width = `${dim.width * scale}px`;
      wrapper.style.height = `${dim.height * scale}px`;
    }
    for (const canvas of scrollEl?.querySelectorAll<HTMLElement>('.pdf-tile-canvas') || []) {
      const data = renderedTileData.get(canvas.dataset.tileKey || '');
      if (data) tileStyle(canvas, data);
    }
  }

  // Load and render link annotation overlay
  async function loadLinkOverlay(pageNum: number, slot: HTMLElement) {
    if (loadedLinks.has(pageNum)) return;
    loadedLinks.add(pageNum);

    try {
      const links = await fetchPdfPageLinks(pdfPath, pageNum);
      if (links.length === 0) { logInfo('pdf', `links p${pageNum}: 0`); return; }
      logInfo('pdf', `links p${pageNum}: ${links.length} items`);

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

  function getPageScrollTop(pageNum: number, pdfY = 0, center = false): number {
    if (!scrollEl || pageNum < 0 || pageNum >= pageDimensions.length) return 0;
    const slot = scrollEl.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
    const pageTop = slot
      ? slot.offsetTop
      : pageOffsets[pageNum] || 0;
    const pageHeight = pageDimensions[pageNum]?.height || 0;
    const hasExplicitY = Number.isFinite(pdfY) && pdfY > 0 && pdfY < pageHeight;
    const yOffset = hasExplicitY ? (pageHeight - pdfY) * scale : 0;
    const target = Math.max(0, pageTop + yOffset - (center ? scrollEl.clientHeight / 2 : 0));
    return target;
  }

  function schedulePageJump(pageNum: number, pdfY = 0, center = false, pdfX = 0) {
    if (!scrollEl || pageNum < 0 || pageNum >= pageDimensions.length) return;
    const token = ++jumpToken;
    renderGeneration++;
    renderScheduler.invalidate(renderGeneration);
    if (!programmaticScroll) jumpOverflowAnchor = scrollEl.style.overflowAnchor;
    scrollEl.style.overflowAnchor = 'none';
    programmaticScroll = true;

    requestAnimationFrame(() => requestAnimationFrame(() => {
      if (!scrollEl || token !== jumpToken) return;
      scrollEl.scrollTop = getPageScrollTop(pageNum, pdfY, center);
      if (center) scrollEl.scrollLeft = Math.max(0, (pageTrackWidth - pageDimensions[pageNum].width * scale) / 2 + pdfX * scale - scrollEl.clientWidth / 2);
      requestAnimationFrame(() => {
        if (!scrollEl || token !== jumpToken) return;
        scrollEl.scrollTop = getPageScrollTop(pageNum, pdfY, center);
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
    loadedLinks.delete(pageNum);
    loadedText.delete(pageNum);
    for (const [tileKey, tile] of renderedTileData) {
      if (tile.page === pageNum) renderedTileData.delete(tileKey);
    }
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

  function updateTiledPageRendering() {
    if (destroyed || !active || !scrollEl || pageOffsets.length < 2) return;
    if (resizing || panelDragging) return;
    if (zoomSession && renderedTileData.size > 0) return;
    const viewport = {
      left: scrollEl.scrollLeft,
      top: scrollEl.scrollTop,
      width: scrollEl.clientWidth,
      height: scrollEl.clientHeight,
    };
    const buffer = BUFFER_PAGES * viewport.height;
    const rangeStart = Math.max(0, findPageAtOffset(viewport.top - buffer));
    const rangeEnd = Math.min(totalPages - 1, findPageAtOffset(viewport.top + viewport.height + buffer));
    const center = findPageAtOffset(viewport.top + viewport.height / 2);
    const zoomBucket = quantizePdfZoom(scale);
    const highScale = zoomBucket * Math.max(PDF_MIN_RENDER_DENSITY, cappedPdfDevicePixelRatio());
    const nextScaleKey = `${pathRevision}:${zoomBucket}:${highScale}`;
    if (renderScaleKey !== nextScaleKey) {
      renderScaleKey = nextScaleKey;
      renderScheduler.clear();
      renderingTiles.clear();
    }
    const now = performance.now();
    const elapsed = now - lastScrollTime;
    if (elapsed > 0 && elapsed < 250) scrollVelocity = (viewport.top - lastScrollTop) / elapsed;
    else if (elapsed >= 250) scrollVelocity = 0;
    lastScrollTime = now;
    const direction = Math.sign(scrollVelocity);
    const visibleStart = findPageAtOffset(viewport.top);
    const visibleEnd = findPageAtOffset(viewport.top + viewport.height);
    const ahead = Math.min(5, 1 + Math.ceil(Math.abs(scrollVelocity) * 2));
    const prefetchStart = Math.max(0, visibleStart - (direction < 0 ? ahead : 1));
    const prefetchEnd = Math.min(totalPages - 1, visibleEnd + (direction > 0 ? ahead : 1));
    const visibleHighRequests: { spec: PdfTileSpec; priority: number; page: number }[] = [];
    const visibleLowRequests: { spec: PdfTileSpec; priority: number; page: number }[] = [];
    const prefetchHighRequests: { spec: PdfTileSpec; priority: number; page: number }[] = [];
    const prefetchLowRequests: { spec: PdfTileSpec; priority: number; page: number }[] = [];
    const layers = new Map<number, HTMLElement>();

    const pageOrder = Array.from({ length: rangeEnd - rangeStart + 1 }, (_, index) => rangeStart + index)
      .sort((left, right) => Math.abs(left - center) - Math.abs(right - center));
    for (const pageNum of pageOrder) {
      const slot = scrollEl.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
      const dim = pageDimensions[pageNum];
      if (!slot || !dim) continue;
      renderedPages.add(pageNum);
      const layer = getTileLayer(pageNum, slot);
      layers.set(pageNum, layer);
      const pageLeft = (pageTrackWidth - dim.width * scale) / 2;
      const base = getPdfBaseSpec(pathRevision, pageNum, dim, zoomBucket);
      const hasTemporaryImage = layer.querySelector('.pdf-tile-canvas') !== null;
      if (!hasTemporaryImage) {
        (pageNum >= visibleStart && pageNum <= visibleEnd ? visibleLowRequests : prefetchLowRequests)
          .push({ spec: base, page: pageNum, priority: pageNum >= visibleStart && pageNum <= visibleEnd
            ? Math.abs(pageNum - center) : 10_000 + Math.abs(pageNum - (direction >= 0 ? visibleEnd : visibleStart)) });
      }
      if (zoomSettled && !isScrolling && !keyScrollActive) {
        const highVisible = getPdfTileSpecs(pathRevision, pageNum, dim, pageLeft, pageOffsets[pageNum], scale, highScale, viewport);
        const highPrefetch = panelDragging ? [] : getPdfTileSpecs(pathRevision, pageNum, dim, pageLeft, pageOffsets[pageNum], scale, highScale, viewport, true);
        visibleHighRequests.push(...highVisible.map(spec => ({
          spec,
          page: pageNum,
          priority: pdfTileViewportDistance(spec, pageLeft, pageOffsets[pageNum], scale, viewport),
        })));
        prefetchHighRequests.push(...highPrefetch
          .filter(spec => !highVisible.some(visible => visible.key === spec.key))
          .map(spec => ({
            spec,
            page: pageNum,
            priority: 200_000 + pdfTileViewportDistance(spec, pageLeft, pageOffsets[pageNum], scale, viewport),
          })));
      }
    }

    for (let pageNum = prefetchStart; pageNum <= prefetchEnd; pageNum++) {
      if (layers.has(pageNum)) continue;
      const dim = pageDimensions[pageNum];
      if (dim) prefetchLowRequests.push({ spec: getPdfBaseSpec(pathRevision, pageNum, dim, zoomBucket),
        page: pageNum, priority: 10_000 + Math.abs(pageNum - (direction >= 0 ? visibleEnd : visibleStart)) });
    }
    const visibleRequests = [...visibleHighRequests, ...visibleLowRequests];
    const signature = `${pathRevision}:${zoomSettled}:${visibleRequests.map(request => request.spec.key).sort().join('|')}`;
    if (signature !== tileViewSignature || visibleHighGeneration !== renderGeneration) {
      tileViewSignature = signature;
      renderGeneration++;
      visibleHighGeneration = renderGeneration;
      requiredVisibleHighTileKeys = new Set(visibleHighRequests.map(request => request.spec.key));
      completedVisibleHighTileKeys = new Set();
      failedVisibleHighTileKeys = new Set();
      visibleHighRetryCounts = new Map();
    }
    const generation = renderGeneration;
    if (lowBatch) releaseLowBatch();
    tileCache.setCurrentScale(highScale);
    tileCache.setPinned(visibleRequests.map(request => request.spec.key));

    for (const pageNum of Array.from(renderedPages)) {
      if (!selectingText && (pageNum < rangeStart || pageNum > rangeEnd)) {
        const slot = scrollEl.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
        if (slot && !window.getSelection()?.containsNode(slot, true)) destroyPage(pageNum, slot);
      }
    }

    const markVisibleHighComplete = (key: string) => {
      if (generation !== visibleHighGeneration || !requiredVisibleHighTileKeys.has(key)) return;
      completedVisibleHighTileKeys.add(key);
    };
    for (const request of visibleHighRequests) request.priority += 2000;
    for (const request of prefetchHighRequests) request.priority += 3000;
    const orderedRequests = [...visibleLowRequests, ...prefetchLowRequests, ...visibleHighRequests, ...prefetchHighRequests]
      .sort((left, right) => left.priority - right.priority).slice(0, 64);
    renderScheduler.retain(new Set(orderedRequests.map(request => request.spec.key)));
    for (const { spec, priority, page } of orderedRequests) {
      const layer = layers.get(page);
      if (!layer && !spec.base) continue;
      const onInserted = requiredVisibleHighTileKeys.has(spec.key)
        ? () => markVisibleHighComplete(spec.key)
        : lowBatch?.generation === generation && lowBatch.pending.has(spec.key)
          ? () => {
            if (lowBatch?.generation !== generation) return;
            lowBatch.pending.delete(spec.key);
            if (lowBatch.pending.size === 0) requestAnimationFrame(releaseLowBatch);
          }
          : undefined;
      if (layer) queueTile(spec, layer, priority, generation, onInserted);
      else if (!renderingTiles.has(spec.key) && !tileCache.has(spec.key)) {
        renderingTiles.add(spec.key);
        const revision = pathRevision;
        renderScheduler.enqueue(spec.key, () => fetchPdfTile(pdfPath, spec, tileCache,
          () => revision === pathRevision && renderScaleKey === nextScaleKey), priority)
          .catch(() => {}).finally(() => { renderingTiles.delete(spec.key); scheduleRenderRetry(); });
      }
    }
    const queueLength = renderScheduler.getQueueLength();
    const cacheBytes = tileCache.getSizeBytes();
    if (queueLength > peakQueueLength || cacheBytes > peakCacheBytes) {
      peakQueueLength = Math.max(peakQueueLength, queueLength);
      peakCacheBytes = Math.max(peakCacheBytes, cacheBytes);
      if (now - lastPeakLogTime >= 1000) {
        lastPeakLogTime = now;
        logInfo('pdf-perf', `preview peaks queue=${peakQueueLength} cache=${(peakCacheBytes / 1048576).toFixed(1)}MiB`);
      }
    }
    lastScrollTop = viewport.top;
  }

  // Render/destroy pages based on viewport + buffer, and preload beyond buffer
  function updatePageRendering() {
    updateTiledPageRendering();
  }

  // Re-render all visible pages (e.g., after search state change)
  function rerenderVisiblePages() {
    for (const layer of scrollEl?.querySelectorAll<HTMLElement>('.pdf-tile-layer') || []) layer.innerHTML = '';
    renderedTileData.clear();
    updateTiledPageRendering();
  }

  // Update CSS dimensions of all rendered canvases/wrappers to match current scale
  function updateCanvasSizes() {
    updateTileLayouts();
  }

  // Reactive: sync canvas/wrapper sizes whenever scale changes
  $effect(() => {
    // Depend on scale — triggers re-run when zoom changes
    const s = scale;
    // Run after Svelte flushes DOM (page-slot heights already updated)
    updateCanvasSizes();
    updateTextLayers();
  });

  function createZoomAnchor(pointer?: { clientX: number; clientY: number }): ZoomAnchor | undefined {
    if (!scrollEl || pageDimensions.length === 0) return undefined;
    const bounds = scrollEl.getBoundingClientRect();
    const viewportX = Math.max(0, Math.min(scrollEl.clientWidth, (pointer?.clientX ?? bounds.left + scrollEl.clientWidth / 2) - bounds.left));
    const viewportY = Math.max(0, Math.min(scrollEl.clientHeight, (pointer?.clientY ?? bounds.top + scrollEl.clientHeight / 2) - bounds.top));
    const page = findPageAtOffset(scrollEl.scrollTop + viewportY);
    const dimensions = pageDimensions[page];
    if (!dimensions) return undefined;
    const trackWidth = Math.max(scrollEl.clientWidth, ...pageDimensions.map(dim => dim.width * scale));
    const pageLeft = (trackWidth - dimensions.width * scale) / 2;

    return {
      page,
      pdfX: (scrollEl.scrollLeft + viewportX - pageLeft) / scale,
      pdfY: Math.max(0, Math.min(dimensions.height, (scrollEl.scrollTop + viewportY - pageOffsets[page]) / scale)),
      viewportX,
      viewportY,
    };
  }

  function scheduleZoomCommit(session: ZoomSession) {
    if (session.rafId !== undefined) return;
    session.rafId = requestAnimationFrame(async (now) => {
      session.rafId = undefined;
      if (zoomSession !== session || !scrollEl) return;

      const elapsed = session.lastFrameTime ? now - session.lastFrameTime : 16;
      session.lastFrameTime = now;
      const commitScale = window.matchMedia('(prefers-reduced-motion: reduce)').matches
        ? session.targetScale : advanceZoomScale(scale, session.targetScale, elapsed);
      if (scale !== commitScale) {
        scale = commitScale;
        await tick();
      }
      if (zoomSession !== session || !scrollEl) return;

      const dimensions = pageDimensions[session.anchor.page];
      if (!dimensions) return;
      const offsets = cumulativePageOffsets(pageDimensions, scale);
      const trackWidth = Math.max(scrollEl.clientWidth, ...pageDimensions.map(dim => dim.width * scale));
      const pageLeft = (trackWidth - dimensions.width * scale) / 2;
      scrollEl.scrollTop = Math.max(0, offsets[session.anchor.page] + session.anchor.pdfY * scale - session.anchor.viewportY);
      scrollEl.scrollLeft = Math.max(0, pageLeft + session.anchor.pdfX * scale - session.anchor.viewportX);

      if (session.targetScale !== commitScale) scheduleZoomCommit(session);
    });
  }

  function finishZoomSession(session: ZoomSession) {
    if (zoomSession !== session) return;
    if (session.inputPending || session.rafId !== undefined || scale !== session.targetScale) {
      session.inputPending = false;
      if (scale !== session.targetScale) scheduleZoomCommit(session);
      session.settleTimer = setTimeout(() => finishZoomSession(session), 16);
      return;
    }
    zoomSession = undefined;
    if (scrollEl) scrollEl.style.overflowAnchor = session.overflowAnchor;
    // A quick direction reversal should reuse the visible image instead of
    // starting PDFium work for a scale that is about to be abandoned.
    if (zoomRenderTimer !== undefined) clearTimeout(zoomRenderTimer);
    zoomRenderTimer = setTimeout(() => {
      zoomRenderTimer = undefined;
      if (destroyed || zoomSession) return;
      zoomSettled = true;
      tileViewSignature = '';
      updatePageRendering();
    }, 350);
  }

  function changeZoom(delta: number, pointer?: { clientX: number; clientY: number }) {
    if (!scrollEl || pageDimensions.length === 0) return;
    if (zoomRenderTimer !== undefined) {
      clearTimeout(zoomRenderTimer);
      zoomRenderTimer = undefined;
    }
    let session = zoomSession;
    if (!session) {
      const anchor = createZoomAnchor(pointer);
      if (!anchor) return;
      session = {
        anchor,
        targetScale: scale,
        rafId: undefined,
        settleTimer: undefined,
        overflowAnchor: scrollEl.style.overflowAnchor,
        lastFrameTime: 0,
        inputPending: false,
      };
      zoomSession = session;
      zoomSettled = false;
      scrollEl.style.overflowAnchor = 'none';
      renderGeneration++;
      renderScheduler.invalidate(renderGeneration);
    }

    // Recalculate from the displayed content for every input, including a moved pointer.
    session.anchor = createZoomAnchor(pointer) ?? session.anchor;
    session.targetScale = nextZoomScale(session.targetScale, delta, MIN_SCALE, MAX_SCALE);
    session.inputPending = true;
    if (session.settleTimer) clearTimeout(session.settleTimer);
    scheduleZoomCommit(session);
    session.settleTimer = setTimeout(() => finishZoomSession(session!), 150);
  }

  // Keyboard handler
  function handleKeydown(event: KeyboardEvent) {
    if (event.code === 'KeyJ' || event.code === 'KeyK') keyScrollActive = true;
    if (showSearch) {
      if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); closeSearch(); return; }
      if (event.key === 'Enter') { event.preventDefault(); event.stopPropagation(); submitSearch(event.shiftKey ? 'prev' : 'next'); return; }
      event.stopPropagation();
      return;
    }

    if (event.ctrlKey && event.code === 'KeyS') return;
    if (event.ctrlKey && event.code === 'KeyL') return;

    // Ctrl+=/- for zoom
    if (event.ctrlKey && (event.code === 'Equal' || event.code === 'NumpadAdd')) {
      event.preventDefault(); event.stopPropagation();
      changeZoom(PDF_ZOOM_KEY_STEP);
      return;
    }
    if (event.ctrlKey && (event.code === 'Minus' || event.code === 'NumpadSubtract')) {
      event.preventDefault(); event.stopPropagation();
      changeZoom(-PDF_ZOOM_KEY_STEP);
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
      const delta = wheelZoomDelta(e.deltaY, e.deltaMode);
      changeZoom(delta, e);
    }
    // Non-Ctrl wheel: let native scroll handle it
    else cancelPendingJump();
  }

  // Search
  async function performSearch() {
    const query = searchQuery.trim();
    const request = ++searchRequest;
    if (!query) { executedQuery = ''; searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = ''; requestAnimationFrame(() => rerenderVisiblePages()); return; }
    isSearching = true; searchStatus = 'Searching...';
    try {
      const results = await searchPdfText(pdfPath, query);
      if (request !== searchRequest) return;
      executedQuery = query;
      const totalMatches = results.reduce((sum, r) => sum + r.matches.length, 0);
      if (totalMatches === 0) { searchState = { results, currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = 'No results'; requestAnimationFrame(() => rerenderVisiblePages()); }
      else {
        searchState = { results, currentMatchPage: results[0].page, currentMatchIndex: 0 };
        searchStatus = `${totalMatches} matches`;
        scrollToMatch(results[0].page, 0);
        // Re-render to show highlights
        requestAnimationFrame(() => rerenderVisiblePages());
      }
    } catch { if (request === searchRequest) searchStatus = 'Search failed'; }
    finally { if (request === searchRequest) isSearching = false; }
  }

  function submitSearch(direction: 'next' | 'prev') {
    if (shouldExecutePdfSearch(searchQuery, executedQuery, isSearching)) void performSearch();
    else navigateMatch(direction);
  }

  function scrollToMatch(pageNum: number, index: number) {
    const match = searchState.results.find(result => result.page === pageNum)?.matches[index];
    if (match) schedulePageJump(pageNum, match.y + match.height / 2, true, match.x + match.width / 2);
  }

  function navigateMatch(direction: 'next' | 'prev') {
    const target = navigateSearchMatch(direction, searchState);
    if (!target) return;
    searchState = { ...searchState, currentMatchPage: target.page, currentMatchIndex: target.index };
    scrollToMatch(target.page, target.index);
    requestAnimationFrame(() => rerenderVisiblePages());
  }

  function closeSearch() {
    showSearch = false; searchQuery = ''; executedQuery = ''; searchRequest++;
    searchState = { results: [], currentMatchPage: 0, currentMatchIndex: 0 }; searchStatus = '';
    requestAnimationFrame(() => rerenderVisiblePages());
  }

  // Scroll to a specific page top
  export function scrollToPage(pageNum: number, pdfY = 0) {
    if (!scrollEl || pageNum < 0 || pageNum >= totalPages) return;
    schedulePageJump(pageNum, pdfY);
  }

  // Direct scrollTop access for cache restore
  export function getScrollTop(): number {
    return scrollEl?.scrollTop ?? 0;
  }

  export function setScrollTop(top: number) {
    if (scrollEl) scrollEl.scrollTop = top;
  }

  export function getScale(): number {
    return scale;
  }

  export function setScale(value: number): void {
    if (!(value > 0)) return;
    if (zoomRenderTimer !== undefined) clearTimeout(zoomRenderTimer);
    zoomRenderTimer = undefined;
    if (zoomSession?.settleTimer) clearTimeout(zoomSession.settleTimer);
    if (zoomSession?.rafId !== undefined) cancelAnimationFrame(zoomSession.rafId);
    if (zoomSession && scrollEl) scrollEl.style.overflowAnchor = zoomSession.overflowAnchor;
    zoomSession = undefined;
    pendingRestoreScale = value;
    scale = value;
    scaleInitialized = true;
  }

  export function resetScale(): void {
    pendingRestoreScale = 0;
    scaleInitialized = false;
  }

  // Focus management
  export function focusPanel(): boolean {
    if (!containerEl) return false;
    containerEl.focus({ preventScroll: true });
    return document.activeElement === containerEl;
  }

  // Reload when pdfPath changes
  let lastPath = '';
  $effect(() => {
    const path = pdfPath;
    if (path && path !== lastPath) {
      lastPath = path;
      scaleInitialized = false;
      tileCache.clear();
      textCache.clear();
      loadedText.clear();
      searchRequest++;
      executedQuery = '';
      renderingTiles.clear();
      renderedTileData.clear();
      pathRevision = `${path}:${Date.now()}`;
      tileViewSignature = '';
      zoomSettled = false;
      if (zoomRenderTimer !== undefined) clearTimeout(zoomRenderTimer);
      zoomRenderTimer = undefined;
      if (zoomSession?.settleTimer) clearTimeout(zoomSession.settleTimer);
      if (zoomSession?.rafId !== undefined) cancelAnimationFrame(zoomSession.rafId);
      if (zoomSession && scrollEl) scrollEl.style.overflowAnchor = zoomSession.overflowAnchor;
      zoomSession = undefined;
      clearLinkCache();
      for (const pageNum of Array.from(renderedPages)) {
        const slot = scrollEl?.querySelector(`.page-slot[data-page="${pageNum}"]`) as HTMLElement | null;
        if (slot) destroyPage(pageNum, slot);
      }
      renderedPages.clear();
      loadedLinks.clear();
      renderGeneration++;
      renderScheduler.clear();
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
          zoomSettled = true;
          updatePageRendering();
        });
      });
    }
  });

  onMount(() => {
    const finishSelection = () => { selectingText = false; updatePageRendering(); };
    window.addEventListener('pointerup', finishSelection);
    const finishKeyScroll = (event: KeyboardEvent) => {
      if (event.code !== 'KeyJ' && event.code !== 'KeyK') return;
      keyScrollActive = false;
      if (!isScrolling) updatePageRendering();
    };
    window.addEventListener('keyup', finishKeyScroll);
    const handleCopy = (event: ClipboardEvent) => copyPdfTextSelection(event, containerEl);
    document.addEventListener('copy', handleCopy);
    resizeObserver = new ResizeObserver(observeWidth);
    if (scrollEl) resizeObserver.observe(scrollEl);
    return () => { window.removeEventListener('pointerup', finishSelection); window.removeEventListener('keyup', finishKeyScroll); document.removeEventListener('copy', handleCopy); };
  });

  // Set up scroll listener and observer when scrollEl becomes available
  $effect(() => {
    const el = scrollEl;
    if (el) {
      // Scroll listener
      let ticking = false;
      const handler = () => {
        isScrolling = true;
        if (scrollSettledTimer !== undefined) clearTimeout(scrollSettledTimer);
        scrollSettledTimer = setTimeout(() => {
          scrollSettledTimer = undefined;
          isScrolling = false;
          updatePageRendering();
        }, 120);
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
    destroyed = true;
    if (scrollSettledTimer !== undefined) clearTimeout(scrollSettledTimer);
    if (zoomRenderTimer !== undefined) clearTimeout(zoomRenderTimer);
    invalidateViewport();
    if (resizeFrame !== undefined) cancelAnimationFrame(resizeFrame);
    if (resizeTimer !== undefined) clearTimeout(resizeTimer);
    releaseLowBatch();
    resizeObserver?.disconnect();
    tileCache.clear();
    textCache.clear();
    renderingTiles.clear();
    renderedTileData.clear();
    if (zoomSession?.settleTimer) clearTimeout(zoomSession.settleTimer);
    if (zoomSession?.rafId !== undefined) cancelAnimationFrame(zoomSession.rafId);
    if (scrollEl && zoomSession) scrollEl.style.overflowAnchor = zoomSession.overflowAnchor;
    zoomSession = undefined;
    renderScheduler.clear();
    clearLinkCache();
  });

  $effect(() => {
    if (panelDragging) {
      if (!resizing) invalidateViewport();
      resizing = true;
      if (resizeTimer !== undefined) clearTimeout(resizeTimer);
      resizeTimer = undefined;
    } else if (resizing) {
      if (resizeFrame !== undefined) cancelAnimationFrame(resizeFrame);
      resizeFrame = undefined;
      if (scrollEl) viewportWidth = scrollEl.clientWidth;
      finishResize();
    }
  });

  $effect(() => {
    if (active) {
      requestAnimationFrame(() => {
        if (scrollEl) viewportWidth = scrollEl.clientWidth;
        updatePageRendering();
      });
    } else {
      renderGeneration++;
      renderScheduler.invalidate(renderGeneration);
    }
  });

  export function getRetainedBytes(): number {
    let canvasBytes = 0;
    for (const data of renderedTileData.values()) canvasBytes += data.width * data.height * 4;
    return Math.max(tileCache.getSizeBytes(), canvasBytes);
  }

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
          if (e.key === 'Enter') { e.stopPropagation(); submitSearch(e.shiftKey ? 'prev' : 'next'); }
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
    <div class="pdf-scroll-area" bind:this={scrollEl}
      onpointerdown={(e) => { if ((e.target as HTMLElement).closest('.pdf-text-layer')) selectingText = true; }}
      onpointerup={() => { selectingText = false; updatePageRendering(); }}
      onclick={() => { if (window.getSelection()?.isCollapsed) containerEl?.focus(); }}>
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

  :global(.pdf-tile-layer) {
    position: absolute;
    inset: 0;
    overflow: hidden;
  }

  :global(.pdf-tile-canvas) {
    position: absolute;
    display: block;
    user-select: none;
    -webkit-user-drag: none;
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
    z-index: 4;
  }
  :global(.pdf-text-layer) {
    position: absolute;
    inset: 0;
    z-index: 5;
    pointer-events: none;
    user-select: text;
  }
  :global(.pdf-text-character) {
    position: absolute;
    display: block;
    color: transparent;
    white-space: pre;
    line-height: 1;
    overflow: hidden;
    pointer-events: auto;
    user-select: text;
  }
  :global(.pdf-text-character::selection) { background: rgba(70, 130, 220, 0.45); }
  :global(.pdf-text-whitespace) { position: absolute; width: 0; height: 0; overflow: hidden; color: transparent; white-space: pre; pointer-events: none; }

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
