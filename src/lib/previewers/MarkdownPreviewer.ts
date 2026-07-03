import type { Previewer, TocHeading } from './types';
import MarkdownIt from 'markdown-it';
import { codeToHtml } from 'shiki';

export class MarkdownPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private md: MarkdownIt;
  onHeadings?: (headings: TocHeading[]) => void;

  constructor() {
    this.md = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
    });

    // Add data-line attribute to headings for scroll sync
    const defaultHeadingOpen = this.md.renderer.rules.heading_open || ((tokens: any, idx: number, options: any, env: any, self: any) => {
      return self.renderToken(tokens, idx, options);
    });
    this.md.renderer.rules.heading_open = (tokens: any, idx: number, options: any, env: any, self: any) => {
      const token = tokens[idx];
      if (token.map) {
        token.attrSet('data-line', String(token.map[0]));
      }
      return defaultHeadingOpen(tokens, idx, options, env, self);
    };
  }

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'md' || ext === 'markdown';
  }

  parseHeadings(text: string): TocHeading[] {
    const tokens = this.md.parse(text, {});
    const headings: TocHeading[] = [];
    const stack: TocHeading[] = [];

    for (let i = 0; i < tokens.length; i++) {
      if (tokens[i].type === 'heading_open') {
        const level = parseInt(tokens[i].tag.replace('h', ''));
        const contentToken = tokens[i + 1];
        const text = contentToken?.children
          ?.map((c: any) => c.content || '')
          .join('') || '';
        const line = tokens[i].map ? tokens[i].map[0] : 0;
        const heading: TocHeading = { level, text, line, children: [], expanded: true };

        // Find parent: pop stack until we find a heading with lower level
        while (stack.length > 0 && stack[stack.length - 1].level >= level) {
          stack.pop();
        }

        if (stack.length === 0) {
          headings.push(heading);
        } else {
          stack[stack.length - 1].children.push(heading);
        }
        stack.push(heading);
      }
    }

    return headings;
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    try {
      // Parse headings and emit
      if (this.onHeadings) {
        this.onHeadings(this.parseHeadings(text));
      }

      // Render markdown synchronously (no async highlight dependency)
      const html = this.md.render(text);
      container.innerHTML = `<div class="preview-markdown">${html}</div>`;

      // Post-process: highlight code blocks with Shiki
      const codeBlocks = container.querySelectorAll('pre > code');
      for (const block of codeBlocks) {
        const pre = block.parentElement!;
        const lang = [...block.classList]
          .find(c => c.startsWith('language-'))
          ?.replace('language-', '') || '';
        const code = block.textContent || '';

        try {
          const highlighted = await codeToHtml(code, {
            lang: lang || 'text',
            theme: 'github-dark',
          });
          pre.replaceWith(
            Object.assign(document.createElement('div'), { innerHTML: highlighted }).firstChild!
          );
        } catch {
          // Shiki failed for this block — keep the original <pre><code>
        }
      }
    } catch (err) {
      console.error('[MarkdownPreviewer] render failed:', err);
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
