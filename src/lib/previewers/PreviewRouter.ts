import type { Previewer } from './types';
import { TextPreviewer } from './TextPreviewer';
import { MarkdownPreviewer } from './MarkdownPreviewer';
import { ImagePreviewer } from './ImagePreviewer';
import { JsonPreviewer } from './JsonPreviewer';
import { ArchivePreviewer } from './ArchivePreviewer';
import { PdfPreviewer } from './PdfPreviewer';
import { VideoPreviewer } from './VideoPreviewer';

export class PreviewRouter {
  private previewers: Previewer[] = [];
  private currentPreviewer: Previewer | null = null;

  constructor() {
    this.previewers = [
      new ArchivePreviewer(),
      new JsonPreviewer(),
      new MarkdownPreviewer(),
      new PdfPreviewer(),
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
      oldPreviewer?.dispose();
      container.innerHTML = '<p class="preview-unsupported">Unsupported file type</p>';
      this.currentPreviewer = null;
      return;
    }

    // Render into a temporary off-screen container (atomic swap)
    const staging = document.createElement('div');
    staging.style.cssText = 'position:absolute;visibility:hidden;width:100%;height:100%;';
    container.parentElement?.appendChild(staging);
    container.dataset.filePath = filePath;
    staging.dataset.filePath = filePath;
    // Copy thumbnail metadata to staging
    for (const key of ['thumbWidth', 'thumbHeight', 'thumbOriginalSize', 'thumbIsThumbnail']) {
      if (container.dataset[key] !== undefined) {
        staging.dataset[key] = container.dataset[key];
      }
    }
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
