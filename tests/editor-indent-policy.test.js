// @ts-nocheck

import { EditorState } from '@codemirror/state';
import {
  EDITOR_TAB_SIZE,
  handleInsertModeEnter,
  handleInsertModeShiftTab,
  handleInsertModeTab,
} from '../src/lib/utils/editor-text-keys.ts';
import {
  getEditorIndentPolicy,
  getEditorIndentUnit,
} from '../src/lib/utils/editor-indent-policy.js';

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

class TestView {
  constructor(doc, selection) {
    this.state = EditorState.create({ doc, selection });
  }

  dispatch(spec) {
    const transaction = this.state.update(spec);
    this.state = transaction.state;
  }
}

function docAfterTab(doc, selection, indentPolicy) {
  const view = new TestView(doc, selection);
  handleInsertModeTab(view, { indentPolicy });
  return view.state.doc.toString();
}

function docAfterShiftTab(doc, selection, indentPolicy) {
  const view = new TestView(doc, selection);
  handleInsertModeShiftTab(view, { indentPolicy });
  return view.state.doc.toString();
}

function docAfterEnter(doc, selection, indentPolicy) {
  const view = new TestView(doc, selection);
  handleInsertModeEnter(view, { indentPolicy });
  return view.state.doc.toString();
}

assertEqual(EDITOR_TAB_SIZE, 4, 'keeps editor tab size at four columns');

assertEqual(getEditorIndentPolicy('D:/repo/Makefile'), 'hard-tab-indent', 'detects Makefile');
assertEqual(getEditorIndentPolicy('D:/repo/makefile'), 'hard-tab-indent', 'detects lowercase makefile');
assertEqual(getEditorIndentPolicy('D:/repo/GNUmakefile'), 'hard-tab-indent', 'detects GNUmakefile');
assertEqual(getEditorIndentPolicy('D:/repo/BSDmakefile'), 'hard-tab-indent', 'detects BSDmakefile');
assertEqual(getEditorIndentPolicy('D:/repo/rules.mk'), 'hard-tab-indent', 'detects mk files');
assertEqual(getEditorIndentPolicy('D:/repo/rules.mak'), 'hard-tab-indent', 'detects mak files');
assertEqual(getEditorIndentPolicy('D:/repo/data.tsv'), 'tab-delimited', 'detects tsv files');
assertEqual(getEditorIndentPolicy('D:/repo/data.tab'), 'tab-delimited', 'detects tab files');
assertEqual(getEditorIndentPolicy('D:/repo/App.svelte'), 'space-indent', 'leaves ordinary files space-indented');
assertEqual(getEditorIndentUnit('hard-tab-indent'), '\t', 'uses hard tab indent unit for Makefile-style files');
assertEqual(getEditorIndentUnit('space-indent'), '    ', 'uses spaces for ordinary files');

assertEqual(
  docAfterTab('a', { anchor: 1 }, 'space-indent'),
  'a   ',
  'ordinary files insert spaces to the next tab stop',
);
assertEqual(
  docAfterTab('abcd', { anchor: 4 }, 'space-indent'),
  'abcd    ',
  'ordinary files insert four spaces when already on a tab stop',
);
assertEqual(
  docAfterTab('one\ntwo', { anchor: 0, head: 7 }, 'space-indent'),
  '    one\n    two',
  'ordinary selected lines indent with space tab stops',
);

assertEqual(
  docAfterTab('echo build', { anchor: 0 }, 'hard-tab-indent'),
  '\techo build',
  'Makefile single-cursor Tab inserts a hard tab',
);
assertEqual(
  docAfterTab('echo build\necho test', { anchor: 0, head: 20 }, 'hard-tab-indent'),
  '\techo build\n\techo test',
  'Makefile selected lines indent with hard tabs',
);
assertEqual(
  docAfterShiftTab('\techo build\n    echo test', { anchor: 0, head: 21 }, 'hard-tab-indent'),
  'echo build\necho test',
  'Makefile Shift+Tab removes hard tabs and falls back to space dedent',
);
assertEqual(
  docAfterEnter('\techo build', { anchor: 11 }, 'hard-tab-indent'),
  '\techo build\n\t',
  'Makefile Enter preserves leading hard tab indentation',
);

assertEqual(
  docAfterTab('name', { anchor: 4 }, 'tab-delimited'),
  'name\t',
  'TSV files insert a literal field separator',
);
