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
import {
  isHardTabIndentPolicy,
  isLiteralTabInsertionPolicy,
  type EditorIndentPolicy,
// @ts-ignore Node's type stripping requires the source extension in tests.
} from './editor-indent-policy.ts';

export const EDITOR_TAB_SIZE = 4;
export type EditorTextKeyOptions = {
  tabSize?: number;
  indentPolicy?: EditorIndentPolicy;
};
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

interface MarkdownListPrefix {
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
  sectionId: number;
  subtreeEndIndex: number;
}

interface MarkdownListTransformResult {
  handled: boolean;
  changes: TextChange[];
}

function getLeadingWhitespace(text: string): string {
  return text.match(/^[ \t]*/)?.[0] ?? '';
}

function normalizeTextKeyOptions(options: number | EditorTextKeyOptions = {}): Required<EditorTextKeyOptions> {
  if (typeof options === 'number') {
    return { tabSize: options, indentPolicy: 'space-indent' };
  }
  return {
    tabSize: options.tabSize ?? EDITOR_TAB_SIZE,
    indentPolicy: options.indentPolicy ?? 'space-indent',
  };
}

function spacesToNextTabStop(column: number, tabSize: number = EDITOR_TAB_SIZE): number {
  const remainder = column % tabSize;
  return remainder === 0 ? tabSize : tabSize - remainder;
}

function parseMarkdownListPrefix(text: string): MarkdownListPrefix | null {
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

function buildMarkdownContinuationPrefix(text: string): string | null {
  const marker = parseMarkdownListPrefix(text);
  if (!marker) return null;
  const nextMarker = marker.ordered && marker.orderedNumber !== null
    ? `${marker.orderedNumber + 1}.`
    : marker.marker;
  return `${marker.indent}${nextMarker}${marker.suffix}`;
}

function buildIndentedMarkdownListPrefix(prefix: MarkdownListPrefix, orderedIndex: number = 1, tabSize: number = EDITOR_TAB_SIZE): string {
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

function buildMarkdownListItems(lines: string[], tabSize: number): MarkdownListItem[] {
  const items: MarkdownListItem[] = [];
  const stack: MarkdownListItem[] = [];
  let sectionId = 0;
  let fence: { marker: string; length: number } | null = null;
  let boundary = -1;

  const closeSection = (lineIndex: number): void => {
    for (const item of stack) item.subtreeEndIndex = Math.min(item.subtreeEndIndex, lineIndex - 1);
    stack.length = 0;
    boundary = lineIndex;
  };

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
    const text = lines[lineIndex];
    const fenceMatch = text.match(/^[ \t]*(`{3,}|~{3,})/);
    if (fence) {
      if (fenceMatch && fenceMatch[1][0] === fence.marker && fenceMatch[1].length >= fence.length) fence = null;
      continue;
    }
    if (fenceMatch) {
      // An indented fence can belong to a list item; its contents are still never list markers.
      if (!stack.length || leadingColumns(text, tabSize) < stack[stack.length - 1].contentColumns) closeSection(lineIndex);
      fence = { marker: fenceMatch[1][0], length: fenceMatch[1].length };
      continue;
    }
    const prefix = parseMarkdownListPrefix(text);
    if (!prefix) {
      if (text.trim() && stack.length && (
        /^\s{0,3}(?:#{1,6}\s|(?:---+|___+|\*\*\*+)\s*$|>)/.test(text)
        || leadingColumns(text, tabSize) < stack[stack.length - 1].contentColumns
      )) closeSection(lineIndex);
      continue;
    }

    const indentColumns = countColumn(prefix.indent, tabSize);
    let previousSibling: MarkdownListItem | undefined;
    while (stack.length > 0 && stack[stack.length - 1].indentColumns >= indentColumns) {
      const popped = stack.pop()!;
      popped.subtreeEndIndex = lineIndex - 1;
      if (popped.indentColumns === indentColumns) previousSibling = popped;
    }
    if (previousSibling && previousSibling.ordered !== prefix.ordered) sectionId++;
    if (!stack.length) {
      const previous = items[items.length - 1];
      if (!previous || boundary > previous.lineIndex) sectionId++;
    }

    const item: MarkdownListItem = {
      lineIndex,
      indentColumns,
      contentColumns: listItemContentColumns(lines[lineIndex], prefix, tabSize),
      ordered: prefix.ordered,
      parentLineIndex: stack.length > 0 ? stack[stack.length - 1].lineIndex : null,
      sectionId,
      subtreeEndIndex: lines.length - 1,
    };
    items.push(item);
    stack.push(item);
  }

  closeSection(lines.length);

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
    const parentRoot = roots.find(root => root.sectionId === item.sectionId && root.lineIndex < item.lineIndex && root.subtreeEndIndex >= item.lineIndex);
    if (!parentRoot) roots.push(item);
  }

  return roots;
}

function applyMarkdownListTreeMovement(
  lines: string[],
  roots: MarkdownListItem[],
  direction: 'indent' | 'dedent',
  tabSize: number,
  onlyRootLine: boolean = false,
  selectionEndLine: number = Infinity,
): void {
  for (const root of roots) {
    const rootTargetColumns = direction === 'indent'
      ? root.indentColumns + tabSize
      : Math.floor(Math.max(0, root.indentColumns - 1) / tabSize) * tabSize;
    const delta = rootTargetColumns - root.indentColumns;
    if (delta === 0) continue;

    const endLine = onlyRootLine ? root.lineIndex : Math.min(root.subtreeEndIndex, selectionEndLine);
    for (let lineIndex = root.lineIndex; lineIndex <= endLine; lineIndex++) {
      if (!lines[lineIndex].trim()) continue;
      lines[lineIndex] = replaceLeadingColumns(lines[lineIndex], leadingColumns(lines[lineIndex], tabSize) + delta);
    }
  }
}

function orderedContainerKey(item: MarkdownListItem): string {
  return `${item.sectionId}:${item.parentLineIndex ?? 'root'}:${item.indentColumns}`;
}

// Shared entry point for insert-mode edits and future normal-mode indentation.
export function renumberOrderedListContainers(lines: string[], tabSize: number, affected?: ReadonlySet<string>): void {
  const items = buildMarkdownListItems(lines, tabSize);
  const lastNumber = new Map<string, number>();

  for (const item of items) {
    if (!item.ordered) continue;

    const containerKey = orderedContainerKey(item);
    const nextNumber = (lastNumber.get(containerKey) ?? 0) + 1;
    lastNumber.set(containerKey, nextNumber);
    if (affected && !affected.has(containerKey)) continue;
    const prefix = parseMarkdownListPrefix(lines[item.lineIndex]);
    if (prefix && prefix.orderedNumber !== nextNumber) {
      lines[item.lineIndex] = `${prefix.indent}${nextNumber}.${prefix.suffix}${lines[item.lineIndex].slice(prefix.prefixLength)}`;
    }
  }
}

function affectedOrderedContainers(before: MarkdownListItem[], after: MarkdownListItem[], movedLines: ReadonlySet<number>): Set<string> {
  const beforeKeys = new Set(before.filter(item => movedLines.has(item.lineIndex) && item.ordered).map(orderedContainerKey));
  const affectedLines = new Set(movedLines);
  for (const item of before) if (item.ordered && beforeKeys.has(orderedContainerKey(item))) affectedLines.add(item.lineIndex);
  const afterKeys = new Set<string>();
  for (const item of after) if (item.ordered && affectedLines.has(item.lineIndex)) afterKeys.add(orderedContainerKey(item));
  return afterKeys;
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

  const movedLines = new Set<number>();
  for (const root of roots) {
    const end = from === to ? root.lineIndex : Math.min(root.subtreeEndIndex, range.to);
    for (let lineIndex = root.lineIndex; lineIndex <= end; lineIndex++) movedLines.add(lineIndex);
  }
  applyMarkdownListTreeMovement(lines, roots, direction, tabSize, from === to, range.to);
  const afterItems = buildMarkdownListItems(lines, tabSize);
  renumberOrderedListContainers(lines, tabSize, affectedOrderedContainers(items, afterItems, movedLines));

  return { handled: true, changes: lineTextChanges(state, lines) };
}

function indentChangesForLines(
  state: EditorView['state'],
  from: number,
  to: number,
  tabSize: number = EDITOR_TAB_SIZE,
  indentPolicy: EditorIndentPolicy = 'space-indent',
): TextChange[] {
  const range = selectedLineRange(state, from, to);
  const changes: TextChange[] = [];
  for (let i = range.from + 1; i <= range.to + 1; i++) {
    const line = state.doc.line(i);
    if (isHardTabIndentPolicy(indentPolicy)) {
      changes.push({ from: line.from, insert: '\t' });
      continue;
    }
    const leadingWhitespace = getLeadingWhitespace(line.text);
    const leadingColumns = countColumn(leadingWhitespace, tabSize);
    const insertCount = spacesToNextTabStop(leadingColumns, tabSize);
    changes.push({ from: line.from, insert: ' '.repeat(insertCount) });
  }
  return changes;
}

function dedentChangesForLines(
  state: EditorView['state'],
  from: number,
  to: number,
  tabSize: number = EDITOR_TAB_SIZE,
  indentPolicy: EditorIndentPolicy = 'space-indent',
): TextChange[] {
  const range = selectedLineRange(state, from, to);
  const changes: TextChange[] = [];
  for (let i = range.from + 1; i <= range.to + 1; i++) {
    const line = state.doc.line(i);
    if (isHardTabIndentPolicy(indentPolicy) && line.text.startsWith('\t')) {
      changes.push({ from: line.from, to: line.from + 1 });
      continue;
    }
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

function insertNewlineAndPreserveLeadingWhitespace(view: EditorView): boolean {
  const { state } = view;
  const { main } = state.selection;
  const line = state.doc.lineAt(main.head);
  const leadingWhitespace = getLeadingWhitespace(line.text);
  const from = main.from;
  const insert = `\n${leadingWhitespace}`;
  view.dispatch({
    changes: { from, to: main.to, insert },
    selection: { anchor: from + insert.length },
  });
  return true;
}

export function handleInsertModeTab(view: EditorView, options: number | EditorTextKeyOptions = {}): boolean {
  if (acceptCompletion(view)) return true;

  const { tabSize, indentPolicy } = normalizeTextKeyOptions(options);
  const { state } = view;
  const { main } = state.selection;
  if (indentPolicy === 'space-indent') {
    const listTreeResult = markdownListTreeChangesForLines(state, main.from, main.to, 'indent', tabSize);
    if (listTreeResult.handled) {
      if (listTreeResult.changes.length > 0) {
        view.dispatch({ changes: listTreeResult.changes });
      }
      return true;
    }
  }

  if (main.empty) {
    if (isLiteralTabInsertionPolicy(indentPolicy)) {
      view.dispatch({
        changes: { from: main.head, insert: '\t' },
        selection: { anchor: main.head + 1 },
      });
      return true;
    }

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

  if (indentPolicy === 'tab-delimited') {
    view.dispatch({
      changes: { from: main.from, to: main.to, insert: '\t' },
      selection: { anchor: main.from + 1 },
    });
    return true;
  }

  const changes = indentChangesForLines(state, main.from, main.to, tabSize, indentPolicy);
  if (changes.length === 0) return false;
  view.dispatch({ changes });
  return true;
}

export function handleInsertModeShiftTab(view: EditorView, options: number | EditorTextKeyOptions = {}): boolean {
  const { tabSize, indentPolicy } = normalizeTextKeyOptions(options);
  const { state } = view;
  const { main } = state.selection;
  const from = main.empty ? main.head : main.from;
  const to = main.empty ? main.head : main.to;
  if (indentPolicy === 'space-indent') {
    const listTreeResult = markdownListTreeChangesForLines(state, from, to, 'dedent', tabSize);
    if (listTreeResult.handled) {
      if (listTreeResult.changes.length > 0) {
        view.dispatch({ changes: listTreeResult.changes });
      }
      return true;
    }
  }

  const changes = dedentChangesForLines(state, from, to, tabSize, indentPolicy);
  if (changes.length === 0) return false;
  view.dispatch({ changes });
  return true;
}

export function handleInsertModeEnter(view: EditorView, options: number | EditorTextKeyOptions = {}): boolean {
  const { indentPolicy } = normalizeTextKeyOptions(options);
  const { state } = view;
  const { main } = state.selection;
  const line = state.doc.lineAt(main.head);
  if (isHardTabIndentPolicy(indentPolicy)) {
    return insertNewlineAndPreserveLeadingWhitespace(view);
  }

  const prefix = parseMarkdownListPrefix(line.text);

  if (!prefix) {
    return insertNewlineAndIndent(view);
  }
  const { tabSize } = normalizeTextKeyOptions(options);
  const beforeItems = buildMarkdownListItems(documentLines(state), tabSize);
  const sourceItem = beforeItems.find(item => item.lineIndex === line.number - 1);
  if (!sourceItem) return insertNewlineAndIndent(view);

  const renumberLocal = (lineShift: number): void => {
    const afterState = view.state;
    const afterLines = documentLines(afterState);
    const afterItems = buildMarkdownListItems(afterLines, tabSize);
    const sourceKey = orderedContainerKey(sourceItem);
    const affectedLines = new Set(beforeItems.filter(item => item.ordered && orderedContainerKey(item) === sourceKey)
      .map(item => item.lineIndex + (item.lineIndex > sourceItem.lineIndex ? lineShift : 0)));
    affectedLines.add(sourceItem.lineIndex + (lineShift > 0 ? 1 : 0));
    const keys = new Set(afterItems.filter(item => item.ordered && affectedLines.has(item.lineIndex)).map(orderedContainerKey));
    renumberOrderedListContainers(afterLines, tabSize, keys);
    const changes = lineTextChanges(afterState, afterLines);
    if (changes.length) view.dispatch({ changes });
  };

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
    // Renumber subsequent ordered list items after removing the empty list item
    renumberLocal(0);
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
  // Renumber subsequent ordered list items after inserting the continuation line
  renumberLocal(1);
  return true;
}
