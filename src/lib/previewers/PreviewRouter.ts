import type { Previewer, TocHeading } from './types';
import { TextPreviewer } from './TextPreviewer';
import { MarkdownPreviewer } from './MarkdownPreviewer';
import { ImagePreviewer } from './ImagePreviewer';
import { JsonPreviewer } from './JsonPreviewer';
import { ArchivePreviewer } from './ArchivePreviewer';
import { VideoPreviewer } from './VideoPreviewer';
import { IpynbPreviewer } from './IpynbPreviewer';

export class PreviewRouter {
  private previewers: Previewer[] = [];
  private currentPreviewer: Previewer | null = null;
  private lastFilePath: string = '';
  onHeadings?: (headings: TocHeading[]) => void;

  constructor() {
    this.previewers = [
      new ArchivePreviewer(),
      new IpynbPreviewer(),
      new JsonPreviewer(),
      new MarkdownPreviewer(),
      new VideoPreviewer(),
      new ImagePreviewer(),
      new TextPreviewer(), // Fallback
    ];
  }

  match(filePath: string): Previewer | null {
    return this.previewers.find(p => p.match(filePath)) || null;
  }

  async preview(filePath: string, content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
    const oldPreviewer = this.currentPreviewer;

    // Find matching previewer
    const previewer = this.match(filePath);
    if (!previewer) {
      this.onHeadings?.([]);
      oldPreviewer?.dispose();
      container.innerHTML = '<p class="preview-unsupported">Unsupported file type</p>';
      this.currentPreviewer = null;
      this.lastFilePath = '';
      return;
    }

    // Incremental update path: same previewer instance + same file + supports update()
    const normalizedPath = filePath.replace(/\//g, '\\').toLowerCase();
    const lastNormalized = this.lastFilePath.replace(/\//g, '\\').toLowerCase();
    if (previewer === oldPreviewer && normalizedPath === lastNormalized && previewer.update) {
      previewer.onHeadings = this.onHeadings;
      await previewer.update(content, container);
      return;
    }

    // Full render path: staging atomic swap
    this.lastFilePath = filePath;
    const staging = document.createElement('div');
    staging.style.cssText = 'position:absolute;visibility:hidden;width:100%;height:100%;';
    container.parentElement?.appendChild(staging);
    container.dataset.filePath = filePath;
    staging.dataset.filePath = filePath;
    for (const key of ['thumbWidth', 'thumbHeight', 'thumbOriginalSize', 'thumbIsThumbnail']) {
      if (container.dataset[key] !== undefined) {
        staging.dataset[key] = container.dataset[key];
      }
    }
    previewer.onHeadings = this.onHeadings;
    await previewer.render(content, staging);

    // Swap: replace old content with new, then clean up old previewer
    container.innerHTML = '';
    while (staging.firstChild) {
      container.appendChild(staging.firstChild);
    }
    staging.remove();
    oldPreviewer?.dispose();

    this.currentPreviewer = previewer;
  }

  dispose(): void {
    if (this.currentPreviewer) {
      this.currentPreviewer.dispose();
      this.currentPreviewer = null;
    }
  }
}
