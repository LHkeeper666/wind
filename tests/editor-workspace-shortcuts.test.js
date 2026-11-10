// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { isInsertTabShortcut, isInsertSlotShortcut } from '../src/lib/utils/editor-workspace-shortcuts.ts';

const key = (code, modifiers = {}) => ({
  code, altKey: false, ctrlKey: false, shiftKey: false, metaKey: false, ...modifiers,
});

test('insert mode accepts only the tab navigation and reorder Alt chords', () => {
  for (const code of ['KeyH', 'KeyL', 'KeyM', 'Comma', 'Period', 'Digit1', 'Digit9']) {
    assert.equal(isInsertTabShortcut(key(code, { altKey: true })), true, code);
  }
  for (const code of ['KeyN', 'KeyU', 'KeyR', 'KeyD', 'Digit0']) {
    assert.equal(isInsertTabShortcut(key(code, { altKey: true })), false, code);
  }
  assert.equal(isInsertTabShortcut(key('Comma', { altKey: true, shiftKey: true })), false);
  assert.equal(isInsertTabShortcut(key('Period', { altKey: true, ctrlKey: true })), false);
  assert.equal(isInsertTabShortcut(key('Comma')), false);
});

test('insert mode accepts only the slot navigation and reorder Ctrl chords', () => {
  for (const code of ['KeyH', 'KeyL', 'Comma', 'Period']) {
    assert.equal(isInsertSlotShortcut(key(code, { ctrlKey: true })), true, code);
  }
  assert.equal(isInsertSlotShortcut(key('KeyN', { ctrlKey: true })), false);
  assert.equal(isInsertSlotShortcut(key('Comma', { ctrlKey: true, shiftKey: true })), false);
  assert.equal(isInsertSlotShortcut(key('Period', { ctrlKey: true, altKey: true })), false);
  assert.equal(isInsertSlotShortcut(key('Period')), false);
});
