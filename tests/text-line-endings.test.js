// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { EditorState } from '@codemirror/state';
import { detectLineEnding, normalizeLineEndings, restoreLineEndings } from '../src/lib/utils/text-line-endings.ts';

for (const [name, original, expected] of [
  ['LF', 'one\ntwo\n', 'one!\ntwo\n'],
  ['CRLF', 'one\r\ntwo\r\n', 'one!\r\ntwo\r\n'],
  ['CR', 'one\rtwo\r', 'one!\rtwo\r'],
]) {
  test(`${name}: opening, editing, undoing and saving preserves line endings`, () => {
    const lineEnding = detectLineEnding(original);
    const baseline = normalizeLineEndings(original);
    const state = EditorState.create({ doc: baseline });
    assert.equal(state.doc.toString(), baseline);
    assert.equal(state.doc.toString() !== baseline, false);

    const edited = state.update({ changes: { from: 3, insert: '!' } }).state;
    assert.equal(edited.doc.toString() !== baseline, true);
    const undone = edited.update({ changes: { from: 3, to: 4, insert: '' } }).state;
    assert.equal(undone.doc.toString() !== baseline, false);
    assert.equal(restoreLineEndings(edited.doc.toString(), lineEnding), expected);
    assert.equal(normalizeLineEndings(restoreLineEndings(edited.doc.toString(), lineEnding)), edited.doc.toString());
  });
}

test('cached normalized text retains its original line ending metadata', () => {
  const source = 'a\r\nb\r\n';
  const cached = { content: normalizeLineEndings(source), savedContent: normalizeLineEndings(source), lineEnding: detectLineEnding(source) };
  assert.equal(cached.content !== cached.savedContent, false);
  assert.equal(restoreLineEndings(cached.content, cached.lineEnding), source);
});

test('mixed line endings use the first delimiter when edited', () => {
  const source = 'a\r\nb\nc\r';
  assert.equal(detectLineEnding(source), 'crlf');
  assert.equal(restoreLineEndings(normalizeLineEndings(source), 'crlf'), 'a\r\nb\r\nc\r\n');
});
