import type { Previewer } from './types';
// @ts-ignore - markdown-it has no bundled types
import MarkdownIt from 'markdown-it';
import { createHighlighter, type Highlighter } from 'shiki';

interface IpynbCell {
  cell_type: string;
  source: string | string[];
  metadata?: Record<string, unknown>;
  execution_count?: number | null;
  outputs?: IpynbOutput[];
}

interface IpynbOutput {
  output_type: string;
  name?: string;
  text?: string | string[];
  data?: Record<string, string | string[]>;
  execution_count?: number | null;
  ename?: string;
  evalue?: string;
  traceback?: string[];
}

interface IpynbNotebook {
  cells?: IpynbCell[];
  metadata?: Record<string, unknown>;
  nbformat?: number;
  nbformat_minor?: number;
}

function joinSource(source: string | string[]): string {
  if (Array.isArray(source)) return source.join('');
  return source;
}

export class IpynbPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private md: any;
  private highlighter: Highlighter | null = null;
  private requestId: number = 0;

  constructor() {
    this.md = new MarkdownIt({ html: false, linkify: true, typographer: false });
  }

  private async getHighlighter(): Promise<Highlighter> {
    if (!this.highlighter) {
      this.highlighter = await createHighlighter({
        themes: ['github-dark'],
        langs: ['python', 'javascript', 'typescript', 'json', 'html', 'css', 'rust', 'bash', 'r', 'sql', 'java', 'markdown'],
      });
    }
    return this.highlighter;
  }

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'ipynb';
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    this.requestId++;
    const reqId = this.requestId;

    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    let notebook: IpynbNotebook;
    try {
      notebook = JSON.parse(text);
    } catch {
      if (reqId !== this.requestId) return;
      container.innerHTML = this.buildError('Failed to parse .ipynb file: invalid JSON', text);
      return;
    }

    if (!notebook.cells || !Array.isArray(notebook.cells)) {
      if (reqId !== this.requestId) return;
      container.innerHTML = this.buildError('Invalid .ipynb format: missing "cells" array', text);
      return;
    }

    const kernelspec = (notebook.metadata as any)?.kernelspec;
    const lang = kernelspec?.language as string || 'python';
    const cells = notebook.cells;
    const highlighter = await this.getHighlighter();
    if (reqId !== this.requestId) return;

    // Pre-load the target language for Shiki if not already loaded
    const hlLang = this.mapLanguage(lang);
    if (!highlighter.getLoadedLanguages().includes(hlLang)) {
      try { await highlighter.loadLanguage(hlLang as any); } catch { /* fall back to text */ }
    }
    if (reqId !== this.requestId) return;

    const cellHtmls: string[] = [];
    for (let i = 0; i < cells.length; i++) {
      cellHtmls.push(this.renderCell(cells[i], i, lang, highlighter));
    }

    const nbformat = notebook.nbformat ?? '?';
    const nbformatMinor = notebook.nbformat_minor ?? '?';
    const metaHtml = `<div class="ipynb-meta">notebook v${nbformat}.${nbformatMinor} · ${cells.length} cells</div>`;

    container.innerHTML = `<div class="preview-ipynb">${metaHtml}<div class="ipynb-cells">${cellHtmls.join('\n')}</div></div>`;
  }

  private renderCell(cell: IpynbCell, index: number, lang: string, hl: Highlighter): string {
    const source = joinSource(cell.source);
    const badge = `<span class="ipynb-badge ipynb-badge-${cell.cell_type}">${cell.cell_type}</span>`;

    switch (cell.cell_type) {
      case 'markdown':
        return `<div class="ipynb-cell ipynb-cell-md" data-cell="${index}">
          <div class="ipynb-cell-header">${badge}</div>
          <div class="ipynb-cell-body">${this.md.render(source)}</div>
        </div>`;

      case 'code': {
        const execLabel = cell.execution_count != null ? `In [${cell.execution_count}]:` : 'In [ ]:';
        const codeHtml = this.highlightCode(hl, source, this.mapLanguage(lang));
        const outputsHtml = this.renderOutputs(cell.outputs || [], cell.execution_count ?? null);

        return `<div class="ipynb-cell ipynb-cell-code" data-cell="${index}">
          <div class="ipynb-cell-header">${badge}</div>
          <div class="ipynb-cell-input">
            <span class="ipynb-exec-label">${execLabel}</span>
            <div class="ipynb-code">${codeHtml}</div>
          </div>
          ${outputsHtml}
        </div>`;
      }

      case 'raw':
      default:
        return `<div class="ipynb-cell ipynb-cell-raw" data-cell="${index}">
          <div class="ipynb-cell-header">${badge}</div>
          <div class="ipynb-cell-body"><pre>${this.escapeHtml(source)}</pre></div>
        </div>`;
    }
  }

  private renderOutputs(outputs: IpynbOutput[], execCount: number | null): string {
    if (outputs.length === 0) return '';
    const parts: string[] = [];
    let outIdx = 0;

    for (const output of outputs) {
      switch (output.output_type) {
        case 'stream': {
          const name = output.name || 'stdout';
          const text = joinSource(output.text || '');
          if (!text.trim()) continue;
          parts.push(`<div class="ipynb-output ipynb-output-stream ipynb-output-stream-${name}">
            <span class="ipynb-exec-label">${name}:</span>
            <pre>${this.escapeHtml(text)}</pre>
          </div>`);
          break;
        }
        case 'execute_result': {
          const label = output.execution_count != null ? `Out [${output.execution_count}]:` : 'Out:';
          parts.push(`<div class="ipynb-output ipynb-output-result">
            <span class="ipynb-exec-label">${label}</span>
            ${this.renderOutputData(output.data || {})}
          </div>`);
          outIdx++;
          break;
        }
        case 'display_data': {
          parts.push(`<div class="ipynb-output ipynb-output-display">
            ${this.renderOutputData(output.data || {})}
          </div>`);
          break;
        }
        case 'error': {
          const traceback = (output.traceback || []).join('\n');
          parts.push(`<div class="ipynb-output ipynb-output-error">
            <span class="ipynb-exec-label">Error: ${this.escapeHtml(output.ename || '')}</span>
            <pre>${this.escapeHtml(output.evalue || '')}\n${this.escapeHtml(traceback)}</pre>
          </div>`);
          break;
        }
      }
    }

    return parts.length > 0 ? `<div class="ipynb-outputs">${parts.join('\n')}</div>` : '';
  }

  private renderOutputData(data: Record<string, string | string[]>): string {
    const parts: string[] = [];

    // image/png first (visual output is more prominent)
    if (data['image/png']) {
      const png = joinSource(data['image/png']);
      parts.push(`<img class="ipynb-output-img" src="data:image/png;base64,${png}" alt="output image">`);
    }
    if (data['image/jpeg']) {
      const jpg = joinSource(data['image/jpeg']);
      parts.push(`<img class="ipynb-output-img" src="data:image/jpeg;base64,${jpg}" alt="output image">`);
    }

    // text/plain
    if (data['text/plain']) {
      const text = joinSource(data['text/plain']);
      parts.push(`<pre>${this.escapeHtml(text)}</pre>`);
    }

    return parts.join('\n');
  }

  private highlightCode(hl: Highlighter, code: string, lang: string): string {
    try {
      const loaded = hl.getLoadedLanguages();
      const effectiveLang = loaded.includes(lang) ? lang : 'text';
      return hl.codeToHtml(code, { lang: effectiveLang, theme: 'github-dark' });
    } catch {
      return `<pre><code>${this.escapeHtml(code)}</code></pre>`;
    }
  }

  private mapLanguage(kernelLang: string): string {
    const map: Record<string, string> = {
      python: 'python', py: 'python',
      javascript: 'javascript', js: 'javascript',
      typescript: 'typescript', ts: 'typescript',
      r: 'r',
      julia: 'julia',
      scala: 'scala',
      java: 'java',
      bash: 'bash', sh: 'bash',
      sql: 'sql',
      ruby: 'ruby',
    };
    return map[kernelLang?.toLowerCase()] || 'text';
  }

  private buildError(message: string, rawText: string): string {
    return `<div class="preview-ipynb">
      <div class="ipynb-error-banner">${this.escapeHtml(message)}</div>
      <pre class="preview-plain"><code>${this.escapeHtml(rawText)}</code></pre>
    </div>`;
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
