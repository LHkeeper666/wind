import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import { EditorView } from '@codemirror/view';
import { Compartment } from '@codemirror/state';

// Gruvbox Dark syntax highlighting
const gruvboxDarkHighlight = HighlightStyle.define([
  { tag: tags.keyword, color: '#fb4934' },
  { tag: [tags.typeName, tags.className, tags.namespace], color: '#fabd2f' },
  { tag: [tags.function(tags.variableName), tags.labelName], color: '#83a598' },
  { tag: [tags.propertyName, tags.attributeName], color: '#8ec07c' },
  { tag: tags.string, color: '#b8bb26' },
  { tag: [tags.number, tags.bool, tags.self, tags.null], color: '#fe8019' },
  { tag: tags.comment, color: '#928374', fontStyle: 'italic' },
  { tag: tags.regexp, color: '#b16286' },
  { tag: [tags.operator, tags.punctuation, tags.bracket], color: '#a89984' },
  { tag: [tags.meta, tags.modifier], color: '#d3869b' },
  { tag: tags.strong, fontWeight: 'bold' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.link, color: '#83a598', textDecoration: 'underline' },
  { tag: tags.heading, color: '#fabd2f', fontWeight: 'bold' },
]);

// Gruvbox Light syntax highlighting
const gruvboxLightHighlight = HighlightStyle.define([
  { tag: tags.keyword, color: '#9d0006' },
  { tag: [tags.typeName, tags.className, tags.namespace], color: '#b57614' },
  { tag: [tags.function(tags.variableName), tags.labelName], color: '#076678' },
  { tag: [tags.propertyName, tags.attributeName], color: '#427b58' },
  { tag: tags.string, color: '#79740e' },
  { tag: [tags.number, tags.bool, tags.self, tags.null], color: '#af3a03' },
  { tag: tags.comment, color: '#928374', fontStyle: 'italic' },
  { tag: tags.regexp, color: '#8f3f71' },
  { tag: [tags.operator, tags.punctuation, tags.bracket], color: '#665c54' },
  { tag: [tags.meta, tags.modifier], color: '#8f3f71' },
  { tag: tags.strong, fontWeight: 'bold' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.link, color: '#076678', textDecoration: 'underline' },
  { tag: tags.heading, color: '#b57614', fontWeight: 'bold' },
]);

export const gruvboxDark = syntaxHighlighting(gruvboxDarkHighlight);
export const gruvboxLight = syntaxHighlighting(gruvboxLightHighlight);

export function getSyntaxTheme(): ReturnType<typeof syntaxHighlighting> {
  const isLight = document.documentElement.getAttribute('data-theme') === 'light';
  return isLight ? gruvboxLight : gruvboxDark;
}

// Compartment for dynamic theme switching (reconfigured on data-theme change)
export function createThemeCompartment(): Compartment {
  return new Compartment();
}

export function getCurrentTheme(): ReturnType<typeof syntaxHighlighting> {
  return getSyntaxTheme();
}

// Chrome theme using CSS variables — auto-adapts to dark/light
export const gruvboxTheme = EditorView.theme({
  '&': { backgroundColor: 'var(--bg-primary)' },
  '.cm-content': { caretColor: 'var(--text-primary)' },
  '.cm-gutters': { backgroundColor: 'var(--bg-primary)', color: 'var(--text-muted)', border: 'none' },
  '.cm-activeLineGutter': { backgroundColor: 'var(--bg-secondary)', color: 'var(--text-primary)' },
  '.cm-activeLine': { backgroundColor: 'var(--bg-secondary)' },
  '&.vim-visual .cm-activeLine': { backgroundColor: 'transparent' },
  '&.vim-visual .cm-activeLineGutter': { backgroundColor: 'transparent' },
  '.cm-cursor': { borderLeftColor: 'var(--text-primary)' },
  '.cm-selectionBackground': { backgroundColor: 'rgba(var(--bg-active-rgb), 0.5)' },
  '&.cm-focused .cm-selectionBackground': { backgroundColor: 'var(--bg-active)' },
  '.cm-matchingBracket': { backgroundColor: 'var(--bg-tertiary)', outline: '1px solid var(--text-secondary)' },
  '.cm-lineNumbers .cm-gutterElement': { color: 'var(--text-muted)' },
});
