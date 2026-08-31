import {
  getCollapseSelectionTarget,
  isProjectTreePathWithin,
  projectTreePathKey,
} from '../src/lib/utils/project-tree-focus.js';

/**
 * @param {unknown} actual
 * @param {unknown} expected
 * @param {string} message
 */
function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
  }
}

assertEqual(projectTreePathKey('C:/Work/App/'), 'c:\\work\\app', 'normalizes slashes and trailing separators');
assertEqual(projectTreePathKey('C:'), 'c:\\', 'keeps drive roots addressable');

assertEqual(
  isProjectTreePathWithin('C:\\repo\\src\\file.ts', 'C:\\repo\\src'),
  true,
  'treats direct children as descendants',
);
assertEqual(
  isProjectTreePathWithin('C:\\repo\\src\\feature\\file.ts', 'C:\\repo\\src'),
  true,
  'treats nested children as descendants',
);
assertEqual(
  isProjectTreePathWithin('C:\\repo\\src\\file.ts', 'C:\\'),
  true,
  'treats drive roots as ancestors',
);

assertEqual(
  isProjectTreePathWithin('C:\\repo\\src-app\\file.ts', 'C:\\repo\\src'),
  false,
  'does not match similarly prefixed sibling paths',
);
assertEqual(
  isProjectTreePathWithin('C:\\repo\\application\\file.ts', 'C:\\repo\\app'),
  false,
  'does not match sibling paths with a shared name prefix',
);

assertEqual(
  getCollapseSelectionTarget('C:\\repo\\src\\file.ts', 'C:\\repo\\src'),
  'C:\\repo\\src',
  'moves selection to a collapsed parent directory',
);
assertEqual(
  getCollapseSelectionTarget('C:\\repo\\src\\feature\\file.ts', 'C:\\repo\\src'),
  'C:\\repo\\src',
  'moves selection to a collapsed ancestor directory',
);

assertEqual(
  getCollapseSelectionTarget('C:\\repo\\src\\file.ts', 'C:\\repo\\docs'),
  'C:\\repo\\src\\file.ts',
  'preserves selection when collapsing an unrelated directory',
);
assertEqual(
  getCollapseSelectionTarget('C:\\repo\\src', 'C:\\repo\\src'),
  'C:\\repo\\src',
  'keeps the selected directory active when collapsing itself',
);
