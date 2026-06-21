import type { Previewer } from './types';

export class JsonPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'json';
  }

  render(content: string | ArrayBuffer, container: HTMLElement): void {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    try {
      const json = JSON.parse(text);
      const html = this.renderJsonTree(json, 0);
      container.innerHTML = `<div class="preview-json">${html}</div>`;

      // Add click handlers for collapsible nodes
      container.querySelectorAll('.json-toggle').forEach((el) => {
        el.addEventListener('click', (e) => {
          const target = e.target as HTMLElement;
          const content = target.nextElementSibling;
          if (content) {
            content.classList.toggle('collapsed');
            target.textContent = content.classList.contains('collapsed') ? '▶' : '▼';
          }
        });
      });
    } catch {
      container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
    }
  }

  private renderJsonTree(value: unknown, depth: number): string {
    if (value === null) return '<span class="json-null">null</span>';
    if (value === undefined) return '<span class="json-undefined">undefined</span>';

    const type = typeof value;

    if (type === 'string') {
      return `<span class="json-string">"${this.escapeHtml(value as string)}"</span>`;
    }
    if (type === 'number') {
      return `<span class="json-number">${value}</span>`;
    }
    if (type === 'boolean') {
      return `<span class="json-boolean">${value}</span>`;
    }

    if (Array.isArray(value)) {
      if (value.length === 0) return '<span class="json-bracket">[]</span>';

      const items = value.map((item, i) => {
        const comma = i < value.length - 1 ? ',' : '';
        return `<div class="json-item" style="padding-left: ${(depth + 1) * 20}px">${this.renderJsonTree(item, depth + 1)}${comma}</div>`;
      }).join('');

      return `<span class="json-toggle">▼</span><span class="json-bracket">[</span><div class="json-content">${items}</div><span class="json-bracket" style="padding-left: ${depth * 20}px">]</span>`;
    }

    if (type === 'object') {
      const obj = value as Record<string, unknown>;
      const keys = Object.keys(obj);
      if (keys.length === 0) return '<span class="json-bracket">{}</span>';

      const items = keys.map((key, i) => {
        const comma = i < keys.length - 1 ? ',' : '';
        return `<div class="json-item" style="padding-left: ${(depth + 1) * 20}px"><span class="json-key">"${this.escapeHtml(key)}"</span>: ${this.renderJsonTree(obj[key], depth + 1)}${comma}</div>`;
      }).join('');

      return `<span class="json-toggle">▼</span><span class="json-bracket">{</span><div class="json-content">${items}</div><span class="json-bracket" style="padding-left: ${depth * 20}px">}</span>`;
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
