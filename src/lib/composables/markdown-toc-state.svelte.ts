import type { TocHeading } from '$lib/previewers';
import { collectExpandedLines } from '$lib/utils/tab-cache';

export interface MarkdownTocCacheData {
  tocOpen: boolean;
  tocHeadings: TocHeading[];
  tocExpandedLines: number[];
  tocFocused: boolean;
  tocSelectedIndex: number;
}

interface MarkdownTocDeps {
  getPanelElement: () => HTMLElement | undefined;
}

export interface MarkdownTocAPI {
  // Getters
  getTocHeadings: () => TocHeading[];
  getTocActiveLine: () => number;
  getTocFocused: () => boolean;
  getTocOpen: () => boolean;

  // Setters
  setTocOpen: (open: boolean) => void;
  setTocFocused: (focused: boolean) => void;

  // Callbacks (for PreviewPane props)
  onHeadingsChange: (headings: TocHeading[]) => void;
  onActiveLineChange: (line: number) => void;

  // Focus management
  handleTocFocusChange: (focused: boolean) => void;

  // Lifecycle
  reset: () => void;

  // Cache integration
  toCacheSnapshot: (getTocSelectedIndex: () => number) => MarkdownTocCacheData;
  fromCacheSnapshot: (data: MarkdownTocCacheData) => void;
}

export function createMarkdownTocState(deps: MarkdownTocDeps): MarkdownTocAPI {
  let tocHeadings: TocHeading[] = $state([]);
  let tocActiveLine: number = $state(-1);
  let tocFocused: boolean = $state(false);
  let tocOpen: boolean = $state(true);

  function onHeadingsChange(headings: TocHeading[]) {
    tocHeadings = headings;
    tocActiveLine = -1;
  }

  function onActiveLineChange(line: number) {
    tocActiveLine = line;
  }

  function handleTocFocusChange(focused: boolean) {
    tocFocused = focused;
    if (!focused) {
      const el = deps.getPanelElement();
      if (el) el.focus();
    }
  }

  function reset() {
    tocHeadings = [];
    tocActiveLine = -1;
    tocFocused = false;
    tocOpen = true;
  }

  function toCacheSnapshot(getTocSelectedIndex: () => number): MarkdownTocCacheData {
    return {
      tocOpen,
      tocHeadings: [...tocHeadings],
      tocExpandedLines: collectExpandedLines(tocHeadings),
      tocFocused,
      tocSelectedIndex: getTocSelectedIndex(),
    };
  }

  function fromCacheSnapshot(data: MarkdownTocCacheData) {
    tocOpen = data.tocOpen;
    tocFocused = data.tocFocused;
    if (data.tocHeadings && data.tocHeadings.length > 0) {
      tocHeadings = data.tocHeadings;
    }
  }

  return {
    getTocHeadings: () => tocHeadings,
    getTocActiveLine: () => tocActiveLine,
    getTocFocused: () => tocFocused,
    getTocOpen: () => tocOpen,

    setTocOpen(open: boolean) { tocOpen = open; },
    setTocFocused(focused: boolean) { tocFocused = focused; },

    onHeadingsChange,
    onActiveLineChange,
    handleTocFocusChange,
    reset,
    toCacheSnapshot,
    fromCacheSnapshot,
  };
}