// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { isSamePreviewIdentity, isValidTextCache, shouldReloadCachedFile, TabCacheManager, textEditAction } from '../src/lib/utils/tab-cache.ts';
import { previewSlotKey } from '../src/lib/utils/preview-slot-key.ts';

const identity = (workspaceId, slotId, path) => ({ tabId: previewSlotKey(workspaceId, slotId), path });
const cached = (overrides = {}) => ({
  filePath: 'note.md', content: 'body', savedContent: 'body', binaryContent: null,
  originalFileSize: 4, textLoadState: 'ready', isModified: false, ...overrides,
});

test('preview confirmation keeps the same identity after rapid selections', () => {
  const a = identity(1, 1, 'a.md');
  const b = identity(2, 1, 'b.md');
  const c = identity(3, 1, 'c.md');
  assert.equal(isSamePreviewIdentity(a, b), false);
  assert.equal(isSamePreviewIdentity(b, c), false);
  assert.equal(isSamePreviewIdentity(c, b), false);
  assert.equal(isSamePreviewIdentity(b, b), true);
  assert.equal(isSamePreviewIdentity(b, a), false);
});

test('tab and preview slot both distinguish the same path', () => {
  assert.equal(isSamePreviewIdentity(identity(1, 1, 'same.md'), identity(2, 1, 'same.md')), false);
  assert.equal(isSamePreviewIdentity(identity(1, 1, 'same.md'), identity(1, 2, 'same.md')), false);
});

test('damaged empty cache is rejected while zero-byte files and empty drafts remain valid', () => {
  assert.equal(isValidTextCache(cached({ content: '', textLoadState: 'transition' })), false);
  assert.equal(isValidTextCache(cached({ content: '', textLoadState: 'error' })), false);
  assert.equal(isValidTextCache(cached({ content: '', textLoadState: 'ready' })), false);
  assert.equal(isValidTextCache(cached({ content: '', savedContent: '', originalFileSize: 0 })), true);
  assert.equal(isValidTextCache(cached({ content: '', isModified: true })), true);
});

test('switching cached files retains each file size and independent draft', () => {
  const cache = new TabCacheManager();
  cache.set(101, cached({ originalFileSize: 100 }));
  cache.set(201, cached({ filePath: 'other.md', content: '', originalFileSize: 0 }));
  assert.equal(cache.get(101).originalFileSize, 100);
  assert.equal(cache.get(201).originalFileSize, 0);
  cache.set(201, cached({ filePath: 'other.md', content: '', isModified: true, originalFileSize: 100 }));
  assert.equal(isValidTextCache(cache.get(201)), true);
  assert.equal(cache.get(101).content, 'body');
});

test('edit entry waits only for loading, reports errors, and accepts ready empty text', () => {
  assert.equal(textEditAction('loading'), 'wait');
  assert.equal(textEditAction('transition'), 'wait');
  assert.equal(textEditAction('error'), 'error');
  assert.equal(textEditAction('ready'), 'edit');
});

test('damaged empty text reloads even with unchanged mtime, without reloading PDF or drafts', () => {
  const empty = cached({ content: '', fileMtime: 123 });
  assert.equal(shouldReloadCachedFile(empty, { size: 4, modified: 123 }, true), true);
  assert.equal(shouldReloadCachedFile(empty, { size: 4, modified: 123 }, false), false);
  assert.equal(shouldReloadCachedFile(cached({ content: '', isModified: true, fileMtime: 123 }), { size: 4, modified: 123 }, true), false);
  assert.equal(shouldReloadCachedFile(cached({ content: '', originalFileSize: 0, fileMtime: 123 }), { size: 0, modified: 123 }, true), false);
});
