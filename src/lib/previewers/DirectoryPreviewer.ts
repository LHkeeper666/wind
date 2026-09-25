import type { Previewer } from './types';
import { invoke } from '@tauri-apps/api/core';
import { directoryCache, type DirCacheEntry } from '$lib/utils/directory-cache';
import type { FileEntry } from '$lib/types/file-explorer';
import { formatSize } from '$lib/utils/file-types';

export class DirectoryPreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    return false;
  }

  async render(_content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const filePath = container.dataset.filePath || '';
    if (!filePath) return;

    // Use shared cache so DirectoryPanel can benefit from preview-loaded data
    if (directoryCache.has(filePath)) {
      container.innerHTML = this.renderEntries(directoryCache.get(filePath)!);
      return;
    }

    try {
      const entries = await invoke<FileEntry[]>('read_directory', { path: filePath });
      // Cache without .. entry (matching DirectoryPanel's cache convention)
      directoryCache.set(filePath, entries.filter(f => f.name !== '..') as DirCacheEntry[]);
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
      const name = entry.is_dir ? this.escapeHtml(entry.name) + '/' : this.escapeHtml(entry.name);
      const nameClass = entry.is_dir ? 'entry-name is-dir' : 'entry-name';
      const size = entry.is_dir || entry.size == null ? '' : formatSize(entry.size);
      const sizeClass = entry.is_dir ? 'dir' : 'file';
      return `<div class="dir-entry">
        <span class="${nameClass}">${name}</span>
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
