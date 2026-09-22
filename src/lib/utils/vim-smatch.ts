import { StateField, StateEffect } from '@codemirror/state';
import { Decoration } from '@codemirror/view';

// Independent StateField for :s live preview (nvim inccommand style)
export const triggerSMatchUpdate = StateEffect.define<void>();
export const clearSMatch = StateEffect.define<void>();

// This will be set by VimOverlay to access the current command buffer
let getOverlayCmdBuf: () => string = () => '';

export function setOverlayCmdBufGetter(getter: () => string) {
  getOverlayCmdBuf = getter;
}

export const sMatchField = StateField.define({
  create() { return Decoration.none as any; },
  update(value, tr) {
    for (const e of tr.effects) {
      if (e.is(clearSMatch)) return Decoration.none as any;
      if (e.is(triggerSMatchUpdate)) {
        const cmd = getOverlayCmdBuf();
        const m = cmd.match(/^(['<,'>]*)([%]?)s(.)/);
        if (!m) return Decoration.none as any;
        const delim = m[3];
        const esc = delim.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const re = new RegExp(`s${esc}([^${esc}]*)(?:${esc}([^${esc}]*))?(?:${esc}([ggiI]*))?`);
        const pm = cmd.match(re);
        if (!pm) return Decoration.none as any;
        const pattern = pm[1];
        const replacement = pm[2] ?? '';
        const flags = pm[3] ?? '';
        const global = flags.includes('g');
        if (!pattern) return Decoration.none as any;
        let regex: RegExp;
        try {
          regex = new RegExp(pattern, flags.replace('g', '') + 'i');
        } catch { return Decoration.none as any; }
        const isVisualRange = m[1] === "'<,'>";
        const hasRange = !isVisualRange && (m[1] !== '' || m[2] !== '');
        const mark = Decoration.mark({ class: replacement ? 'cm-sMatch-replace' : 'cm-sMatch' });
        const decos: any[] = [];
        const doc = tr.state.doc;
        let startLine: number;
        let endLine: number;
        if (isVisualRange) {
          const sel = tr.state.selection.main;
          startLine = doc.lineAt(sel.from).number;
          endLine = doc.lineAt(sel.to).number;
        } else if (hasRange) {
          startLine = 1;
          endLine = doc.lines;
        } else {
          startLine = doc.lineAt(tr.state.selection.main.head).number;
          endLine = startLine;
        }
        for (let i = startLine; i <= endLine; i++) {
          const line = doc.line(i);
          if (global) {
            const lineRegex = new RegExp(pattern, 'gi');
            let m: RegExpExecArray | null;
            while ((m = lineRegex.exec(line.text)) !== null) {
              decos.push(mark.range(line.from + m.index, line.from + m.index + m[0].length));
              if (!m[0].length) break;
            }
          } else {
            const m = line.text.match(regex);
            if (m) decos.push(mark.range(line.from + m.index!, line.from + m.index! + m[0].length));
          }
        }
        return Decoration.set(decos.sort((a, b) => a.from - b.from));
      }
    }
    return value.map(tr.changes);
  },
  provide: f => EditorView.decorations.from(f),
});

// Need EditorView for the provide field
import { EditorView } from 'codemirror';