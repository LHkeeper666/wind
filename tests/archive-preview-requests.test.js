// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { ArchivePreviewRequests } from '../src/lib/utils/archive-preview-requests.ts';

test('rapid A to B to C selection only starts the current read', async () => {
  const requests = new ArchivePreviewRequests();
  let selected = 'A';
  const started = [];
  const load = key => requests.run(key, () => selected === key, async () => {
    started.push(key);
    return key;
  });
  const a = load('A');
  selected = 'B';
  const b = load('B');
  selected = 'C';
  const c = load('C');
  assert.deepEqual(await Promise.all([a, b, c]), [null, null, 'C']);
  assert.deepEqual(started, ['C']);
});

test('same target shares an in-flight read and stale result is ignored', async () => {
  const requests = new ArchivePreviewRequests();
  let selected = 'A';
  let starts = 0;
  let finish;
  const start = () => {
    starts++;
    return new Promise(resolve => { finish = resolve; });
  };
  const first = requests.run('A', () => selected === 'A', start);
  const second = requests.run('A', () => selected === 'A', start);
  await new Promise(resolve => setTimeout(resolve, 60));
  assert.equal(starts, 1);
  selected = 'B';
  finish('A');
  assert.deepEqual(await Promise.all([first, second]), [null, null]);
});
