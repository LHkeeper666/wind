import {
  acceptCompletion,
  closeCompletion,
  moveCompletionSelection,
  startCompletion,
} from '@codemirror/autocomplete';
import { insertNewlineAndIndent } from '@codemirror/commands';
import { countColumn, findColumn } from '@codemirror/state';
import type { KeyBinding } from '@codemirror/view';
import type { EditorView } from 'codemirror';

export const EDITOR_TAB_SIZE = 4;
export const editorAutocompleteKeymap: KeyBinding[] = [
  { key: 'Ctrl-Space', run: startCompletion },
  { mac: 'Alt-`', run: startCompletion },
  { mac: 'Alt-i', run: startCompletion },
  { key: 'Escape', run: closeCompletion },
  { key: 'ArrowDown', run: moveCompletionSelection(true) },
  { key: 'ArrowUp', run: moveCompletionSelection(false) },
  { key: 'PageDown', run: moveCompletionSelection(true, 'page') },
  { key: 'PageUp', run: moveCompletionSelection(false, 'page') },
];

export interface MarkdownListPrefix {
  prefixLength: number;
  indent: string;
  marker: string;
  ordered: boolean;
  orderedNumber: number | null;
  suffix: string;
}

type TextChange = { from: number; to?: number; insert?: string };

interface MarkdownListItem {
  lineIndex: number;
  indentColumns: number;
  contentColumns: number;
  ordered: boolean;
  parentLineIndex: number | null;
  subtreeEndIndex: number;
}

interface MarkdownListTransformResult {
  handled: boolean;
  changes: TextChange[];
}

function getLeadingWhitespace(text: string): string {
  return text.match(/^[ \t]*/)?.[0] ?? '';
}

export function spacesToNextTabStop(column: number, tabSize: number = EDITOR_TAB_SIZE): number {
  const remainder = column % tabSize;
  return remainder === 0 ? tabSize : tabSize - remainder;
}

export function parseMarkdownListPrefix(text: string): MarkdownListPrefix | null {
  const match = text.match(/^([ \t]*)(?:([-*+])|(\d+)\.)(\s(?:\[[ x]\]\s)?)/);
  if (!match || !match[0].length) return null;
  const suffix = match[4] ?? ' ';
  return {
    prefixLength: match[0].length,
    indent: match[1] ?? '',
    marker: match[2] ?? `${match[3]}.`,
    ordered: !!match[3],
    orderedNumber: match[3] ? Number(match[3]) : null,
    suffix,
  };
}

export function buildMarkdownContinuationPrefix(text: string): string | null {
  const marker = parseMarkdownListPrefix(text);
  if (!marker) return null;
  const nextMarker = marker.ordered && marker.orderedNumber !== null
    ? `${marker.orderedNumber + 1}.`
    : marker.marker;
  return `${marker.indent}${nextMarker}${marker.suffix}`;
}

export function buildIndentedMarkdownListPrefix(prefix: MarkdownListPrefix, orderedIndex: number = 1, tabSize: number = EDITOR_TAB_SIZE): string {
  const marker = prefix.ordered ? `${orderedIndex}.` : prefix.marker;
  return `${' '.repeat(tabSize)}${prefix.indent}${marker}${prefix.suffix}`;
}

function selectedLineRange(state: EditorView['state'], from: number, to: number): { from: number; to: number } {
  const startLine = state.doc.lineAt(from).number - 1;
  const effectiveTo = to > from && to <= state.doc.length && state.doc.lineAt(to).from === to
    ? to - 1
    : to;
  const endLine = state.doc.lineAt(Math.max(from, effectiveTo)).number - 1;
  return { from: startLine, to: endLine };
}

function leadingColumns(text: string, tabSize: number): number {
  return countColumn(getLeadingWhitespace(text), tabSize);
}

function replaceLeadingColumns(text: string, columns: number): string {
  return `${' '.repeat(Math.max(0, columns))}${text.slice(getLeadingWhitespace(text).length)}`;
}

function documentLines(state: EditorView['state']): string[] {
  const lines: string[] = [];
  for (let i = 1; i <= state.doc.lines; i++) {
    lines.push(state.doc.line(i).text);
  }
  return lines;
}

function listItemContentColumns(text: string, prefix: MarkdownListPrefix, tabSize: number): number {
  return countColumn(text.slice(0, prefix.prefixLength), tabSize);
}

function findMarkdownListSubtreeEnd(
  lines: string[],
  item: MarkdownListItem,
  nextAtSameOrHigherLevel: MarkdownListItem | undefined,
  tabSize: number,
): number {
  const naturalEnd = nextAtSameOrHigherLevel ? nextAtSameOrHigherLevel.lineIndex - 1 : lines.length - 1;
  let afterBlankLine = false;

  for (let lineIndex = item.lineIndex + 1; lineIndex <= naturalEnd; lineIndex++) {
    const text = lines[lineIndex];
    if (!text.trim()) {
      afterBlankLine = true;
      continue;
    }

    const prefix = parseMarkdownListPrefix(text);
    if (prefix) {
      afterBlankLine = false;
      continue;
    }

    if (!afterBlankLine) continue;
    if (leadingColumns(text, tabSize) >= item.contentColumns) continue;

    return lineIndex - 1;
  }

  return naturalEnd;
}

function buildMarkdownListItems(lines: string[], tabSize: number): MarkdownListItem[] {
  const items: MarkdownListItem[] = [];
  const stack: MarkdownListItem[] = [];

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
    const prefix = parseMarkdownListPrefix(lines[lineIndex]);
    if (!prefix) continue;

    const indentColumns = countColumn(prefix.indent, tabSize);
    while (stack.length > 0 && stack[stack.length - 1].indentColumns >= indentColumns) {
      stack.pop();
    }

    const item: MarkdownListItem = {
      lineIndex,
      indentColumns,
      contentColumns: listItemContentColumns(lines[lineIndex], prefix, tabSize),
      ordered: prefix.ordered,
      parentLineIndex: stack.length > 0 ? stack[stack.length - 1].lineIndex : null,
      subtreeEndIndex: lines.length - 1,
    };
    items.push(item);
    stack.push(item);
  }

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    const nextAtSameOrHigherLevel = items.slice(i + 1).find(next => next.indentColumns <= item.indentColumns);
    item.subtreeEndIndex = findMarkdownListSubtreeEnd(lines, item, nextAtSameOrHigherLevel, tabSize);
  }

  return items;
}

function selectedMarkdownListRoots(
  items: MarkdownListItem[],
  lineFrom: number,
  lineTo: number,
  empty: boolean,
): MarkdownListItem[] {
  const selected = empty
    ? items.filter(item => item.lineIndex === lineFrom)
    : items.filter(item => item.lineIndex >= lineFrom && item.lineIndex <= lineTo);
  const roots: MarkdownListItem[] = [];

  for (const item of selected) {
    const parentRoot = roots.find(root => root.lineIndex < item.lineIndex && root.subtreeEndIndex >= item.lineIndex);
    if (!parentRoot) roots.push(item);
  }

  return roots;
}

function applyMarkdownListTreeMovement(
  lines: string[],
  roots: MarkdownListItem[],
  direction: 'indent' | 'dedent',
  tabSize: number,
): void {
  for (const root of roots) {
    const rootTargetColumns = direction === 'indent'
      ? root.indentColumns + tabSize
      : Math.floor(Math.max(0, root.indentColumns - 1) / tabSize) * tabSize;
    const delta = rootTargetColumns - root.indentColumns;
    if (delta === 0) continue;

    for (let lineIndex = root.lineIndex; lineIndex <= root.subtreeEndIndex; lineIndex++) {
      if (!lines[lineIndex].trim()) continue;
      lines[lineIndex] = replaceLeadingColumns(lines[lineIndex], leadingColumns(lines[lineIndex], tabSize) + delta);
    }
  }
}

function hasOrderedContainerBarrier(
  lines: string[],
  fromLineIndex: number,
  toLineIndex: number,
  indentColumns: number,
  tabSize: number,
): boolean {
  for (let lineIndex = fromLineIndex; lineIndex <= toLineIndex; lineIndex++) {
    const text = lines[lineIndex];
    if (!text.trim()) continue;

    const prefix = parseMarkdownListPrefix(text);
    if (prefix) {
      if (countColumn(prefix.indent, tabSize) <= indentColumns) return true;
      continue;
    }

    if (leadingColumns(text, tabSize) <= indentColumns) return true;
  }

  return false;
}

function renumberOrderedListContainers(lines: string[], tabSize: number): void {
  const items = buildMarkdownListItems(lines, tabSize);
  const lastItemByContainer = new Map<string, { item: MarkdownListItem; number: number }>();

  for (const item of items) {
    if (!item.ordered) continue;

    const containerKey = `${item.parentLineIndex ?? 'root'}:${item.indentColumns}`;
    const previous = lastItemByContainer.get(containerKey);
    const shouldReset = !previous
      || hasOrderedContainerBarrier(lines, previous.item.lineIndex + 1, item.lineIndex - 1, item.indentColumns, tabSize);
    const nextNumber = shouldReset ? 1 : previous.number + 1;
    const prefix = parseMarkdownListPrefix(lines[item.lineIndex]);
    if (prefix && prefix.orderedNumber !== nextNumber) {
      lines[item.lineIndex] = `${prefix.indent}${nextNumber}.${prefix.suffix}${lines[item.lineIndex].slice(prefix.prefixLength)}`;
    }
    lastItemByContainer.set(containerKey, { item, number: nextNumber });
  }
}

function lineTextChanges(state: EditorView['state'], finalLines: string[]): TextChange[] {
  const changes: TextChange[] = [];

  for (let lineIndex = 0; lineIndex < finalLines.length; lineIndex++) {
    const line = state.doc.line(lineIndex + 1);
    const original = line.text;
    const updated = finalLines[lineIndex];
    if (original === updated) continue;

    let start = 0;
    while (start < original.length && start < updated.length && original[start] === updated[start]) {
      start++;
    }

    let originalEnd = original.length;
    let updatedEnd = updated.length;
    while (
      originalEnd > start
      && updatedEnd > start
      && original[originalEnd - 1] === updated[updatedEnd - 1]
    ) {
      originalEnd--;
      updatedEnd--;
    }

    changes.push({
      from: line.from + start,
      to: line.from + originalEnd,
      insert: updated.slice(start, updatedEnd),
    });
  }

  return changes;
}

function markdownListTreeChangesForLines(
  state: EditorView['state'],
  from: number,
  to: number,
  direction: 'indent' | 'dedent',
  tabSize: number = EDITOR_TAB_SIZE,
): MarkdownListTransformResult {
  const lines = documentLines(state);
  const items = buildMarkdownListItems(lines, tabSize);
  const range = selectedLineRange(state, from, to);
  const roots = selectedMarkdownListRoots(items, range.from, range.to, from === to);

  if (roots.length === 0) return { handled: false, changes: [] };

  applyMarkdownListTreeMovement(lines, roots, direction, tabSize);
  renumberOrderedListContainers(lines, tabSize);

  return { handled: true, changes: lineTextChanges(state, lines) };
}

function indentChangesForLines(state: EditorView['state'], from: number, to: number, tabSize: number = EDITOR_TAB_SIZE): TextChange[] {
  const range = selectedLineRange(state, from, to);
  const changes: TextChange[] = [];
  for (let i = range.from + 1; i <= range.to + 1; i++) {
    const line = state.doc.line(i);
    const leadingWhitespace = getLeadingWhitespace(line.text);
    const leadingColumns = countColumn(leadingWhitespace, tabSize);
    const insertCount = spacesToNextTabStop(leadingColumns, tabSize);
    changes.push({ from: line.from, insert: ' '.repeat(insertCount) });
  }
  return changes;
}

function dedentChangesForLines(state: EditorView['state'], from: number, to: number, tabSize: number = EDITOR_TAB_SIZE): TextChange[] {
  const range = selectedLineRange(state, from, to);
  const changes: TextChange[] = [];
  for (let i = range.from + 1; i <= range.to + 1; i++) {
    const line = state.doc.line(i);
    const leadingWhitespace = getLeadingWhitespace(line.text);
    const leadingColumns = countColumn(leadingWhitespace, tabSize);
    if (leadingColumns > 0) {
      const targetColumns = Math.floor((leadingColumns - 1) / tabSize) * tabSize;
      const currentChars = findColumn(line.text, leadingColumns, tabSize, true);
      const targetChars = findColumn(line.text, targetColumns, tabSize, true);
      if (currentChars > targetChars) {
        changes.push({ from: line.from + targetChars, to: line.from + currentChars });
      }
      continue;
    }
  }
  return changes;
}

export function handleInsertModeTab(view: EditorView, tabSize: number = EDITOR_TAB_SIZE): boolean {
  if (acceptCompletion(view)) return true;

  const { state } = view;
  const { main } = state.selection;
  const listTreeResult = markdownListTreeChangesForLines(state, main.from, main.to, 'indent', tabSize);
  if (listTreeResult.handled) {
    if (listTreeResult.changes.length > 0) {
      view.dispatch({ changes: listTreeResult.changes });
    }
    return true;
  }

  if (main.empty) {
    const line = state.doc.lineAt(main.head);
    const cursorColumn = countColumn(line.text, tabSize, main.head - line.from);
    const insertCount = spacesToNextTabStop(cursorColumn, tabSize);
    const insert = ' '.repeat(insertCount);
    view.dispatch({
      changes: { from: main.head, insert },
      selection: { anchor: main.head + insert.length },
    });
    return true;
  }

  const changes = indentChangesForLines(state, main.from, main.to, tabSize);
  if (changes.length === 0) return false;
  view.dispatch({ changes });
  return true;
}

export function handleInsertModeShiftTab(view: EditorView, tabSize: number = EDITOR_TAB_SIZE): boolean {
  const { state } = view;
  const { main } = state.selection;
  const from = main.empty ? main.head : main.from;
  const to = main.empty ? main.head : main.to;
  const listTreeResult = markdownListTreeChangesForLines(state, from, to, 'dedent', tabSize);
  if (listTreeResult.handled) {
    if (listTreeResult.changes.length > 0) {
      view.dispatch({ changes: listTreeResult.changes });
    }
    return true;
  }

  const changes = dedentChangesForLines(state, from, to, tabSize);
  if (changes.length === 0) return false;
  view.dispatch({ changes });
  return true;
}

export function handleInsertModeEnter(view: EditorView): boolean {
  const { state } = view;
  const { main } = state.selection;
  const line = state.doc.lineAt(main.head);
  const prefix = parseMarkdownListPrefix(line.text);

  if (!prefix) {
    return insertNewlineAndIndent(view);
  }

  const contentAfter = line.text.slice(prefix.prefixLength).trim();
  if (!contentAfter) {
    const indentMatch = line.text.match(/^[ \t]{1,4}/);
    if (indentMatch) {
      view.dispatch({
        changes: { from: line.from, to: line.from + indentMatch[0].length },
        selection: { anchor: line.from + prefix.prefixLength - indentMatch[0].length },
      });
    } else {
      view.dispatch({
        changes: { from: line.from, to: line.from + prefix.prefixLength },
        selection: { anchor: line.from },
      });
    }
    return true;
  }

  const continuation = buildMarkdownContinuationPrefix(line.text);
  if (!continuation) {
    return insertNewlineAndIndent(view);
  }

  view.dispatch({
    changes: { from: main.head, insert: `\n${continuation}` },
    selection: { anchor: main.head + 1 + continuation.length },
  });
  return true;
}
