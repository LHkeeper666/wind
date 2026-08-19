import { Compartment, type Extension, type EditorState } from '@codemirror/state';
import { lineNumbers } from '@codemirror/view';
import type { EditorView } from 'codemirror';
import { vimOptions } from './vim-options';

export const lineNumberCompartment = new Compartment();

let registered = false;
const registeredViews = new Map<EditorView, Compartment>();

function buildLineNumbers(nu: boolean, rnu: boolean): Extension {
  if (!nu && !rnu) return [];
  return lineNumbers({
    formatNumber: (lineNo: number, state: EditorState): string => {
      if (!rnu) return String(lineNo);
      const cursorLine = state.doc.lineAt(state.selection.main.head).number;
      const diff = Math.abs(cursorLine - lineNo);
      if (nu && rnu) {
        return diff === 0 ? String(lineNo) : String(diff);
      }
      return String(diff);
    },
  });
}

function rebuild() {
  const nu = vimOptions.get<boolean>('number');
  const rnu = vimOptions.get<boolean>('relativenumber');
  for (const [view, compartment] of registeredViews) {
    view.dispatch({ effects: compartment.reconfigure(buildLineNumbers(nu, rnu)) });
  }
}

export function setupVimLineNumbers(comp: Compartment, view: EditorView): void {
  registeredViews.set(view, comp);

  if (!registered) {
    registered = true;

    vimOptions.register({
      name: 'number',
      shortName: 'nu',
      type: 'boolean',
      defaultValue: true,
      persist: true,
    });
    vimOptions.register({
      name: 'relativenumber',
      shortName: 'rnu',
      type: 'boolean',
      defaultValue: false,
      persist: true,
    });

    vimOptions.onChange('number', () => rebuild());
    vimOptions.onChange('relativenumber', () => rebuild());
  }

  // Sync editor with current option state. On first init, config may not be
  // loaded yet, so rebuild uses defaults. Once config loads, a final rebuild
  // applies persisted values (safe: compartment reconfigure only affects gutter).
  rebuild();
  vimOptions.load().then(() => {
    if (registeredViews.has(view)) rebuild();
  });
}

export function teardownVimLineNumbers(view: EditorView): void {
  registeredViews.delete(view);
}
