import type { Previewer } from './types';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';

interface ArchiveEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
}

function formatSize(bytes: number | null): string {
  if (bytes === null || bytes === undefined) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export class ArchivePreviewer implements Previewer {
  private container: HTMLElement | null = null;

  match(filePath: string): boolean {
    const lower = filePath.toLowerCase();
    return lower.endsWith('.zip') || lower.endsWith('.tar') || lower.endsWith('.tar.gz')
      || lower.endsWith('.tgz') || lower.endsWith('.7z');
  }

  async render(_content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    this.container = container;
    const filePath = container.dataset.filePath || '';
    if (!filePath) return;

    try {
      const entries = await invokeArchiveWithOptionalPassword<ArchiveEntry[]>('read_archive_directory', {
        archivePath: filePath,
        internalPath: '',
      }, 'password', 'Archive password:', { promptOnRequired: false });
      if (entries === null) {
        container.innerHTML = '<p class="preview-unsupported">Password required to preview this archive. Press l to enter it.</p>';
        return;
      }
      container.innerHTML = this.renderEntries(filePath, entries);
    } catch (err) {
      container.innerHTML = `<p class="preview-unsupported">Failed to read archive: ${err}</p>`;
    }
  }

  private renderEntries(filePath: string, entries: ArchiveEntry[]): string {
    const fileName = filePath.split(/[/\\]/).pop() || filePath;
    const fileCount = entries.filter(e => !e.is_dir).length;
    const totalSize = entries.reduce((sum, e) => sum + (e.size ?? 0), 0);

    const header = `<div class="archive-header">
      <span class="archive-name">${this.escapeHtml(fileName)}</span>
      <span class="archive-meta">${fileCount} files${totalSize > 0 ? ' · ' + formatSize(totalSize) : ''}</span>
    </div>`;

    if (entries.length === 0) {
      return `${header}<p class="preview-empty">Empty archive</p>`;
    }

    const rows = entries.map(entry => {
      const name = entry.is_dir ? this.escapeHtml(entry.name) + '/' : this.escapeHtml(entry.name);
      const nameClass = entry.is_dir ? 'entry-name is-dir' : 'entry-name';
      const size = entry.is_dir ? '' : formatSize(entry.size);
      const sizeClass = entry.is_dir ? 'dir' : 'file';
      return `<div class="dir-entry">
        <span class="${nameClass}">${name}</span>
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
