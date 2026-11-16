import { invoke } from '@tauri-apps/api/core';
import { advanceZoomScale, wheelZoomDelta } from '../utils/pdf-zoom';
import { logError } from '../utils/log';

const PAN_STEP = 40;

export interface ImageViewportSnapshot {
  zoomRatio: number;
  centerX: number;
  centerY: number;
  manualZoom: boolean;
}

export class ImageViewport {
  private scale = 1;
  private targetScale = 1;
  private panX = 0;
  private panY = 0;
  private anchorX = 0;
  private anchorY = 0;
  private imageX = 0;
  private imageY = 0;
  private raf = 0;
  private lastFrame = 0;
  private resized: ResizeObserver;
  private originalPending = false;
  private originalLoaded = false;
  private automaticAttempted = false;
  private disposed = false;
  private manualZoom = false;
  private originalUrl: string | null = null;
  private originalTimer: ReturnType<typeof setTimeout> | undefined;
  private readonly width: number;
  private readonly height: number;
  private pendingRestore: ImageViewportSnapshot | null = null;

  constructor(private viewport: HTMLElement, private img: HTMLImageElement, private slot: HTMLElement,
      private onOriginalLoaded: () => void) {
    this.width = img.naturalWidth;
    this.height = img.naturalHeight;
    this.resized = new ResizeObserver(() => {
      if (this.pendingRestore || !this.manualZoom) this.fit();
      else this.paint();
    });
    this.resized.observe(viewport);
    this.fit();
  }

  owns(viewport: HTMLElement, img: HTMLImageElement): boolean {
    return this.viewport === viewport && this.img === img && !this.disposed;
  }

  snapshot(): ImageViewportSnapshot | null {
    const vw = this.viewport.clientWidth;
    const vh = this.viewport.clientHeight;
    if (vw <= 0 || vh <= 0 || !this.width || !this.height || this.scale <= 0) return this.pendingRestore;
    const fitScale = Math.min(vw / this.width, vh / this.height);
    const left = (vw - this.width * this.scale) / 2 + this.panX;
    const top = (vh - this.height * this.scale) / 2 + this.panY;
    return {
      zoomRatio: this.scale / fitScale,
      centerX: (vw / 2 - left) / (this.width * this.scale),
      centerY: (vh / 2 - top) / (this.height * this.scale),
      manualZoom: this.manualZoom,
    };
  }

  restore(snapshot: ImageViewportSnapshot) {
    this.pendingRestore = snapshot;
    this.fit();
  }

  private fit() {
    if (!this.width || !this.height) return;
    const vw = this.viewport.clientWidth;
    const vh = this.viewport.clientHeight;
    if (vw <= 0 || vh <= 0) return;
    const fitScale = Math.min(vw / this.width, vh / this.height);
    const restore = this.pendingRestore;
    this.pendingRestore = null;
    this.scale = restore ? fitScale * restore.zoomRatio : fitScale;
    this.targetScale = this.scale;
    this.panX = restore ? (0.5 - restore.centerX) * this.width * this.scale : 0;
    this.panY = restore ? (0.5 - restore.centerY) * this.height * this.scale : 0;
    this.manualZoom = restore?.manualZoom ?? false;
    this.paint();
  }

  private paint() {
    const vw = this.viewport.clientWidth;
    const vh = this.viewport.clientHeight;
    if (vw <= 0 || vh <= 0 || !Number.isFinite(this.scale) || this.scale <= 0) return;
    const w = this.width * this.scale;
    const h = this.height * this.scale;
    this.panX = Math.max(-Math.max(0, w - vw) / 2, Math.min(Math.max(0, w - vw) / 2, this.panX));
    this.panY = Math.max(-Math.max(0, h - vh) / 2, Math.min(Math.max(0, h - vh) / 2, this.panY));
    this.img.style.width = `${w}px`;
    this.img.style.height = `${h}px`;
    this.img.style.left = `${(vw - w) / 2 + this.panX}px`;
    this.img.style.top = `${(vh - h) / 2 + this.panY}px`;
  }

  pan(dx: number, dy: number) {
    this.panX += dx;
    this.panY += dy;
    this.paint();
  }

  zoom(delta: number, clientX?: number, clientY?: number) {
    if (!this.width || !this.height) return;
    const bounds = this.viewport.getBoundingClientRect();
    if (bounds.width <= 0 || bounds.height <= 0 || !Number.isFinite(this.scale) || this.scale <= 0) return;
    this.anchorX = clientX === undefined ? bounds.width / 2 : Math.max(0, Math.min(bounds.width, clientX - bounds.left));
    this.anchorY = clientY === undefined ? bounds.height / 2 : Math.max(0, Math.min(bounds.height, clientY - bounds.top));
    this.imageX = (this.anchorX - (bounds.width - this.width * this.scale) / 2 - this.panX) / this.scale;
    this.imageY = (this.anchorY - (bounds.height - this.height * this.scale) / 2 - this.panY) / this.scale;
    this.manualZoom = true;
    const min = Math.min(bounds.width / this.width, bounds.height / this.height) * 0.1;
    this.targetScale = Math.max(min, Math.min(Math.max(16, min * 160), this.targetScale * Math.exp(delta)));
    if (this.originalTimer) { clearTimeout(this.originalTimer); this.originalTimer = undefined; }
    if (delta > 0 && this.slot.dataset.thumbIsThumbnail === 'true' && !this.originalLoaded && !this.originalPending
        && !this.automaticAttempted && this.targetScale * devicePixelRatio > 1) {
      this.originalTimer = setTimeout(() => {
        this.originalTimer = undefined;
        if (this.disposed || this.originalLoaded || this.originalPending) return;
        this.automaticAttempted = true;
        void this.loadOriginal();
      }, 180);
    }
    if (!this.raf) this.raf = requestAnimationFrame(this.animate);
  }

  private animate = (now: number) => {
    this.raf = 0;
    const elapsed = this.lastFrame ? now - this.lastFrame : 16;
    this.lastFrame = now;
    this.scale = matchMedia('(prefers-reduced-motion: reduce)').matches
      ? this.targetScale : advanceZoomScale(this.scale, this.targetScale, elapsed);
    this.panX = this.anchorX - (this.viewport.clientWidth - this.width * this.scale) / 2 - this.imageX * this.scale;
    this.panY = this.anchorY - (this.viewport.clientHeight - this.height * this.scale) / 2 - this.imageY * this.scale;
    this.paint();
    if (this.scale !== this.targetScale) this.raf = requestAnimationFrame(this.animate);
    else this.lastFrame = 0;
  };

  wheel(event: WheelEvent) {
    this.zoom(wheelZoomDelta(event.deltaY, event.deltaMode), event.clientX, event.clientY);
  }

  async loadOriginal() {
    if (this.originalPending || this.originalLoaded || this.disposed) return;
    const path = this.slot.dataset.filePath;
    if (!path) return;
    this.originalPending = true;
    const button = this.slot.querySelector<HTMLButtonElement>('.image-view-original');
    if (button) { button.disabled = true; button.textContent = '加载中...'; }
    let url: string | null = null;
    try {
      const base64 = await invoke<string>('read_binary_file', { path });
      if (this.disposed || this.slot.dataset.filePath !== path) return;
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      url = URL.createObjectURL(new Blob([bytes]));
      const decoded = new Image();
      decoded.src = url;
      await decoded.decode();
      if (this.disposed || this.slot.dataset.filePath !== path) return;
      decoded.alt = this.img.alt;
      decoded.draggable = false;
      decoded.style.width = this.img.style.width;
      decoded.style.height = this.img.style.height;
      decoded.style.left = this.img.style.left;
      decoded.style.top = this.img.style.top;
      this.img.replaceWith(decoded);
      this.img = decoded;
      this.originalUrl = url;
      this.originalLoaded = true;
      if (button) { button.disabled = false; button.textContent = '原图已加载'; }
      this.slot.dataset.thumbIsThumbnail = 'false';
      queueMicrotask(this.onOriginalLoaded);
      url = null;
    } catch (error) {
      logError('ImageViewport', `Failed to load original image: ${error}`);
      if (button && !this.disposed) { button.disabled = false; button.textContent = '重试原图'; }
    } finally {
      if (url) URL.revokeObjectURL(url);
      this.originalPending = false;
    }
  }

  dispose() {
    this.disposed = true;
    if (this.originalTimer) clearTimeout(this.originalTimer);
    this.resized.disconnect();
    if (this.raf) cancelAnimationFrame(this.raf);
    if (this.originalUrl) URL.revokeObjectURL(this.originalUrl);
  }

  handleKeydown(event: KeyboardEvent): boolean {
    if (event.ctrlKey && !event.altKey && !event.metaKey) {
      if (event.code === 'Equal' || event.code === 'NumpadAdd') { this.zoom(0.05); return true; }
      if (event.code === 'Minus' || event.code === 'NumpadSubtract') { this.zoom(-0.05); return true; }
      return false;
    }
    if (event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) return false;
    switch (event.code) {
      case 'KeyJ': this.pan(0, -PAN_STEP); return true;
      case 'KeyK': this.pan(0, PAN_STEP); return true;
      case 'KeyH': this.pan(PAN_STEP, 0); return true;
      case 'KeyL': this.pan(-PAN_STEP, 0); return true;
      default: return false;
    }
  }
}
