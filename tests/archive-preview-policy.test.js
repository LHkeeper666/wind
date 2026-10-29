// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { archivePreviewSkipReason, archivePreviewMessage, ARCHIVE_PREVIEW_LIMIT } from '../src/lib/utils/archive-preview-policy.ts';

test('video and oversized entries skip content reads before IPC', () => {
  assert.equal(archivePreviewSkipReason(true, 1), 'video');
  assert.equal(archivePreviewSkipReason(true, null), 'video');
  assert.equal(archivePreviewSkipReason(false, ARCHIVE_PREVIEW_LIMIT + 1), 'size_limit');
  assert.equal(archivePreviewSkipReason(false, ARCHIVE_PREVIEW_LIMIT), null);
});

test('placeholder explains size limit and RAR extraction path', () => {
  assert.match(archivePreviewMessage('size_limit'), /10 MiB/);
  assert.match(archivePreviewMessage('rar_unbounded'), /Extract the file/);
  assert.match(archivePreviewMessage('video'), /system player/);
});
