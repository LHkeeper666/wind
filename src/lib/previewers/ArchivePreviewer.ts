import type { Previewer } from './types';
import { invoke } from '@tauri-apps/api/core';

interface ArchiveEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

const ARCHIVE_EXTENSIONS = new Set(['zip']);

export class ArchivePreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ARCHIVE_EXTENSIONS.has(ext);
  }

  async render(_content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const filePath = container.dataset.filePath || '';
    if (!filePath) return;

    try {
      const entries = await invoke<ArchiveEntry[]>('list_archive_entries', { path: filePath });
      container.innerHTML = this.renderEntries(filePath, entries);
    } catch (err) {
      container.innerHTML = `<p class="preview-unsupported">Failed to read archive: ${err}</p>`;
    }
  }

  private renderEntries(filePath: string, entries: ArchiveEntry[]): string {
    const fileName = filePath.split(/[/\\]/).pop() || filePath;
    const totalSize = entries.reduce((sum, e) => sum + e.size, 0);
    const fileCount = entries.filter(e => !e.is_dir).length;

    const header = `<div class="archive-header">
      <span class="archive-icon">📦</span>
      <span class="archive-name">${this.escapeHtml(fileName)}</span>
      <span class="archive-meta">${fileCount} files · ${formatSize(totalSize)}</span>
    </div>`;

    if (entries.length === 0) {
      return `${header}<p class="preview-empty">Empty archive</p>`;
    }

    const rows = entries.map(entry => {
      const icon = entry.is_dir ? '📁' : '📄';
      const size = entry.is_dir ? '' : formatSize(entry.size);
      const sizeClass = entry.is_dir ? 'dir' : 'file';
      return `<div class="dir-entry">
        <span class="entry-icon">${icon}</span>
        <span class="entry-name">${this.escapeHtml(entry.name)}</span>
        <span class="entry-size ${sizeClass}">${size}</span>
      </div>`;
    });

    return `${header}<div class="dir-list">${rows.join('')}</div>`;
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
