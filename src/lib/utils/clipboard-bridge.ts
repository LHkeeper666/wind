import { Vim, getCM } from '@replit/codemirror-vim';
import type { EditorView } from 'codemirror';

export interface ClipboardBridge {
  readForPaste(): Promise<'text' | 'empty' | 'fallback'>;
  refresh(): void;
  dispose(): void;
}

let registerPatched = false;
let localWrite: Promise<void> | null = null;
let writeGeneration = 0;

export function isDefaultPut(cm: any, key: string): boolean {
  if (key !== 'p' && key !== 'P') return false;
  const vim = cm.state?.vim;
  const input = vim?.inputState;
  return !!input && !vim.insertMode && !vim.expectLiteralNext
    && !input.registerName && !input.operator
    && input.keyBuffer?.every((part: string) => /^\d*$/.test(part));
}

export function initClipboardBridge(editorView: EditorView, _overlayElement?: HTMLElement): ClipboardBridge {
  const cm = getCM(editorView);
  if (!cm) return { async readForPaste() { return 'fallback'; }, refresh() {}, dispose() {} };

  // 1. Patch pushText: sync yank/delete to system clipboard
  const rc = Vim.getRegisterController();
  if (!registerPatched) {
    const origPushText = rc.pushText.bind(rc);
    rc.pushText = (registerName: string | null | undefined, operator: string, text: string, linewise?: boolean, blockwise?: boolean) => {
      origPushText(registerName, operator, text, linewise, blockwise);
      if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText && (!registerName || registerName === '"' || registerName === '+' || registerName === '*')) {
        const generation = ++writeGeneration;
        const pending = navigator.clipboard.writeText(text).catch(() => {});
        localWrite = pending;
        void pending.finally(() => { if (writeGeneration === generation) localWrite = null; });
      }
    };
    registerPatched = true;
  }

  async function readForPaste(): Promise<'text' | 'empty' | 'fallback'> {
    // A local yank/delete has already updated the Vim register while its system write is pending.
    if (localWrite || typeof navigator === 'undefined' || !navigator.clipboard?.readText) return 'fallback';
    const generation = writeGeneration;
    try {
      const value = await navigator.clipboard.readText();
      if (generation !== writeGeneration || localWrite) return 'fallback';
      rc.unnamedRegister.setText(value);
      return value ? 'text' : 'empty';
    } catch {
      return 'fallback';
    }
  }

  return {
    readForPaste,
    refresh() {},
    dispose() {},
  };
}
