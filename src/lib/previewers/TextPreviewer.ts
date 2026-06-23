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

// Skip syntax highlighting for files larger than 200KB to avoid blocking the main thread
const MAX_HIGHLIGHT_SIZE = 200 * 1024;

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export class TextPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const fileName = filePath.split(/[/\\]/).pop()?.toLowerCase() || '';
    // Dotfiles like .gitignore, .env, .eslintrc
    if (fileName.startsWith('.') && !fileName.includes('.', 1)) return true;
    return SUPPORTED_EXTENSIONS.has(ext) || filePath.toLowerCase().includes('readme');
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);

    if (text.length > MAX_HIGHLIGHT_SIZE) {
      container.innerHTML = `
        <div class="preview-code">
          <div class="large-file-notice" style="padding:8px 12px;background:#3c3836;color:#d79921;font-size:12px;font-family:var(--font-mono);margin-bottom:8px;">文件过大 (${formatSize(text.length)})，已跳过语法高亮</div>
          <pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>
        </div>`;
      return;
    }

    try {
      const html = await codeToHtml(text, {
        lang: this.getLanguage(container.dataset.filePath || ''),
        theme: 'github-dark',
      });
      container.innerHTML = `<div class="preview-code">${html}</div>`;
    } catch {
      container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
    }
  }

  private getLanguage(filePath: string): string {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const fileName = filePath.split(/[/\\]/).pop()?.toLowerCase() || '';
    const dotfileLangMap: Record<string, string> = {
      '.gitignore': 'gitignore', '.gitattributes': 'gitignore',
      '.editorconfig': 'ini', '.env': 'bash', '.eslintrc': 'json',
      '.prettierrc': 'json', '.babelrc': 'json', '.npmrc': 'ini',
      '.dockerignore': 'gitignore', '.eslintignore': 'gitignore',
      '.prettierignore': 'gitignore',
    };
    if (dotfileLangMap[fileName]) return dotfileLangMap[fileName];
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
