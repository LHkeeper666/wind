import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import { EditorView } from '@codemirror/view';

// Gruvbox dark syntax highlighting
export const gruvboxHighlightStyle = HighlightStyle.define([
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

// Gruvbox dark syntax extension
export const gruvboxDark = syntaxHighlighting(gruvboxHighlightStyle);

// Gruvbox dark chrome (editor UI)
export const gruvboxTheme = EditorView.theme({
  '&': { backgroundColor: '#282828' },
  '.cm-content': { caretColor: '#ebdbb2' },
  '.cm-gutters': { backgroundColor: '#282828', color: '#7c6f64', border: 'none' },
  '.cm-activeLineGutter': { backgroundColor: '#3c3836', color: '#ebdbb2' },
  '.cm-activeLine': { backgroundColor: '#3c383640', borderLeft: '2px solid #fabd2f' },
  '.cm-cursor': { borderLeftColor: '#ebdbb2' },
  '.cm-selectionBackground': { backgroundColor: '#665c5480' },
  '&.cm-focused .cm-selectionBackground': { backgroundColor: '#665c54' },
  '.cm-matchingBracket': { backgroundColor: '#504945', outline: '1px solid #a89984' },
  '.cm-lineNumbers .cm-gutterElement': { color: '#7c6f64' },
});
