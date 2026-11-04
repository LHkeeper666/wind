// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const source = readFileSync(new URL('../src/lib/utils/pdf-shared.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText;
const exports = {};
const requests = [];
const invoke = (_command, args) => new Promise(resolve => requests.push({ ...args, resolve }));
new Function('exports', 'require', compiled)(exports, name => {
  if (name === '@tauri-apps/api/core') return { invoke };
  if (name === './log') return { logInfo() {} };
  throw new Error(`Unexpected import: ${name}`);
});

test('PDF render queue retains, reprioritizes and deduplicates compatible work', async () => {
  const scheduler = new exports.PdfRenderScheduler();
  let releaseRunning;
  const running = scheduler.enqueue('running', () => new Promise(resolve => { releaseRunning = resolve; }), 0);
  const stale = scheduler.enqueue('stale', () => Promise.resolve('stale'), 200);
  const order = [];
  const prefetch = scheduler.enqueue('prefetch', async () => { order.push('prefetch'); }, 200);
  const same = scheduler.enqueue('prefetch', async () => { throw Error('duplicate'); }, 100);
  assert.equal(prefetch, same);
  const visible = scheduler.enqueue('visible', async () => { order.push('visible'); }, 0);
  scheduler.retain(new Set(['running', 'prefetch', 'visible']));
  await assert.rejects(stale, /stale PDF render request/);
  releaseRunning();
  await Promise.all([running, prefetch, visible]);
  assert.deepEqual(order, ['visible', 'prefetch']);
});

test('PDF base pixel cap and shared cache budget', () => {
  const base = exports.getPdfBaseSpec('revision', 2, { width: 5000, height: 4000 }, 1.125);
  assert.ok(base.width * base.height <= exports.PDF_BASE_MAX_PIXELS + base.width + base.height);
  assert.match(base.key, /revision:base:2:1.125/);
  const cache = new exports.PdfTileCache();
  cache.setPinned([base.key]);
  cache.set(base, { data: '', format: 'jpeg', width: base.width, height: base.height,
    page: 2, scale: base.scale, tileX: 0, tileY: 0, lowRes: true });
  assert.equal(cache.getSizeBytes(), base.width * base.height * 4);
});

test('PDF coordinates convert from bottom left to top left', () => {
  assert.deepEqual(
    exports.pdfTextRect({ text: '中', x: 10, y: 30, width: 12, height: 16 }, 100, 2),
    { left: 20, top: 108, width: 24, height: 32 },
  );
});

test('text layout cache is bounded and rejects stale results after clear', async () => {
  const cache = new exports.PdfTextLayoutCache(2);
  const first = cache.get('v1', 'file.pdf', 0);
  const same = cache.get('v1', 'file.pdf', 0);
  assert.equal(requests.length, 1);
  requests.shift().resolve({ page: 0, characters: [] });
  assert.deepEqual(await first, await same);

  const pending = cache.get('v1', 'file.pdf', 1);
  cache.clear();
  requests.shift().resolve({ page: 1, characters: [] });
  await pending;
  const fresh = cache.get('v2', 'file.pdf', 1);
  assert.equal(requests.length, 1);
  requests.shift().resolve({ page: 1, characters: [{ text: '新' }] });
  assert.equal((await fresh).characters[0].text, '新');
});

test('search input changes trigger a new request', () => {
  assert.equal(exports.shouldExecutePdfSearch('new', 'old', false), true);
  assert.equal(exports.shouldExecutePdfSearch(' old ', 'old', false), false);
  assert.equal(exports.shouldExecutePdfSearch('old', 'old', true), true);
});

test('copy uses character text without block layout separators', () => {
  const layer = {};
  const root = { contains: node => node === layer };
  const node = { parentElement: { closest: () => layer } };
  const previousWindow = globalThis.window;
  globalThis.window = {
    getSelection: () => ({
      isCollapsed: false,
      rangeCount: 1,
      anchorNode: node,
      focusNode: node,
      getRangeAt: () => ({ cloneContents: () => ({ textContent: 'First line\r\nSecond line' }) }),
    }),
  };
  let copied = '';
  let prevented = false;
  try {
    exports.copyPdfTextSelection({
      clipboardData: { setData: (_type, value) => { copied = value; } },
      preventDefault: () => { prevented = true; },
    }, root);
    assert.equal(copied, 'First line\r\nSecond line');
    assert.equal(prevented, true);
  } finally {
    globalThis.window = previousWindow;
  }
});
