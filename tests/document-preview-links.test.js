// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { classifyPreviewLink, headingAnchorIds, linkifiedHref } from '../src/lib/utils/document-preview-links.ts';

const source = 'C:\\docs\\notes\\readme.md';

test('resolves relative and absolute local links without WebView URLs', () => {
  assert.deepEqual(classifyPreviewLink('../images/a%20b.png', source), { kind: 'file', path: 'C:\\docs\\images\\a b.png' });
  assert.deepEqual(classifyPreviewLink('D:\\work\\file.pdf', source), { kind: 'file', path: 'D:\\work\\file.pdf' });
  assert.deepEqual(classifyPreviewLink('file:///C:/docs/a%23b.md', source), { kind: 'file', path: 'C:\\docs\\a#b.md' });
  assert.deepEqual(classifyPreviewLink('file://server/share/file.md', source), { kind: 'file', path: '\\\\server\\share\\file.md' });
});

test('preserves local file fragments separately from encoded path characters', () => {
  assert.deepEqual(classifyPreviewLink('tasks.md#some-heading', source), { kind: 'file', path: 'C:\\docs\\notes\\tasks.md', fragment: 'some-heading' });
  assert.deepEqual(classifyPreviewLink('../tasks%23draft.md#%E4%BB%BB%E5%8A%A1', source), { kind: 'file', path: 'C:\\docs\\tasks#draft.md', fragment: '任务' });
  assert.deepEqual(classifyPreviewLink('D:\\work\\tasks.md#', source), { kind: 'file', path: 'D:\\work\\tasks.md', fragment: '' });
  assert.deepEqual(classifyPreviewLink('file:///C:/docs/tasks.md#heading', source), { kind: 'file', path: 'C:\\docs\\tasks.md', fragment: 'heading' });
  assert.deepEqual(classifyPreviewLink('file:///C:/docs/tasks.md#', source), { kind: 'file', path: 'C:\\docs\\tasks.md', fragment: '' });
  assert.deepEqual(classifyPreviewLink('tasks.md#11.10', source), { kind: 'file', path: 'C:\\docs\\notes\\tasks.md', fragment: '11.10' });
});

test('classifies websites and same-document anchors', () => {
  assert.deepEqual(classifyPreviewLink('https://example.com/a', source), { kind: 'url', url: 'https://example.com/a' });
  assert.deepEqual(classifyPreviewLink('#some%20heading', source), { kind: 'anchor', id: 'some heading' });
  assert.deepEqual(classifyPreviewLink('#', source), { kind: 'anchor', id: '' });
});

test('matches heading text fragments when rendered headings have generated ids', () => {
  assert.deepEqual(headingAnchorIds(['标题', 'Some Heading!', '标题', '标题']),
    ['标题', 'some-heading', '标题-1', '标题-2']);
  assert.deepEqual(headingAnchorIds(['11.10']), ['1110']);
});

test('uses HTTPS for generated bare-domain links and preserves explicit schemes', () => {
  assert.equal(linkifiedHref('http://Claude.ai/code', true), 'https://Claude.ai/code');
  assert.equal(linkifiedHref('http://example.com', false), 'http://example.com');
});

test('rejects unsupported and malformed links', () => {
  for (const href of ['javascript:alert(1)', 'mailto:a@example.com', '//example.com', '%ZZ', 'file:///C:/x?a=1']) {
    assert.deepEqual(classifyPreviewLink(href, source), { kind: 'unsupported' }, href);
  }
});
