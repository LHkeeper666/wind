import type { TocHeading } from '$lib/previewers';
import type { PdfPageDimensions, PdfOutlineItem } from '$lib/utils/pdf-shared';

export interface TextContentSnapshot {
  tabId: number;
  path: string;
  generation: number;
  content: string;
}

export interface TabEditorCache {
  filePath: string;
  content: string;
  savedContent: string;
  binaryContent: ArrayBuffer | null;
  mode: 'global-normal' | 'editor-normal' | 'editor-insert';
  editorCursorPos: number;
  editorScrollTop: number;
  previewScrollTop: number;
  isModified: boolean;
  pdfCurrentPage: number;
  pdfPageCount: number;
  pdfPageDimensions: PdfPageDimensions[];
  pdfOutline: PdfOutlineItem[];
  pdfTocOpen: boolean;
  fileMtime: number;
  tocOpen: boolean;
  tocHeadings: TocHeading[];
  tocExpandedLines: number[];
  tocFocused: boolean;
  tocSelectedIndex: number;
}

export function collectExpandedLines(headings: TocHeading[]): number[] {
  const lines: number[] = [];
  function walk(items: TocHeading[]) {
    for (const h of items) {
      if (h.expanded && h.children.length > 0) {
        lines.push(h.line);
        walk(h.children);
      }
    }
  }
  walk(headings);
  return lines;
}

export function restoreExpandedLines(headings: TocHeading[], lines: Set<number>) {
  function walk(items: TocHeading[]) {
    for (const h of items) {
      if (lines.has(h.line) && h.children.length > 0) {
        h.expanded = true;
        walk(h.children);
      }
    }
  }
  walk(headings);
}

export class TabCacheManager {
  private cache = new Map<number, TabEditorCache>();
  private renderVersions = new Map<number, number>();

  get(tabId: number): TabEditorCache | undefined {
    return this.cache.get(tabId);
  }

  set(tabId: number, entry: TabEditorCache): void {
    this.cache.set(tabId, entry);
  }

  delete(tabId: number): void {
    this.cache.delete(tabId);
    this.renderVersions.delete(tabId);
  }

  has(tabId: number): boolean {
    return this.cache.has(tabId);
  }

  requestRender(tabId: number): number {
    const version = (this.renderVersions.get(tabId) ?? 0) + 1;
    this.renderVersions.set(tabId, version);
    return version;
  }

  isCurrentRender(tabId: number, path: string, version: number, currentTabId: number, currentPath: string): boolean {
    return currentTabId === tabId
      && currentPath === path
      && this.renderVersions.get(tabId) === version;
  }

  getRenderVersion(tabId: number): number {
    return this.renderVersions.get(tabId) ?? 0;
  }
}