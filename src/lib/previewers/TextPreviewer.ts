import type { Previewer } from './types';
import { codeToHtml } from 'shiki';

// Used only for language detection, NOT for match filtering
const KNOWN_LANG_EXTENSIONS = new Set([
  'js', 'ts', 'jsx', 'tsx', 'py', 'java', 'go', 'rs', 'c', 'cpp', 'h', 'hpp',
  'css', 'scss', 'less', 'html', 'xml', 'json', 'yaml', 'yml', 'toml', 'ini',
  'sh', 'bash', 'zsh', 'fish', 'ps1', 'bat', 'cmd', 'sql', 'md', 'txt',
  'rb', 'php', 'swift', 'kt', 'kts', 'scala', 'r', 'lua', 'pl', 'pm',
  'hs', 'ml', 'ex', 'exs', 'erl', 'clj', 'lisp', 'el', 'vim',
  'dockerfile', 'makefile', 'cmake', 'gradle', 'sbt', 'vue', 'svelte',
]);

// Binary extensions — these files get hex dump
const BINARY_EXTENSIONS = new Set([
  // Executables & libraries
  'exe', 'dll', 'so', 'dylib', 'bin', 'obj', 'o', 'a', 'lib', 'sys', 'drv',
  // Archives
  'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'zst', 'lz4', 'cab',
  // Media - audio
  'mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a', 'opus', 'mid', 'midi',
  // Media - video
  'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg',
  // Media - image (binary ones, SVG is text)
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'psd', 'raw', 'cr2', 'nef',
  // Documents
  'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp',
  // Fonts
  'ttf', 'otf', 'woff', 'woff2', 'eot',
  // Databases & compiled
  'db', 'sqlite', 'sqlite3', 'mdb', 'accdb', 'class', 'pyc', 'pyo',
  // Other binary
  'iso', 'img', 'vhd', 'vhdx', 'qcow2', 'wasm', 'jar',
]);

// Special filename → language mapping
const FILENAME_LANG_MAP: Record<string, string> = {
  'makefile': 'makefile',
  'cmakelists.txt': 'makefile',
  'dockerfile': 'dockerfile',
  'docker-compose.yml': 'yaml',
  'docker-compose.yaml': 'yaml',
  'vagrantfile': 'ruby',
  'gemfile': 'ruby',
  'rakefile': 'ruby',
  '.gitignore': 'gitignore',
  '.gitattributes': 'gitignore',
  '.editorconfig': 'ini',
  '.env': 'bash',
  '.eslintrc': 'json',
  '.prettierrc': 'json',
  '.babelrc': 'json',
  '.npmrc': 'ini',
  '.dockerignore': 'gitignore',
  '.eslintignore': 'gitignore',
  '.prettierignore': 'gitignore',
};

// Skip syntax highlighting for files larger than 200KB to avoid blocking the main thread
const MAX_HIGHLIGHT_SIZE = 200 * 1024;

// Max hex dump size (64KB of binary → ~4096 lines)
const MAX_HEX_DUMP_BYTES = 64 * 1024;

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function isBinaryContent(data: ArrayBuffer): boolean {
  const bytes = new Uint8Array(data);
  const checkLen = Math.min(bytes.length, 8192);
  for (let i = 0; i < checkLen; i++) {
    if (bytes[i] === 0) return true;
  }
  return false;
}

export class TextPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(_filePath: string): boolean {
    // Always match — TextPreviewer is the fallback for all files
    // that aren't handled by other previewers (image, pdf, video, archive, markdown, json)
    return true;
  }

  async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const filePath = container.dataset.filePath || '';
    const ext = filePath.split('.').pop()?.toLowerCase() || '';

    // Determine if this is binary
    const isBinary = BINARY_EXTENSIONS.has(ext) ||
      (typeof content !== 'string' && isBinaryContent(content));

    if (isBinary) {
      this.renderHexDump(content, container);
      return;
    }

    // Text rendering
    const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
    const originalSize = parseInt(container.dataset.originalFileSize || '0', 10);
    const isTruncated = originalSize > text.length;

    let notice = '';
    if (isTruncated) {
      notice = `<div class="hex-notice">文件过大 (${formatSize(originalSize)})，仅显示前 ${formatSize(text.length)}</div>`;
    }

    if (text.length > MAX_HIGHLIGHT_SIZE) {
      container.innerHTML = `
        <div class="preview-code">
          ${notice}
          <div class="large-file-notice" style="padding:8px 12px;background:#3c3836;color:#d79921;font-size:12px;font-family:var(--font-mono);margin-bottom:8px;">已跳过语法高亮</div>
          <pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>
        </div>`;
      return;
    }

    try {
      const html = await codeToHtml(text, {
        lang: this.getLanguage(filePath),
        theme: 'github-dark',
      });
      container.innerHTML = `<div class="preview-code">${notice}${html}</div>`;
    } catch {
      container.innerHTML = `<div class="preview-code">${notice}<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre></div>`;
    }
  }

  private renderHexDump(content: string | ArrayBuffer, container: HTMLElement): void {
    let bytes: Uint8Array;
    if (typeof content === 'string') {
      bytes = new TextEncoder().encode(content);
    } else {
      bytes = new Uint8Array(content);
    }

    const totalSize = bytes.length;
    const dumpBytes = bytes.slice(0, MAX_HEX_DUMP_BYTES);
    const lines: string[] = [];

    for (let offset = 0; offset < dumpBytes.length; offset += 16) {
      const hexParts: string[] = [];
      const asciiParts: string[] = [];

      for (let i = 0; i < 16; i++) {
        if (offset + i < dumpBytes.length) {
          const byte = dumpBytes[offset + i];
          hexParts.push(byte.toString(16).padStart(2, '0'));
          asciiParts.push(byte >= 0x20 && byte <= 0x7e ? String.fromCharCode(byte) : '.');
        } else {
          hexParts.push('  ');
          asciiParts.push(' ');
        }
        if (i === 7) hexParts.push(''); // extra space at midpoint
      }

      const addr = offset.toString(16).padStart(8, '0');
      const hex = hexParts.join(' ');
      const ascii = asciiParts.join('');
      lines.push(`${addr}  ${hex}  |${ascii}|`);
    }

    const truncated = totalSize > MAX_HEX_DUMP_BYTES;
    const notice = truncated
      ? `<div class="hex-notice">二进制文件 · ${formatSize(totalSize)} · 仅显示前 ${formatSize(MAX_HEX_DUMP_BYTES)}</div>`
      : `<div class="hex-notice">二进制文件 · ${formatSize(totalSize)}</div>`;

    container.innerHTML = `
      <div class="preview-hex">
        ${notice}
        <pre class="hex-dump"><code>${this.escapeHtml(lines.join('\n'))}</code></pre>
      </div>`;
  }

  private getLanguage(filePath: string): string {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    const fileName = filePath.split(/[/\\]/).pop()?.toLowerCase() || '';

    // Check special filenames first
    if (FILENAME_LANG_MAP[fileName]) return FILENAME_LANG_MAP[fileName];

    // .env.* files
    if (fileName.startsWith('.env.')) return 'bash';

    // Dotfiles like .gitignore, .env, .eslintrc
    if (fileName.startsWith('.') && !fileName.includes('.', 1)) {
      return FILENAME_LANG_MAP[fileName] || 'text';
    }

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
