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
