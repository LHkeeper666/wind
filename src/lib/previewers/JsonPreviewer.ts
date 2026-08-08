import type { Previewer } from './types';

const MAX_DEPTH = 50;
const MAX_NODES = 20000;

export class JsonPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private nodeCount = 0;
  private limited = false;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'json';
  }

  render(content: string | ArrayBuffer, container: HTMLElement): void {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    let json: unknown;
    let truncated = false;

    try {
      json = JSON.parse(text);
    } catch {
      const lastBrace = text.lastIndexOf('}');
      const lastBracket = text.lastIndexOf(']');
      const lastClose = Math.max(lastBrace, lastBracket);
      if (lastClose > 0) {
        const partial = text.substring(0, lastClose + 1);
        try {
          json = JSON.parse(partial);
          truncated = true;
        } catch { /* still invalid */ }
      }
      if (!json) {
        container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
        return;
      }
    }

    this.nodeCount = 0;
    this.limited = false;
    const html = this.renderJsonTree(json, 0);

    let notice = '';
    if (truncated) notice += '<div class="hex-notice">File truncated — partial preview</div>';
    if (this.limited) notice += `<div class="hex-notice">Preview limited (${this.nodeCount}+ nodes, max depth ${MAX_DEPTH})</div>`;

    container.innerHTML = `<div class="preview-json">${notice}${html}</div>`;

    container.querySelectorAll('.json-toggle').forEach((el) => {
      el.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        const bracket = target.nextElementSibling;
        const content = bracket?.nextElementSibling;
        if (content) {
          content.classList.toggle('collapsed');
          target.textContent = content.classList.contains('collapsed') ? '▶' : '▼';
        }
      });
    });
  }

  private renderJsonTree(value: unknown, depth: number): string {
    this.nodeCount++;
    if (this.limited) return '';

    if (depth >= MAX_DEPTH) {
      this.limited = true;
      return '<span class="json-null">…</span>';
    }
    if (this.nodeCount > MAX_NODES) {
      this.limited = true;
      return '<span class="json-null">…</span>';
    }

    if (value === null) return '<span class="json-null">null</span>';
    if (value === undefined) return '<span class="json-undefined">undefined</span>';

    const type = typeof value;

    if (type === 'string') {
      const s = value as string;
      const display = s.length > 500 ? `"${this.escapeHtml(s.substring(0, 500))}…"` : `"${this.escapeHtml(s)}"`;
      return `<span class="json-string">${display}</span>`;
    }
    if (type === 'number') {
      return `<span class="json-number">${value}</span>`;
    }
    if (type === 'boolean') {
      return `<span class="json-boolean">${value}</span>`;
    }

    if (Array.isArray(value)) {
      if (value.length === 0) return '<span class="json-bracket">[]</span>';
      if (this.limited) return '';

      const childIndent = 20;
      const items: string[] = [];
      for (let i = 0; i < value.length && !this.limited; i++) {
        const comma = i < value.length - 1 ? ',' : '';
        items.push(`<div class="json-item" style="padding-left: ${childIndent}px">${this.renderJsonTree(value[i], depth + 1)}${comma}</div>`);
      }

      return `<span class="json-toggle">▼</span><span class="json-bracket">[</span><div class="json-content">${items.join('')}</div><span class="json-bracket">]</span>`;
    }

    if (type === 'object') {
      const obj = value as Record<string, unknown>;
      const keys = Object.keys(obj);
      if (keys.length === 0) return '<span class="json-bracket">{}</span>';
      if (this.limited) return '';

      const childIndent = 20;
      const items: string[] = [];
      for (let i = 0; i < keys.length && !this.limited; i++) {
        const key = keys[i];
        const comma = i < keys.length - 1 ? ',' : '';
        items.push(`<div class="json-item" style="padding-left: ${childIndent}px"><span class="json-key">"${this.escapeHtml(key)}"</span>: ${this.renderJsonTree(obj[key], depth + 1)}${comma}</div>`);
      }

      return `<span class="json-toggle">▼</span><span class="json-bracket">{</span><div class="json-content">${items.join('')}</div><span class="json-bracket">}</span>`;
    }

    return String(value);
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
