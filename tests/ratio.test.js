import {
  correctActiveColumn,
  nextLayoutRatios,
  nextVisibleColumn,
  parseRatioCommand,
  RATIO_PRESETS,
  visibleColumnCount,
// @ts-expect-error Node's type stripping resolves the .ts source directly.
} from '../src/lib/utils/ratio.ts';

/**
 * @param {unknown} actual
 * @param {unknown} expected
 * @param {string} message
 */
function assertEqual(actual, expected, message) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}: expected ${expectedJson}, got ${actualJson}`);
  }
}

/**
 * @param {string} input
 * @param {number[]} expected
 * @param {string} message
 */
function assertRatios(input, expected, message) {
  const result = parseRatioCommand(input);
  assertEqual(result, { kind: 'ratios', ratios: expected }, message);
}

/**
 * @param {string} input
 * @param {string} message
 */
function assertError(input, message) {
  const result = parseRatioCommand(input);
  assertEqual(result.kind, 'error', message);
}

assertRatios('ratio 2:1:2', [2, 1, 2], 'parses colon syntax');
assertRatios('ratio 2.1.2', [2, 1, 2], 'parses dot syntax');
assertRatios('ratio 0:1:4', [0, 1, 4], 'keeps a single zero position');
assertRatios('ratio 1:0:4', [1, 0, 4], 'keeps a middle zero position');
assertRatios('ratio 0:1:0', [0, 1, 0], 'keeps a trailing zero position');
assertRatios('ratio 0:0:5', [0, 0, 1], 'normalizes zero parent and current to preview preset');
assertRatios('ratio 0.0.999', [0, 0, 1], 'normalizes dot syntax preview form');

assertEqual(parseRatioCommand('ratio'), { kind: 'preset', preset: 'default' }, 'bare ratio is the default preset');
assertEqual(parseRatioCommand('ratio default'), { kind: 'preset', preset: 'default' }, 'default alias');
assertEqual(parseRatioCommand('ratio dual'), { kind: 'preset', preset: 'dual' }, 'dual alias');
assertEqual(parseRatioCommand('ratio preview'), { kind: 'preset', preset: 'preview' }, 'preview alias');

assertError('ratio 1:2', 'rejects two values');
assertError('ratio 1:2:3:4', 'rejects four values');
assertError('ratio -1:2:3', 'rejects negative values');
assertError('ratio 1.5:2:3', 'rejects decimals');
assertError('ratio 1:2.2', 'rejects mixed separators');
assertError('ratio 1.2:2', 'rejects mixed separators in the other order');
assertError('ratio 0:0:0', 'rejects all zero values');
assertError('ratio a:b:c', 'rejects non-numeric values');

assertEqual(visibleColumnCount([1, 1, 3]), 3, 'counts three visible columns');
assertEqual(visibleColumnCount([0, 1, 4]), 2, 'counts two visible columns');
assertEqual(visibleColumnCount([0, 0, 1]), 1, 'counts one visible column');
assertEqual(visibleColumnCount([0, 0, 0]), 0, 'counts no visible columns');

assertEqual(nextLayoutRatios([1, 1, 3], [1, 1, 3]), [0, 1, 4], 'three columns cycle to two');
assertEqual(nextLayoutRatios([2, 1, 2], [2, 1, 2]), [0, 1, 4], 'custom three columns cycle to fixed two preset');
assertEqual(nextLayoutRatios([0, 1, 4], [2, 1, 2]), [0, 0, 1], 'two columns cycle to one');
assertEqual(nextLayoutRatios([1, 0, 4], [1, 1, 3]), [0, 0, 1], 'non-standard two columns cycle to one');
assertEqual(nextLayoutRatios([0, 0, 1], [2, 1, 2]), [2, 1, 2], 'one column restores the three-column snapshot');
assertEqual(nextLayoutRatios([0, 0, 1], [0, 1, 4]), RATIO_PRESETS.default, 'one column falls back to the default three-column preset');

assertEqual(correctActiveColumn('current', [0, 0, 1]), 'preview', 'hidden current moves focus to preview');
assertEqual(correctActiveColumn('parent', [0, 1, 4]), 'current', 'hidden parent moves focus to current');
assertEqual(correctActiveColumn('preview', [0, 1, 0]), 'current', 'hidden preview moves focus to current');
assertEqual(correctActiveColumn('terminal', [0, 0, 1]), 'terminal', 'terminal focus is left alone');
assertEqual(correctActiveColumn('current', [1, 1, 3]), 'current', 'visible focus is left alone');

assertEqual(nextVisibleColumn('current', 'left', [0, 1, 4]), null, 'does not move left into a hidden parent');
assertEqual(nextVisibleColumn('current', 'left', [1, 1, 4]), 'parent', 'moves left to a visible parent');
assertEqual(nextVisibleColumn('current', 'right', [0, 1, 0]), null, 'does not move right into a hidden preview');
assertEqual(nextVisibleColumn('parent', 'right', [0, 1, 4]), 'current', 'skips a hidden current when moving right');
assertEqual(nextVisibleColumn('parent', 'right', [0, 0, 1]), 'preview', 'skips both hidden columns when moving right');
