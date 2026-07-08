import type { Previewer, TocHeading } from './types';
// @ts-ignore - markdown-it has no bundled types
import MarkdownIt from 'markdown-it';
// @ts-ignore - markdown-it-texmath has no bundled types
import texmath from 'markdown-it-texmath';
import katex from 'katex';
import { createHighlighter, type Highlighter } from 'shiki';
import { invoke } from '@tauri-apps/api/core';
import 'katex/dist/katex.min.css';

export class MarkdownPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private md: any;
  private highlighter: Highlighter | null = null;
  private mermaidModule: any = null;
  onHeadings?: (headings: TocHeading[]) => void;

  private async getHighlighter(): Promise<Highlighter> {
    if (!this.highlighter) {
      this.highlighter = await createHighlighter({
        themes: ['github-dark'],
        langs: ['javascript', 'typescript', 'json', 'html', 'css', 'python', 'rust', 'bash', 'powershell', 'markdown'],
      });
    }
    return this.highlighter;
  }

  constructor() {
    this.md = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
    });

    // LaTeX math support via texmath with async KaTeX rendering.
    // Use a placeholder engine that outputs raw LaTeX text synchronously,
    // then replace with rendered KaTeX async after first paint.
    // texmath calls engine.renderToString(), so both names are required.
    const placeholderRender = (latex: string, options: { displayMode: boolean }) =>
      `<span class="math-placeholder" data-latex="${this.escapeAttr(latex)}" data-display="${options.displayMode}">${this.escapeHtml(latex)}</span>`;
    this.md.use(texmath, {
      engine: {
        render: placeholderRender,
        renderToString: placeholderRender,
      } as any,
      delimiters: ['dollars', 'brackets'],
      katexOptions: {
        throwOnError: false,
      },
    });

    // Obsidian wikilink image support: ![[file]], ![[file|w]], ![[file|wxh]]
    this.md.inline.ruler.after('image', 'obsidian_image', (state: any, silent: boolean) => {
      const src = state.src;
      const pos = state.pos;

      // Must start with ![[  (not just [[)
      if (src.charCodeAt(pos) !== 0x21 || src.charCodeAt(pos + 1) !== 0x5B || src.charCodeAt(pos + 2) !== 0x5B) {
        return false;
      }

      // Find closing ]]
      const closeIdx = src.indexOf(']]', pos + 3);
      if (closeIdx === -1) return false;

      if (silent) return true;

      const inner = src.slice(pos + 3, closeIdx);
      const parts = inner.split('|');
      const filename = parts[0].trim();
      const sizeSpec = parts[1]?.trim();

      const token = state.push('obsidian_image', 'img', 0);
      token.attrs = [['src', filename]];

      if (sizeSpec) {
        const dimMatch = sizeSpec.match(/^(\d+)(?:x(\d+))?$/);
        if (dimMatch) {
          token.attrs.push(['width', dimMatch[1]]);
          if (dimMatch[2]) {
            token.attrs.push(['height', dimMatch[2]]);
          }
        }
      }

      token.content = filename;
      state.pos = closeIdx + 2;
      return true;
    });

    // Render obsidian_image tokens as <img>
    this.md.renderer.rules.obsidian_image = (tokens: any, idx: number) => {
      const token = tokens[idx];
      const attrs = token.attrs?.map(([k, v]: [string, string]) => `${k}="${v}"`).join(' ') || '';
      return `<img ${attrs} alt="${token.content}">`;
    };

    // Add data-line attribute to block-level elements for editor cursor sync
    const blockTokens = ['heading_open', 'paragraph_open', 'bullet_list_open', 'ordered_list_open',
      'list_item_open', 'blockquote_open', 'table_open'];
    for (const type of blockTokens) {
      const defaultRule = this.md.renderer.rules[type] ||
        ((tokens: any, idx: number, options: any, env: any, self: any) => self.renderToken(tokens, idx, options));
      this.md.renderer.rules[type] = (tokens: any, idx: number, options: any, env: any, self: any) => {
        const token = tokens[idx];
        if (token.map) token.attrSet('data-line', String(token.map[0]));
        return defaultRule(tokens, idx, options, env, self);
      };
    }
    // Also handle self-closing block tokens (fence, code_block, hr)
    for (const type of ['fence', 'code_block', 'hr']) {
      const defaultRule = this.md.renderer.rules[type] ||
        ((tokens: any, idx: number, _opts: any, _env: any, self: any) => self.renderToken(tokens, idx, _opts));
      this.md.renderer.rules[type] = (tokens: any, idx: number, opts: any, env: any, self: any) => {
        const token = tokens[idx];
        const result = defaultRule(tokens, idx, opts, env, self);
        if (token.map) {
          return result.replace(/^<(\w+)/, `<$1 data-line="${token.map[0]}"`);
        }
        return result;
      };
    }
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
        const heading: TocHeading = { level, text, line, children: [], expanded: false };

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

    // Auto-expand: if only one top-level heading, expand it one level
    if (headings.length === 1) {
      headings[0].expanded = true;
    }

    return headings;
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
    const filePath = container.dataset.filePath || '';

    try {
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      const tMd = performance.now();

      // Parse headings and emit
      if (this.onHeadings) {
        this.onHeadings(this.parseHeadings(text));
      }

      // Render markdown synchronously (no async highlight dependency)
      const html = this.md.render(text);
      container.innerHTML = `<div class="preview-markdown">${html}</div>`;
      const mdMs = (performance.now() - tMd).toFixed(0);

      // Collect code blocks and images for parallel processing
      const codeBlocks = container.querySelectorAll('pre > code');
      const images = container.querySelectorAll('img');

      // Prepare code block highlighting tasks (skip mermaid, handle separately)
      const highlightTasks: Promise<void>[] = [];
      const mermaidTasks: Promise<void>[] = [];
      const highlighterPromise = this.getHighlighter();

      for (const block of codeBlocks) {
        const pre = block.parentElement!;
        const lang = [...block.classList]
          .find(c => c.startsWith('language-'))
          ?.replace('language-', '') || '';
        const code = block.textContent || '';

        if (lang === 'mermaid') {
          mermaidTasks.push(this.renderMermaidBlock(pre, code));
          continue;
        }

        highlightTasks.push(
          this.highlightCodeBlock(highlighterPromise, pre, code, lang)
        );
      }

      // Prepare image loading tasks
      const imageTasks: Promise<void>[] = [];
      for (const img of images) {
        const src = img.getAttribute('src');
        if (!src) continue;
        if (/^(https?:\/\/|data:|blob:)/.test(src)) continue;

        let resolvedPath = src;
        if (filePath && !src.startsWith('/') && !src.match(/^[A-Z]:\\/i)) {
          const dir = filePath.replace(/[\\/][^\\/]*$/, '');
          resolvedPath = dir + '\\' + src;
        }
        resolvedPath = resolvedPath.replace(/\//g, '\\');

        imageTasks.push(this.loadLocalImage(img, resolvedPath));
      }

      // Run all post-processing in parallel.
      // Math rendering is fire-and-forget so the outer await chain
      // returns quickly — the browser paints the raw LaTeX immediately,
      // then formulas appear progressively in the background.
      const mathCount = container.querySelectorAll('.math-placeholder').length;
      if (mathCount > 0) {
        this.renderMathAsync(container); // fire-and-forget, no await
      }
      const tPost = performance.now();
      await Promise.all([...highlightTasks, ...mermaidTasks, ...imageTasks]);
      const postMs = (performance.now() - tPost).toFixed(0);
      const totalMs = (performance.now() - tMd).toFixed(0);
      const stats = [`md:${mdMs}ms`];
      if (highlightTasks.length) stats.push(`code:${highlightTasks.length}`);
      if (mermaidTasks.length) stats.push(`mermaid:${mermaidTasks.length}`);
      if (imageTasks.length) stats.push(`img:${imageTasks.length}`);
      if (mathCount) stats.push(`math:${mathCount}(bg)`);
      console.log(`[md-render] ${fileName} total:${totalMs}ms md:${mdMs}ms post:${postMs}ms ${stats.join(' ')}`);
    } catch (err) {
      console.error('[MarkdownPreviewer] render failed:', err);
      container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
    }
  }

  private async highlightCodeBlock(
    highlighterPromise: Promise<Highlighter>,
    pre: Element,
    code: string,
    lang: string
  ): Promise<void> {
    try {
      const highlighter = await highlighterPromise;
      // Load language if not already loaded
      if (lang && !highlighter.getLoadedLanguages().includes(lang)) {
        try {
          await highlighter.loadLanguage(lang as any);
        } catch {
          // Language not available, fall back to text
          lang = 'text';
        }
      }
      const highlighted = highlighter.codeToHtml(code, {
        lang: lang || 'text',
        theme: 'github-dark',
      });
      const wrapper = document.createElement('div');
      wrapper.innerHTML = highlighted;
      const shikiPre = wrapper.firstChild as HTMLElement;
      if (shikiPre) {
        // Preserve data-line from the original pre for editor cursor sync
        const dataLine = pre.getAttribute('data-line');
        if (dataLine) shikiPre.setAttribute('data-line', dataLine);
        pre.replaceWith(shikiPre);
      }
    } catch {
      // Shiki failed for this block — keep the original <pre><code>
    }
  }

  private async renderMermaidBlock(pre: Element, code: string): Promise<void> {
    try {
      if (!this.mermaidModule) {
        this.mermaidModule = await import('mermaid');
      }
      const mermaid = this.mermaidModule.default;
      const isDark = document.documentElement.classList.contains('dark');
      mermaid.initialize({
        startOnLoad: false,
        theme: isDark ? 'dark' : 'default',
      });
      const id = 'mermaid-' + Math.random().toString(36).slice(2, 8);
      const { svg } = await mermaid.render(id, code);
      const wrapper = document.createElement('div');
      wrapper.className = 'mermaid-container';
      wrapper.innerHTML = svg;
      pre.replaceWith(wrapper);
    } catch {
      pre.classList.add('mermaid-error');
    }
  }

  private async loadLocalImage(img: HTMLImageElement, resolvedPath: string): Promise<void> {
    try {
      const base64 = await invoke<string>('read_binary_file', { path: resolvedPath });
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes.buffer]);
      img.src = URL.createObjectURL(blob);
    } catch (err) {
      console.warn('[MarkdownPreviewer] Failed to load image:', resolvedPath, err);
    }
  }

  private async renderMathAsync(container: HTMLElement): Promise<void> {
    const placeholders = Array.from(container.querySelectorAll<HTMLElement>('.math-placeholder'));
    if (placeholders.length === 0) return;

    // Yield so the browser paints raw LaTeX before we start replacing
    await new Promise<void>(resolve => setTimeout(resolve, 0));

    // Batch-render formulas: up to 30 per frame or 35ms time budget,
    // yielding via setTimeout between batches to keep the UI responsive.
    const BATCH_SIZE = 30;
    const TIME_BUDGET = 35;

    let i = 0;
    while (i < placeholders.length) {
      const frameStart = performance.now();
      let count = 0;

      while (i < placeholders.length && count < BATCH_SIZE && (performance.now() - frameStart) < TIME_BUDGET) {
        const ph = placeholders[i];
        i++;
        count++;

        if (!ph.parentNode) continue;
        const latex = ph.getAttribute('data-latex') || '';
        const displayMode = ph.getAttribute('data-display') === 'true';
        try {
          const html = katex.renderToString(latex, { displayMode, throwOnError: false });
          const span = document.createElement('span');
          span.innerHTML = html;
          ph.replaceWith(span);
        } catch {
          // Keep placeholder text on error
        }
      }

      if (i < placeholders.length) {
        await new Promise<void>(resolve => setTimeout(resolve, 0));
      }
    }
  }

  private escapeAttr(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/"/g, '&quot;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
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
    if (this.highlighter) {
      this.highlighter.dispose();
      this.highlighter = null;
    }
  }
}
