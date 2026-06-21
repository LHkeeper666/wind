import type { Previewer } from './types';
import { invoke } from '@tauri-apps/api/core';

interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export class DirectoryPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    // This is checked by the caller (PreviewEditor) before invoking
    return false;
  }

  async render(_content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const filePath = container.dataset.filePath || '';
    if (!filePath) return;

    try {
      const entries = await invoke<FileEntry[]>('read_directory', { path: filePath });
      container.innerHTML = this.renderEntries(entries);
    } catch (err) {
      container.innerHTML = `<p class="preview-unsupported">Failed to read directory: ${err}</p>`;
    }
  }

  private renderEntries(entries: FileEntry[]): string {
    if (entries.length === 0) {
      return '<p class="preview-empty">Empty directory</p>';
    }

    const rows = entries.map(entry => {
      const icon = entry.is_dir ? '📁' : '📄';
      const size = entry.is_dir || entry.size == null ? '' : formatSize(entry.size);
      const sizeClass = entry.is_dir ? 'dir' : 'file';
      return `<div class="dir-entry">
        <span class="entry-icon">${icon}</span>
        <span class="entry-name">${this.escapeHtml(entry.name)}</span>
        <span class="entry-size ${sizeClass}">${size}</span>
      </div>`;
    });

    return `<div class="dir-list">${rows.join('')}</div>`;
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
