// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { ftpConflictDecision, nextFtpCopyName, queuedMoveMessage, summarizeMoveOutcomes } from '../src/lib/utils/ftp-move-policy.ts';

test('mixed move outcomes keep failed and partial entries for retry', () => {
  const results = summarizeMoveOutcomes([
    { entry: 'a', result: { status: 'done' } },
    { entry: 'b', result: { status: 'failed', error: 'timeout' } },
    { entry: 'c', result: { status: 'partial', error: 'cleanup failed' } },
  ], ['d'], 1);
  assert.equal(results.moved, 2);
  assert.deepEqual(results.failed, ['d', 'b', 'c']);
});

test('FTP keep-copy name retains extension and avoids occupied names', () => {
  const occupied = new Set(['report.pdf', 'report (copy 1).pdf']);
  assert.equal(nextFtpCopyName('report.pdf', occupied), 'report (copy 2).pdf');
  assert.equal(nextFtpCopyName('folder', new Set(['folder (copy 1)'])), 'folder (copy 2)');
});

test('a pending move is labelled queued until a terminal outcome exists', () => {
  assert.equal(queuedMoveMessage(2), '2 move(s) queued');
  assert.deepEqual(summarizeMoveOutcomes([], []), { moved: 0, failed: [] });
});

test('FTP conflict choices preserve skip and abort, and rename keep-copy', () => {
  const occupied = new Set(['report.pdf']);
  assert.deepEqual(ftpConflictDecision('report.pdf', occupied, 'skip'), { action: 'skip' });
  assert.deepEqual(ftpConflictDecision('report.pdf', occupied, 'abort'), { action: 'abort' });
  assert.deepEqual(ftpConflictDecision('report.pdf', occupied, 'overwrite'), { action: 'target', targetName: 'report.pdf' });
  assert.deepEqual(ftpConflictDecision('report.pdf', occupied, 'keep-copy'), { action: 'target', targetName: 'report (copy 1).pdf' });
});
