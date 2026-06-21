import type { Previewer } from './types';

export class PdfPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'pdf';
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;

    if (content instanceof ArrayBuffer && content.byteLength > 0) {
      // Rendered page PNG — display like an image
      const blob = new Blob([content], { type: 'image/png' });
      const url = URL.createObjectURL(blob);

      await new Promise<void>((resolve) => {
        const img = new Image();
        img.onload = () => {
          container.innerHTML = '';
          const wrapper = document.createElement('div');
          wrapper.className = 'preview-pdf-page';
          img.style.maxWidth = '100%';
          img.style.maxHeight = '100%';
          img.style.objectFit = 'contain';
          wrapper.appendChild(img);
          container.appendChild(wrapper);
          resolve();
        };
        img.onerror = () => {
          container.innerHTML = '<p class="preview-unsupported">Failed to render PDF page</p>';
          URL.revokeObjectURL(url);
          resolve();
        };
        img.src = url;
      });
    } else {
      // Loading placeholder
      container.innerHTML = '<p class="preview-unsupported">Loading PDF...</p>';
    }
  }

  dispose(): void {
    if (this.container) {
      const img = this.container.querySelector('img');
      if (img?.src.startsWith('blob:')) {
        URL.revokeObjectURL(img.src);
      }
      this.container.innerHTML = '';
      this.container = null;
    }
  }
}
