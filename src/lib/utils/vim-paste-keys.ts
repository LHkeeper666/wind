import { Vim, getCM } from '@replit/codemirror-vim';
import type { EditorView } from 'codemirror';
import { isDefaultPut, type ClipboardBridge } from './clipboard-bridge';

type KeyPress = {
  key: string;
  view: EditorView;
  bridge: ClipboardBridge | null;
  valid: () => boolean;
  after: (cm: any) => void;
};

export function createVimKeyQueue() {
  const keys: KeyPress[] = [];
  let running = false;

  async function drain() {
    if (running) return;
    running = true;
    try {
      while (keys.length) {
        const press = keys.shift()!;
        if (!press.valid()) continue;
        const cm = getCM(press.view);
        if (!cm) continue;
        if (press.bridge && isDefaultPut(cm, press.key)) {
          await press.bridge.readForPaste();
          if (!press.valid()) continue;
        }
        (Vim as any).multiSelectHandleKey?.(cm, press.key, 'user');
        press.after(cm);
      }
    } finally {
      running = false;
    }
  }

  return (press: KeyPress) => {
    keys.push(press);
    void drain();
  };
}
