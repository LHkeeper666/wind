import type { Previewer } from './types';
import MarkdownIt from 'markdown-it';

export class MarkdownPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private md: MarkdownIt;

  constructor() {
    this.md = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
    });
  }

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'md' || ext === 'markdown';
  }

  render(content: string | ArrayBuffer, container: HTMLElement): void {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    try {
      const html = this.md.render(text);
      container.innerHTML = `<div class="preview-markdown">${html}</div>`;
    } catch {
      container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
    }
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  dispose(): void {
    if (this.container) {
      this.container.innerHTML = '';
      this.container = null;
    }
  }
}
