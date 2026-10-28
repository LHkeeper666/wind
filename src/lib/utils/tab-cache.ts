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
  editorSelectionAnchor: number;
  editorSelectionHead: number;
  editorScrollTop: number;
  previewScrollTop: number;
  isModified: boolean;
  pdfCurrentPage: number;
  pdfPageCount: number;
  pdfPageDimensions: PdfPageDimensions[];
  pdfOutline: PdfOutlineItem[];
  pdfTocOpen: boolean;
  pdfTocFocused: boolean;
  pdfTocExpanded: string[];
  pdfTocSelectedIndex: number;
  pdfTocScrollTop: number;
  pdfScale: number;
  pdfScrollTop: number;
  fileMtime: number;
  originalFileSize: number;
  textLoadState: 'ready' | 'loading' | 'error' | 'transition';
  tocOpen: boolean;
  tocHeadings: TocHeading[];
  tocExpandedLines: number[];
  tocActiveLine: number;
  tocFocused: boolean;
  tocSelectedIndex: number;
  tocScrollTop: number;
}

export function isValidTextCache(entry: TabEditorCache): boolean {
  if (entry.isModified) return true;
  return entry.textLoadState === 'ready'
    && !(entry.content === '' && entry.originalFileSize > 0);
}

export function shouldReloadCachedFile(entry: TabEditorCache, meta: { size: number; modified: number }, isText: boolean): boolean {
  if (entry.isModified) return false;
  return (isText && meta.size > 0 && entry.content === '' && entry.binaryContent === null)
    || meta.modified !== entry.fileMtime;
}

export function isSamePreviewIdentity(
  current: { tabId: number; path: string | null },
  target: { tabId: number; path: string | null },
): boolean {
  return current.tabId === target.tabId && current.path === target.path;
}

export function textEditAction(state: 'loading' | 'ready' | 'error' | 'transition' | null): 'wait' | 'error' | 'edit' {
  if (state === 'error') return 'error';
  return state === 'ready' ? 'edit' : 'wait';
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
