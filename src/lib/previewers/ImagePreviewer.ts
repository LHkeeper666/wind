import type { Previewer } from './types';
import { logError } from '../utils/log';
import { ImageViewport, type ImageViewportSnapshot } from './ImageViewport';

const SUPPORTED_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico']);

export class ImagePreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private urls = new Set<string>();
  private pendingUrls = new Set<string>();
  private viewports = new Map<HTMLElement, ImageViewport>();
  private savedViewports = new WeakMap<HTMLElement, { path: string; state: ImageViewportSnapshot }>();

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return SUPPORTED_EXTENSIONS.has(ext);
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;

    if (typeof content === 'string') {
      const url = URL.createObjectURL(new Blob([content], { type: 'image/svg+xml' }));
      this.urls.add(url);
      await this.renderImage(url, container);
    } else {
      // Binary image — preload via new Image() before writing to DOM
      const isSvg = /\.svg$/i.test(container.dataset.filePath || '');
      const blob = new Blob([content], isSvg ? { type: 'image/svg+xml' } : undefined);
      const url = URL.createObjectURL(blob);
      this.urls.add(url);
      await this.renderImage(url, container);
    }
  }

  private async renderImage(url: string, container: HTMLElement): Promise<void> {
      this.pendingUrls.add(url);
      await new Promise<void>((resolve) => {
        const img = new Image();
        img.onload = () => {
          this.pendingUrls.delete(url);
          container.innerHTML = '';
          const wrapper = document.createElement('div');
          wrapper.className = 'preview-image';
          wrapper.appendChild(img);
          container.appendChild(wrapper);

          // Show thumbnail info bar if metadata exists
          const meta = this.readMeta(container);
          if (meta) {
            const bar = this.createInfoBar(meta, container);
            container.appendChild(bar);
          }

          resolve();
        };
        img.onerror = () => {
          this.pendingUrls.delete(url);
          container.innerHTML = '<p class="preview-unsupported">Failed to load image</p>';
          URL.revokeObjectURL(url);
          this.urls.delete(url);
          resolve();
        };
        img.src = url;
      });
  }

  afterSwap(container: HTMLElement): void {
    this.container = container;
    container.classList.add('image-preview-slot');
    const wrapper = container.querySelector<HTMLElement>('.preview-image');
    const img = wrapper?.querySelector('img');
    const previous = this.viewports.get(container);
    if (wrapper && img && previous?.owns(wrapper, img)) {
      queueMicrotask(() => this.cleanupUnused());
      return;
    }
    if (previous) {
      const state = previous.snapshot();
      if (state) this.savedViewports.set(container, { path: container.dataset.filePath || '', state });
      previous.dispose();
    }
    this.viewports.delete(container);
    if (wrapper && img) {
      const viewport = new ImageViewport(wrapper, img, container, () => this.cleanupUnused());
      this.viewports.set(container, viewport);
      const saved = this.savedViewports.get(container);
      if (saved && saved.path === container.dataset.filePath) viewport.restore(saved.state);
      this.savedViewports.delete(container);
      const button = container.querySelector<HTMLButtonElement>('.image-view-original');
      if (button) button.onclick = () => void viewport.loadOriginal();
    }
    queueMicrotask(() => this.cleanupUnused());
  }

  handleKeydown(container: HTMLElement, event: KeyboardEvent): boolean {
    return this.viewports.get(container)?.handleKeydown(event) ?? false;
  }

  handleWheel(container: HTMLElement, event: WheelEvent): boolean {
    const viewport = this.viewports.get(container);
    if (!viewport || !event.ctrlKey) return false;
    viewport.wheel(event);
    return true;
  }

  release(container: HTMLElement): void {
    const viewport = this.viewports.get(container);
    const state = viewport?.snapshot();
    if (state) this.savedViewports.set(container, { path: container.dataset.filePath || '', state });
    viewport?.dispose();
    this.viewports.delete(container);
    container.classList.remove('image-preview-slot');
    queueMicrotask(() => this.cleanupUnused());
  }

  cleanupUnused(): void {
    const active = new Set(Array.from(document.querySelectorAll('img'), img => img.src));
    for (const url of this.urls) {
      if (!active.has(url) && !this.pendingUrls.has(url)) {
        URL.revokeObjectURL(url);
        this.urls.delete(url);
      }
    }
  }

  private readMeta(container: HTMLElement): { width: number; height: number; originalSize: number; isThumbnail: boolean } | null {
    const ds = container.dataset;
    if (!ds.thumbWidth || !ds.thumbHeight) return null;
    return {
      width: parseInt(ds.thumbWidth, 10),
      height: parseInt(ds.thumbHeight, 10),
      originalSize: parseInt(ds.thumbOriginalSize || '0', 10),
      isThumbnail: ds.thumbIsThumbnail === 'true',
    };
  }

  private formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  private createInfoBar(
    meta: { width: number; height: number; originalSize: number; isThumbnail: boolean },
    container: HTMLElement,
  ): HTMLElement {
    const bar = document.createElement('div');
    bar.className = 'image-info-bar';

    const info = document.createElement('span');
    info.textContent = meta.isThumbnail
      ? `原图 ${meta.width}×${meta.height} · 已缩放预览`
      : `${meta.width}×${meta.height}`;
    if (meta.originalSize > 0) {
      info.textContent += ` · ${this.formatSize(meta.originalSize)}`;
    }
    bar.appendChild(info);

    if (meta.isThumbnail) {
      const btn = document.createElement('button');
      btn.textContent = '查看原图';
      btn.className = 'image-view-original';
      bar.appendChild(btn);
    }

    return bar;
  }

  dispose(): void {
    for (const viewport of this.viewports.values()) viewport.dispose();
    this.viewports.clear();
    this.cleanupUnused();
    this.container = null;
  }
}
