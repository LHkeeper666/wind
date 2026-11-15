import { invoke } from '@tauri-apps/api/core';
import type { Previewer } from './types';
import { parseRst } from './rst-parser';
import { highlightRstCode } from './rst-highlighter';

export class RstPreviewer implements Previewer {
  onHeadings?: Previewer['onHeadings'];
  private urls = new Set<string>();

  match(filePath: string): boolean { return /\.rst$/i.test(filePath); }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
    const { html, headings } = parseRst(text);
    container.innerHTML = `<div class="preview-markdown preview-rst">${html}</div>`;
    this.onHeadings?.(headings);
    const codeBlocks = container.querySelectorAll<HTMLElement>('pre > code[class*="language-"]');
    await Promise.all(Array.from(codeBlocks, async block => {
      const pre = block.parentElement;
      if (!pre) return;
      const language = [...block.classList].find(name => name.startsWith('language-'))?.slice(9) ?? '';
      try {
        const highlighted = await highlightRstCode(block.textContent ?? '', language);
        const wrapper = document.createElement('div');
        wrapper.innerHTML = highlighted;
        const result = wrapper.firstElementChild;
        if (result) pre.replaceWith(result);
      } catch { /* Keep the escaped code block when highlighting fails. */ }
    }));
    const filePath = container.dataset.filePath ?? '';
    const images = container.querySelectorAll<HTMLImageElement>('img[data-rst-src]');
    await Promise.all(Array.from(images, async img => {
      const src = img.dataset.rstSrc ?? '';
      if (!src || /^(https?:|mailto:|data:|blob:|\/\/)/i.test(src)) {
        img.replaceWith(document.createTextNode(`Image: ${src}`)); return;
      }
      const path = /^[a-z]:[\\/]/i.test(src) || src.startsWith('\\')
        ? src : `${filePath.replace(/[\\/][^\\/]*$/, '')}\\${src}`;
      try {
        const base64 = await invoke<string>('read_binary_file', { path: path.replace(/\//g, '\\') });
        const bytes = Uint8Array.from(atob(base64), char => char.charCodeAt(0));
        const url = URL.createObjectURL(new Blob([bytes]));
        this.urls.add(url);
        img.src = url;
        img.onerror = () => { img.replaceWith(document.createTextNode(`Failed to load image: ${src}`)); URL.revokeObjectURL(url); this.urls.delete(url); };
      } catch {
        img.replaceWith(document.createTextNode(`Failed to load image: ${src}`));
      }
    }));
  }

  afterSwap(): void { this.cleanupUnused(); }

  cleanupUnused(): void {
    const active = new Set(Array.from(document.querySelectorAll('img'), image => image.src));
    for (const url of this.urls) if (!active.has(url)) { URL.revokeObjectURL(url); this.urls.delete(url); }
  }

  dispose(): void { this.cleanupUnused(); }
}
