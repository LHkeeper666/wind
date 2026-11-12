import { EditorState, Compartment } from '@codemirror/state';
import { EditorView, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, highlightActiveLine } from '@codemirror/view';
import { syntaxHighlighting, defaultHighlightStyle, foldGutter } from '@codemirror/language';
import { getLanguage } from '$lib/utils/language';
import { getSyntaxTheme, gruvboxTheme, editorFontTheme } from '$lib/utils/editor-theme';
import { EDITOR_TAB_SIZE } from '$lib/utils/editor-text-keys';

export function createReadOnlyTextPreview(parent: HTMLElement, path: string, content: string): () => void {
  const syntaxTheme = new Compartment();
  const extensions = [
    EditorState.readOnly.of(true),
    EditorView.editable.of(false),
    highlightActiveLineGutter(),
    foldGutter(),
    lineNumbers(),
    highlightSpecialChars(),
    highlightActiveLine(),
    EditorView.lineWrapping,
    EditorState.tabSize.of(EDITOR_TAB_SIZE),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    syntaxTheme.of(getSyntaxTheme()),
    gruvboxTheme,
    editorFontTheme,
  ];
  const language = getLanguage(path);
  if (language) extensions.push(language);

  const view = new EditorView({
    state: EditorState.create({ doc: content, extensions }),
    parent,
  });
  const themeObserver = new MutationObserver(() => {
    view.dispatch({ effects: syntaxTheme.reconfigure(getSyntaxTheme()) });
  });
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });

  return () => {
    themeObserver.disconnect();
    view.destroy();
  };
}
