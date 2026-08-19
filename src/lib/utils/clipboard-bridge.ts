import { Vim, getCM } from '@replit/codemirror-vim';
import type { EditorView } from 'codemirror';

export interface ClipboardBridge {
  injectClipboard(): void;
  getCache(): string;
  refresh(): void;
  dispose(): void;
}

let clipboardCache = '';
let registerPatched = false;

export function initClipboardBridge(editorView: EditorView, overlayElement?: HTMLElement): ClipboardBridge {
  const cm = getCM(editorView);
  if (!cm) return { injectClipboard() {}, getCache() { return ''; }, refresh() {}, dispose() {} };

  const supportsClipboard = typeof navigator !== 'undefined' && !!navigator.clipboard;

  // 1. Patch pushText: sync yank/delete to system clipboard
  const rc = Vim.getRegisterController();
  if (!registerPatched) {
    const origPushText = rc.pushText.bind(rc);
    rc.pushText = (registerName: string | null | undefined, operator: string, text: string, linewise?: boolean, blockwise?: boolean) => {
      origPushText(registerName, operator, text, linewise, blockwise);
      if (supportsClipboard && (!registerName || registerName === '"' || registerName === '+' || registerName === '*')) {
        clipboardCache = text;
        navigator.clipboard.writeText(text).catch(() => {});
      }
    };
    registerPatched = true;
  }

  // 2. Focus pre-read: cache clipboard for synchronous put

  async function refreshClipboardCache() {
    if (!supportsClipboard) return;
    try {
      clipboardCache = await navigator.clipboard.readText();
    } catch {
      // Clipboard read may be denied in some contexts
    }
  }

  if (overlayElement) {
    overlayElement.addEventListener('focus', refreshClipboardCache);
  }
  editorView.contentDOM.addEventListener('focus', refreshClipboardCache);

  return {
    injectClipboard() {
      if (clipboardCache) {
        rc.unnamedRegister.setText(clipboardCache);
      }
    },
    getCache() { return clipboardCache; },
    refresh: refreshClipboardCache,
    dispose() {
      if (overlayElement) overlayElement.removeEventListener('focus', refreshClipboardCache);
      editorView.contentDOM.removeEventListener('focus', refreshClipboardCache);
    },
  };
}
