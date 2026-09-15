import { invoke } from '@tauri-apps/api/core';

export interface PdfPageData {
  data: string; // base64-encoded image
  width: number;
  height: number;
  format: string; // "jpeg" or "png"
}

export interface TextMatch {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PdfSearchResult {
  page: number;
  matches: TextMatch[];
}

export interface PdfSearchState {
  results: PdfSearchResult[];
  currentMatchPage: number;
  currentMatchIndex: number;
}

export const RENDER_SCALE = 2.0;
export const PRELOAD_RANGE = 3;
const PDF_PAGE_SEPARATOR = 1;

// --- Page Dimensions (from get_pdf_info) ---

export interface PdfPageDimensions {
  width: number;  // PDF points (1/72 inch)
  height: number;
}

// --- Outline / Bookmarks ---

export interface PdfOutlineItem {
  title: string;
  page: number;    // 0-based page index
  x: number;       // target x in PDF points
  y: number;       // target y in PDF points
  children: PdfOutlineItem[];
}

// --- Link Annotations ---

export interface PdfLinkAnnotation {
  rect: [number, number, number, number]; // [left, bottom, right, top] in page points
  target_page: number;
  target_x: number;
  target_y: number;
  is_external: boolean;
  url: string | null;
}

// --- Page Cache ---

export class PdfPageCache {
  private cache = new Map<number, { data: PdfPageData; bytes: number; lastUsed: number }>();
  private preloading = new Set<number>();
  private pinned = new Set<number>();
  private bytes = 0;

  constructor(private readonly maxBytes = 80 * 1024 * 1024) {}

  private estimateBytes(data: PdfPageData): number {
    return Math.max(1, Math.ceil(data.data.length * 0.75));
  }

  get(page: number): PdfPageData | undefined {
    const entry = this.cache.get(page);
    if (!entry) return undefined;
    entry.lastUsed = performance.now();
    return entry.data;
  }

  has(page: number): boolean {
    return this.cache.has(page);
  }

  set(page: number, data: PdfPageData): void {
    const bytes = this.estimateBytes(data);
    const previous = this.cache.get(page);
    if (!previous && bytes > this.maxBytes) return;
    if (previous) this.bytes -= previous.bytes;
    this.cache.set(page, { data, bytes, lastUsed: performance.now() });
    this.bytes += bytes;
    this.evict();
  }

  delete(page: number): void {
    const entry = this.cache.get(page);
    if (entry) this.bytes -= entry.bytes;
    this.cache.delete(page);
  }

  setPinnedPages(pages: Iterable<number>): void {
    this.pinned = new Set(pages);
    this.evict();
  }

  getBytes(): number { return this.bytes; }
  getSize(): number { return this.cache.size; }

  private evict(): void {
    while (this.bytes > this.maxBytes) {
      let candidate: number | undefined;
      let oldest = Number.POSITIVE_INFINITY;
      for (const [page, entry] of this.cache) {
        if (!this.pinned.has(page) && entry.lastUsed < oldest) {
          candidate = page;
          oldest = entry.lastUsed;
        }
      }
      if (candidate === undefined) {
        for (const [page, entry] of this.cache) {
          if (entry.lastUsed < oldest) {
            candidate = page;
            oldest = entry.lastUsed;
          }
        }
      }
      if (candidate === undefined) break;
      this.delete(candidate);
    }
  }

  isPreloading(page: number): boolean {
    return this.preloading.has(page);
  }

  markPreloading(page: number): void {
    this.preloading.add(page);
  }

  unmarkPreloading(page: number): void {
    this.preloading.delete(page);
  }

  clear(): void {
    this.cache.clear();
    this.preloading.clear();
    this.pinned.clear();
    this.bytes = 0;
  }
}

export type PdfRenderPriority = 0 | 1 | 2 | 3;

interface PdfRenderJob<T> {
  priority: PdfRenderPriority;
  sequence: number;
  generation: number;
  task: () => Promise<T>;
  resolve: (value: T) => void;
  reject: (reason: unknown) => void;
}

export class PdfRenderScheduler {
  private queue: PdfRenderJob<unknown>[] = [];
  private running = false;
  private sequence = 0;

  enqueue<T>(task: () => Promise<T>, priority: PdfRenderPriority, generation: number): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      this.queue.push({ priority, sequence: this.sequence++, generation, task, resolve: value => resolve(value as T), reject });
      this.pump();
    });
  }

  invalidate(generation: number): void {
    const stale = this.queue.filter(job => job.generation < generation);
    this.queue = this.queue.filter(job => job.generation >= generation);
    for (const job of stale) job.reject(new Error('stale PDF render request'));
  }

  clear(): void {
    const pending = this.queue;
    this.queue = [];
    for (const job of pending) job.reject(new Error('PDF render queue cleared'));
  }

  private pump(): void {
    if (this.running || this.queue.length === 0) return;
    this.queue.sort((a, b) => a.priority - b.priority || a.sequence - b.sequence);
    const job = this.queue.shift()!;
    this.running = true;
    job.task().then(job.resolve, job.reject).finally(() => {
      this.running = false;
      this.pump();
    });
  }
}

// --- Fetch & Render ---

export async function fetchPdfPage(
  path: string,
  pageNum: number,
  cache: PdfPageCache,
  scale?: number,
  shouldCommit: () => boolean = () => true,
): Promise<PdfPageData> {
  const cached = cache.get(pageNum);
  if (cached) return cached;

  const actualScale = scale ?? RENDER_SCALE;
  const t0 = performance.now();
  const result = await invoke<PdfPageData>('render_pdf_page', {
    path,
    page: pageNum,
    scale: actualScale,
  });
  const t1 = performance.now();
  console.log(`[pdf-perf] fetchPdfPage p${pageNum} scale=${actualScale} invoke=${(t1 - t0).toFixed(1)}ms data=${result.width}x${result.height} base64=${(result.data.length / 1024).toFixed(0)}KB`);
  if (shouldCommit()) cache.set(pageNum, result);
  return result;
}

export async function preloadPdfPages(
  path: string,
  currentPage: number,
  totalPages: number,
  cache: PdfPageCache,
  rangeStart?: number,
  rangeEnd?: number,
  scale?: number,
): Promise<void> {
  const start = rangeStart ?? Math.max(0, currentPage - PRELOAD_RANGE);
  const end = rangeEnd ?? Math.min(totalPages - 1, currentPage + PRELOAD_RANGE);
  // Fire all preloads concurrently (don't await each one)
  const promises: Promise<void>[] = [];
  for (let i = start; i <= end; i++) {
    if (cache.has(i) || cache.isPreloading(i)) continue;
    cache.markPreloading(i);
    promises.push(
      fetchPdfPage(path, i, cache, scale)
        .then(() => {})
        .catch(() => {})
        .finally(() => cache.unmarkPreloading(i))
    );
  }
  // Don't await — let preloads happen in background
  if (promises.length > 0) {
    Promise.all(promises).catch(() => {});
  }
}

// --- Canvas Drawing ---

export function drawPdfPageToCanvas(
  canvas: HTMLCanvasElement,
  data: PdfPageData,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const binary = atob(data.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const mimeType = data.format === 'jpeg' ? 'image/jpeg' : 'image/png';
    const blob = new Blob([bytes.buffer], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const img = new Image();
    img.onload = () => {
      canvas.width = data.width;
      canvas.height = data.height;
      const ctx = canvas.getContext('2d');
      if (!ctx) { URL.revokeObjectURL(url); reject(new Error('no ctx')); return; }
      ctx.clearRect(0, 0, data.width, data.height);
      ctx.drawImage(img, 0, 0);
      URL.revokeObjectURL(url);
      resolve();
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load page image'));
    };
    img.src = url;
  });
}

export function drawSearchHighlights(
  ctx: CanvasRenderingContext2D,
  page: number,
  pageHeight: number,
  searchState: PdfSearchState,
): void {
  const pageMatches = searchState.results.find(r => r.page === page);
  if (!pageMatches || pageMatches.matches.length === 0) return;

  for (let i = 0; i < pageMatches.matches.length; i++) {
    const match = pageMatches.matches[i];
    const isActive = page === searchState.currentMatchPage && i === searchState.currentMatchIndex;

    ctx.fillStyle = isActive ? 'rgba(255, 165, 0, 0.5)' : 'rgba(255, 255, 0, 0.35)';

    const x = match.x * RENDER_SCALE;
    const y = pageHeight - (match.y + match.height) * RENDER_SCALE;
    const w = match.width * RENDER_SCALE;
    const h = match.height * RENDER_SCALE;

    ctx.fillRect(x, y, w, h);
  }
}

export function drawPageWithHighlights(
  canvas: HTMLCanvasElement,
  data: PdfPageData,
  searchState: PdfSearchState,
  currentPage: number,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const binary = atob(data.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const mimeType = data.format === 'jpeg' ? 'image/jpeg' : 'image/png';
    const blob = new Blob([bytes.buffer], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const img = new Image();
    img.onload = () => {
      canvas.width = data.width;
      canvas.height = data.height;
      const ctx = canvas.getContext('2d');
      if (!ctx) { URL.revokeObjectURL(url); reject(new Error('no ctx')); return; }
      ctx.clearRect(0, 0, data.width, data.height);
      ctx.drawImage(img, 0, 0);
      URL.revokeObjectURL(url);

      drawSearchHighlights(ctx, currentPage, data.height, searchState);
      resolve();
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load page image'));
    };
    img.src = url;
  });
}

// --- Search ---

export async function searchPdfText(
  path: string,
  query: string,
): Promise<PdfSearchResult[]> {
  if (!query.trim()) return [];
  return invoke<PdfSearchResult[]>('search_pdf_text', {
    path,
    query: query.trim(),
  });
}

export function navigateSearchMatch(
  direction: 'next' | 'prev',
  searchState: PdfSearchState,
): { page: number; index: number } | null {
  if (searchState.results.length === 0) return null;

  const allMatches: { page: number; index: number }[] = [];
  for (const result of searchState.results) {
    for (let i = 0; i < result.matches.length; i++) {
      allMatches.push({ page: result.page, index: i });
    }
  }

  let currentPos = allMatches.findIndex(
    m => m.page === searchState.currentMatchPage && m.index === searchState.currentMatchIndex,
  );

  if (currentPos === -1) {
    currentPos = 0;
  } else if (direction === 'next') {
    currentPos = (currentPos + 1) % allMatches.length;
  } else {
    currentPos = (currentPos - 1 + allMatches.length) % allMatches.length;
  }

  return allMatches[currentPos];
}

export function getTotalMatchCount(searchState: PdfSearchState): number {
  return searchState.results.reduce((sum, r) => sum + r.matches.length, 0);
}

// --- Outline ---

export async function fetchPdfOutline(path: string): Promise<PdfOutlineItem[]> {
  return invoke<PdfOutlineItem[]>('get_pdf_outline', { path });
}

// --- Link Annotations ---

const linkCache = new Map<string, PdfLinkAnnotation[]>();

export async function fetchPdfPageLinks(path: string, page: number): Promise<PdfLinkAnnotation[]> {
  const key = `${path}:${page}`;
  const cached = linkCache.get(key);
  if (cached) return cached;
  const result = await invoke<PdfLinkAnnotation[]>('get_pdf_page_links', { path, page });
  linkCache.set(key, result);
  return result;
}

export function clearLinkCache(): void {
  linkCache.clear();
}

// --- Continuous Scroll Helpers ---

/**
 * Calculate cumulative height offsets for each page slot.
 * Returns an array where result[i] is the top offset of page i.
 */
export function cumulativePageOffsets(pageDimensions: PdfPageDimensions[], scale: number): number[] {
  const offsets: number[] = [0];
  for (let i = 0; i < pageDimensions.length; i++) {
    offsets.push(offsets[i] + pageDimensions[i].height * scale + PDF_PAGE_SEPARATOR);
  }
  return offsets;
}

/**
 * Given a scrollTop position, find which page the center of the viewport falls on.
 */
export function pageAtScrollPosition(scrollTop: number, viewportHeight: number, offsets: number[]): number {
  const center = scrollTop + viewportHeight / 2;
  for (let i = 1; i < offsets.length; i++) {
    if (offsets[i] > center) return i - 1;
  }
  return offsets.length - 2; // last page
}

/**
 * Scroll to a specific page with optional y-offset (in PDF points).
 * pdfY is from the bottom of the page (PDF coordinate system).
 */
export function scrollToPosition(
  page: number,
  pdfY: number,
  pageDimensions: PdfPageDimensions[],
  scale: number,
): number {
  const offsets = cumulativePageOffsets(pageDimensions, scale);
  const pageTop = offsets[page] || 0;
  // Convert PDF y (from bottom) to CSS offset (from top)
  const pageHeight = pageDimensions[page]?.height || 0;
  const hasExplicitY = Number.isFinite(pdfY) && pdfY > 0 && pdfY < pageHeight;
  const yOffset = hasExplicitY ? (pageHeight - pdfY) * scale : 0;
  return pageTop + yOffset;
}
