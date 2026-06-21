import type { Previewer } from './types';

const SUPPORTED_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico']);

export class ImagePreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return SUPPORTED_EXTENSIONS.has(ext);
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;

    if (typeof content === 'string') {
      // SVG content
      container.innerHTML = `<div class="preview-image">${content}</div>`;
    } else {
      // Binary image — preload via new Image() before writing to DOM
      const blob = new Blob([content]);
      const url = URL.createObjectURL(blob);
      await new Promise<void>((resolve) => {
        const img = new Image();
        img.onload = () => {
          container.innerHTML = '';
          const wrapper = document.createElement('div');
          wrapper.className = 'preview-image';
          img.style.maxWidth = '100%';
          img.style.maxHeight = '100%';
          img.style.objectFit = 'contain';
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
          container.innerHTML = '<p class="preview-unsupported">Failed to load image</p>';
          URL.revokeObjectURL(url);
          resolve();
        };
        img.src = url;
      });
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
      btn.onclick = () => this.loadOriginal(container);
      bar.appendChild(btn);
    }

    return bar;
  }

  private async loadOriginal(container: HTMLElement): Promise<void> {
    const filePath = container.dataset.filePath;
    if (!filePath) return;

    // Show loading state
    const bar = container.querySelector('.image-info-bar');
    if (bar) {
      const btn = bar.querySelector('.image-view-original') as HTMLButtonElement | null;
      if (btn) {
        btn.textContent = '加载中...';
        btn.disabled = true;
      }
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const base64 = await invoke<string>('read_binary_file', { path: filePath });
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes.buffer]);
      const url = URL.createObjectURL(blob);

      // Replace image
      const img = container.querySelector('img');
      if (img) {
        if (img.src.startsWith('blob:')) {
          URL.revokeObjectURL(img.src);
        }
        img.src = url;
      }

      // Update info bar — read original dimensions from dataset
      const origW = container.dataset.thumbWidth || '';
      const origH = container.dataset.thumbHeight || '';
      if (bar) {
        bar.innerHTML = '';
        const info = document.createElement('span');
        info.textContent = origW && origH ? `${origW}×${origH}` : '原图已加载';
        bar.appendChild(info);
      }
    } catch (error) {
      console.error('Failed to load original image:', error);
      if (bar) {
        const btn = bar.querySelector('.image-view-original') as HTMLButtonElement | null;
        if (btn) {
          btn.textContent = '加载失败';
          btn.disabled = false;
        }
      }
    }
  }

  dispose(): void {
    if (this.container) {
      const img = this.container.querySelector('img');
      if (img) {
        if (img.src.startsWith('blob:')) {
          URL.revokeObjectURL(img.src);
        }
        img.src = '';
      }
      this.container.innerHTML = '';
      this.container = null;
    }
  }
}
