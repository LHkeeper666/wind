import type { Previewer } from './types';
import { codeToHtml } from 'shiki';

const SUPPORTED_EXTENSIONS = new Set([
  'js', 'ts', 'jsx', 'tsx', 'py', 'java', 'go', 'rs', 'c', 'cpp', 'h', 'hpp',
  'css', 'scss', 'less', 'html', 'xml', 'json', 'yaml', 'yml', 'toml', 'ini',
  'sh', 'bash', 'zsh', 'fish', 'ps1', 'bat', 'cmd', 'sql', 'md', 'txt',
  'rb', 'php', 'swift', 'kt', 'kts', 'scala', 'r', 'lua', 'pl', 'pm',
  'hs', 'ml', 'ex', 'exs', 'erl', 'clj', 'lisp', 'el', 'vim', 'lua',
  'dockerfile', 'makefile', 'cmake', 'gradle', 'sbt', 'vue', 'svelte',
]);

export class TextPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return SUPPORTED_EXTENSIONS.has(ext) || filePath.toLowerCase().includes('readme');
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    try {
      const html = await codeToHtml(text, {
        lang: this.getLanguage(container.dataset.filePath || ''),
        theme: 'github-dark',
      });
      container.innerHTML = `<div class="preview-code">${html}</div>`;
    } catch {
      // Fallback to plain text
      container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
    }
  }

  private getLanguage(filePath: string): string {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = {
      'js': 'javascript', 'ts': 'typescript', 'jsx': 'jsx', 'tsx': 'tsx',
      'py': 'python', 'rb': 'ruby', 'rs': 'rust', 'go': 'go',
      'java': 'java', 'kt': 'kotlin', 'swift': 'swift', 'c': 'c',
      'cpp': 'cpp', 'h': 'c', 'hpp': 'cpp', 'cs': 'csharp',
      'php': 'php', 'scala': 'scala', 'r': 'r', 'lua': 'lua',
      'sh': 'bash', 'bash': 'bash', 'zsh': 'zsh', 'ps1': 'powershell',
      'sql': 'sql', 'html': 'html', 'xml': 'xml', 'css': 'css',
      'scss': 'scss', 'less': 'less', 'json': 'json', 'yaml': 'yaml',
      'yml': 'yaml', 'toml': 'toml', 'md': 'markdown', 'vue': 'vue',
      'svelte': 'svelte', 'dockerfile': 'dockerfile',
    };
    return langMap[ext] || 'text';
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
