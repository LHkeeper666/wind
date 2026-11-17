// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
// @ts-expect-error Node's type stripping resolves the .ts source directly.
import { firstAvailableCopyPath, buildLocalPasteTasks } from '../src/lib/utils/copy-naming.ts';

test('keeps extension and chooses the first available numeric suffix', async () => {
  const existing = new Set(['c:\\dest\\report.pdf', 'c:\\dest\\report (1).pdf']);
  const reserved = new Set();
  const exists = async path => existing.has(path.toLowerCase());
  assert.equal(await firstAvailableCopyPath('C:\\dest\\report.pdf', reserved, exists), 'C:\\dest\\report (2).pdf');
  assert.equal(await firstAvailableCopyPath('C:\\dest\\report.pdf', reserved, exists), 'C:\\dest\\report (3).pdf');
});

test('appends suffix to extensionless and leading-dot names', async () => {
  const exists = async () => false;
  assert.equal(await firstAvailableCopyPath('C:\\dest\\README', new Set(), exists), 'C:\\dest\\README (1)');
  assert.equal(await firstAvailableCopyPath('C:\\dest\\.gitignore', new Set(), exists), 'C:\\dest\\.gitignore (1)');
});

test('nested copy avoids another incoming sibling name', async () => {
  const exists = async path => path === 'C:\\source\\report (1).pdf';
  assert.equal(
    await firstAvailableCopyPath('C:\\dest\\report.pdf', new Set(), exists, 'C:\\source\\'),
    'C:\\dest\\report (2).pdf',
  );
});

test('top-level copy and cut tasks carry the selected destination', () => {
  const source = 'C:\\source\\report.pdf';
  const args = [[source], [{ path: source, size: 12 }], 'C:\\dest', new Map([[source, 'C:\\dest\\report (1).pdf']]), new Map(), new Map(), new Set([source])];
  for (const operation of ['copy', 'cut']) {
    const [task] = buildLocalPasteTasks(args[0], args[1], args[2], operation, ...args.slice(3));
    assert.equal(task.destination, 'C:\\dest\\report (1).pdf');
    assert.equal(task.keep_copy, true);
    assert.equal(task.op_type, operation === 'copy' ? 'copy' : 'move');
  }
});

test('directory task carries nested rename and skip decisions', () => {
  const source = 'C:\\source\\folder';
  const [task] = buildLocalPasteTasks(
    [source], [{ path: source, skip_rel_paths: ['ignored.txt'] }], 'C:\\dest', 'copy',
    new Map(), new Map([[source, ['skip.txt']]]),
    new Map([[source, { 'nested\\report.pdf': 'report (1).pdf' }]]), new Set(),
  );
  assert.equal(task.destination, 'C:\\dest\\folder');
  assert.deepEqual(task.rename_rel_paths, { 'nested\\report.pdf': 'report (1).pdf' });
  assert.deepEqual(task.skip_rel_paths, ['ignored.txt', 'skip.txt']);
});
