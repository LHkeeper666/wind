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
// @ts-ignore Node's type stripping requires the source extension.
} from '../src/lib/utils/editor-indent-policy.ts';

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

const separated = '7. first\n8. second\n\n# Later\n\n1. third\n2. fourth';
assertEqual(
  docAfterEnter(separated, { anchor: separated.length }, 'space-indent'),
  '7. first\n8. second\n\n# Later\n\n1. third\n2. fourth\n3. ',
  'Enter continues only the later list',
);
assertEqual(
  docAfterTab(separated, { anchor: separated.indexOf('2. fourth') }, 'space-indent'),
  '7. first\n8. second\n\n# Later\n\n1. third\n    1. fourth',
  'Tab leaves the earlier section unchanged',
);
const nested = '7. first\n8. second\n\nBody text\n\n1. third\n    1. child\n    2. child two\n2. fourth';
assertEqual(
  docAfterShiftTab(nested, { anchor: nested.indexOf('2. child two') }, 'space-indent'),
  '7. first\n8. second\n\nBody text\n\n1. third\n    1. child\n2. child two\n3. fourth',
  'Shift+Tab renumbers only source and target containers',
);
const fenced = '1. first\n```\n9. code\n```\n\n1. second\n2. third';
assertEqual(
  docAfterTab(fenced, { anchor: fenced.indexOf('2. third') }, 'space-indent'),
  '1. first\n```\n9. code\n```\n\n1. second\n    1. third',
  'fenced number lines are not list items',
);
const blankList = '1. first\n\n2. second\n3. third';
assertEqual(
  docAfterTab(blankList, { anchor: blankList.indexOf('2. second') }, 'space-indent'),
  '1. first\n\n    1. second\n2. third',
  'a single blank line keeps the source list connected',
);
const mixed = '1. first\n- bullet\n2. second\n3. third';
assertEqual(
  docAfterTab(mixed, { anchor: mixed.indexOf('3. third') }, 'space-indent'),
  '1. first\n- bullet\n1. second\n    1. third',
  'a same-level list type switch separates ordered containers',
);
const nine = Array.from({ length: 10 }, (_, index) => `${index + 1}. item ${index + 1}`).join('\n');
assertEqual(
  docAfterTab(nine, { anchor: nine.indexOf('9. item 9') }, 'space-indent'),
  Array.from({ length: 10 }, (_, index) => index === 8 ? '    1. item 9' : `${index === 9 ? 9 : index + 1}. item ${index + 1}`).join('\n'),
  'numbering remains correct across the 9 to 10 marker width',
);
const multiSection = '7. first\n8. second\n\n# Heading\n\n4. third\n5. fourth';
assertEqual(
  docAfterTab(multiSection, { anchor: 0, head: multiSection.length }, 'space-indent'),
  '    1. first\n    2. second\n\n# Heading\n\n    1. third\n    2. fourth',
  'cross-section selection renumbers each independent container',
);
const emptyLater = '7. first\n8. second\n\n# Heading\n\n1. third\n2. ';
assertEqual(
  docAfterEnter(emptyLater, { anchor: emptyLater.length }, 'space-indent'),
  '7. first\n8. second\n\n# Heading\n\n1. third\n',
  'exiting an empty later item leaves the earlier section unchanged',
);
