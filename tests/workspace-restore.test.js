// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { clampEditorSelection, isCurrentRestore } from '../src/lib/utils/workspace-restore.ts';
import { TabCacheManager } from '../src/lib/utils/tab-cache.ts';

test('late restore callbacks cannot target a newer workspace, slot, or generation', () => {
  const request = { workspaceId: 2, slotId: 3, generation: 7 };
  assert.equal(isCurrentRestore(request, request), true);
  assert.equal(isCurrentRestore(request, { ...request, workspaceId: 4 }), false);
  assert.equal(isCurrentRestore(request, { ...request, slotId: 5 }), false);
  assert.equal(isCurrentRestore(request, { ...request, generation: 8 }), false);
});

test('editor selection retains zero and direction while clamping to changed content', () => {
  assert.deepEqual(clampEditorSelection({ anchor: 0, head: 6 }, 4), { anchor: 0, head: 4 });
  assert.deepEqual(clampEditorSelection({ anchor: 8, head: 2 }, 4), { anchor: 4, head: 2 });
  assert.deepEqual(clampEditorSelection({ anchor: -2, head: 99 }, 0), { anchor: 0, head: 0 });
});

test('preview slots retain independent content and position while stale renders are rejected', () => {
  const cache = new TabCacheManager();
  cache.set(101, { filePath: 'a.md', content: 'draft A', previewScrollTop: 80, tocSelectedIndex: 2, tocScrollTop: 130 });
  cache.set(102, { filePath: 'b.md', content: 'draft B', previewScrollTop: 220, tocSelectedIndex: 4, tocScrollTop: 310 });
  assert.equal(cache.get(101).content, 'draft A');
  assert.equal(cache.get(102).previewScrollTop, 220);
  assert.equal(cache.get(101).tocScrollTop, 130);
  assert.equal(cache.get(102).tocScrollTop, 310);
  const oldVersion = cache.requestRender(101);
  const newVersion = cache.requestRender(101);
  assert.equal(cache.isCurrentRender(101, 'a.md', oldVersion, 101, 'a.md'), false);
  assert.equal(cache.isCurrentRender(101, 'a.md', newVersion, 102, 'b.md'), false);
  assert.equal(cache.isCurrentRender(101, 'a.md', newVersion, 101, 'a.md'), true);
  cache.delete(101);
  assert.equal(cache.get(101), undefined);
  assert.equal(cache.get(102).content, 'draft B');
});
