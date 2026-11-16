// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const source = readFileSync(new URL('../src/lib/previewers/ImageViewport.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText;

function harness({ request = async () => btoa('original'), decode = async () => {},
  viewportSize = { width: 600, height: 400 } } = {}) {
  const frames = new Map();
  let nextFrame = 1;
  let requests = 0;
  let revoked = 0;
  let loaded = 0;
  let resize;
  const button = { disabled: false, textContent: '查看原图' };
  const slot = {
    dataset: { filePath: 'photo.jpg', thumbIsThumbnail: 'true' },
    querySelector: () => button,
  };
  const viewport = {
    clientWidth: viewportSize.width, clientHeight: viewportSize.height,
    getBoundingClientRect() { return { left: 0, top: 0, width: this.clientWidth, height: this.clientHeight }; },
  };
  let displayed;
  const img = { naturalWidth: 1200, naturalHeight: 800, style: {}, src: 'thumbnail',
    decode: async () => {}, replaceWith(next) { displayed = next; } };
  displayed = img;
  const globals = {
    ResizeObserver: class { constructor(callback) { resize = callback; } observe() {} disconnect() {} },
    requestAnimationFrame: callback => { const id = nextFrame++; frames.set(id, callback); return id; },
    cancelAnimationFrame: id => frames.delete(id),
    matchMedia: () => ({ matches: false }),
    devicePixelRatio: 1,
    Image: class { style = {}; set src(value) { this._src = value; } get src() { return this._src; }
      decode() { return decode(); } },
    URL: { createObjectURL: () => 'blob:original', revokeObjectURL: () => revoked++ },
  };
  const dependencies = {
    '@tauri-apps/api/core': { invoke: async () => { requests++; return request(); } },
    '../utils/pdf-zoom': {
      wheelZoomDelta: y => -y * 0.001,
      advanceZoomScale: (current, target) => Math.abs(target - current) < 0.0005 ? target : current + (target - current) * 0.5,
    },
    '../utils/log': { logError() {} },
  };
  const exports = {};
  const names = Object.keys(globals);
  new Function('require', 'exports', ...names, compiled)(id => dependencies[id], exports, ...Object.values(globals));
  const controller = new exports.ImageViewport(viewport, img, slot, () => loaded++);
  const flushFrames = () => {
    for (let i = 0; i < 100 && frames.size; i++) {
      const callbacks = [...frames.values()];
      frames.clear();
      callbacks.forEach(callback => callback((i + 1) * 16));
    }
  };
  return { controller, get img() { return displayed; }, slot, button, viewport, flushFrames, resize: () => resize(),
    get requests() { return requests; }, get revoked() { return revoked; },
    get loaded() { return loaded; } };
}

test('image fits viewport and movement stays within visible bounds', () => {
  const h = harness();
  assert.equal(h.img.style.width, '600px');
  assert.equal(h.img.style.height, '400px');
  h.controller.pan(100, 100);
  assert.equal(h.img.style.left, '0px');
  assert.equal(h.img.style.top, '0px');
  assert.equal(h.requests, 0);
  h.controller.dispose();
});

test('resize refits before manual zoom and directional keys pan only the image', () => {
  const h = harness();
  h.viewport.clientWidth = 300;
  h.viewport.clientHeight = 400;
  h.resize();
  assert.equal(h.img.style.width, '300px');
  assert.equal(h.img.style.height, '200px');
  assert.equal(h.img.style.top, '100px');
  h.controller.zoom(1);
  h.flushFrames();
  const event = { code: 'KeyL', ctrlKey: false, altKey: false, metaKey: false, shiftKey: false };
  const oldLeft = parseFloat(h.img.style.left);
  assert.equal(h.controller.handleKeydown(event), true);
  assert.ok(parseFloat(h.img.style.left) < oldLeft);
  assert.equal(h.controller.handleKeydown({ ...event, ctrlKey: true }), false);
  h.controller.dispose();
});

test('image refits when selection preview becomes visible after rendering', () => {
  const h = harness({ viewportSize: { width: 0, height: 0 } });
  assert.equal(h.img.style.width, undefined);
  h.controller.zoom(0.8);
  assert.equal(h.img.style.width, undefined);
  h.viewport.clientWidth = 600;
  h.viewport.clientHeight = 400;
  h.resize();
  assert.equal(h.img.style.width, '600px');
  assert.equal(h.img.style.height, '400px');
  h.controller.zoom(0.1);
  h.flushFrames();
  assert.ok(parseFloat(h.img.style.width) > 600);
  assert.equal(h.requests, 0);
  h.controller.dispose();
});

test('slot reconstruction restores image zoom and pan', () => {
  const first = harness();
  first.controller.zoom(1);
  first.flushFrames();
  first.controller.pan(-70, -30);
  const saved = first.controller.snapshot();
  const expected = { ...first.img.style };
  first.controller.dispose();
  const second = harness({ viewportSize: { width: 0, height: 0 } });
  second.controller.restore(saved);
  second.viewport.clientWidth = 600;
  second.viewport.clientHeight = 400;
  second.resize();
  for (const field of ['width', 'height', 'left', 'top']) {
    assert.ok(Math.abs(parseFloat(second.img.style[field]) - parseFloat(expected[field])) < 0.001, field);
  }
  second.controller.dispose();
});

test('enlargement crosses thumbnail resolution once and preserves view on replacement', async () => {
  const h = harness();
  h.controller.zoom(0.8, 150, 100);
  h.flushFrames();
  const before = { ...h.img.style };
  assert.equal(h.requests, 0);
  await new Promise(resolve => setTimeout(resolve, 210));
  assert.equal(h.img.src, 'blob:original');
  assert.equal(h.loaded, 1);
  assert.equal(h.button.textContent, '原图已加载');
  assert.deepEqual(h.img.style, before);
  h.controller.zoom(0.1);
  assert.equal(h.requests, 1);
  h.controller.dispose();
  assert.equal(h.revoked, 1);
});

test('thumbnail stays mounted until original image is decoded', async () => {
  let finishDecode;
  const h = harness({ decode: () => new Promise(resolve => { finishDecode = resolve; }) });
  const originalNode = h.img;
  const loading = h.controller.loadOriginal();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(h.img, originalNode);
  assert.equal(h.img.src, 'thumbnail');
  finishDecode();
  await loading;
  assert.notEqual(h.img, originalNode);
  assert.equal(h.img.src, 'blob:original');
  h.controller.dispose();
});

test('stale original does not replace the next image', async () => {
  let resolveRequest;
  const h = harness({ request: () => new Promise(resolve => { resolveRequest = resolve; }) });
  h.controller.zoom(0.8);
  await new Promise(resolve => setTimeout(resolve, 210));
  assert.equal(h.requests, 1);
  h.slot.dataset.filePath = 'next.jpg';
  resolveRequest(btoa('old image'));
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(h.img.src, 'thumbnail');
  assert.equal(h.loaded, 0);
  h.controller.dispose();
});

test('failed original keeps thumbnail and allows retry', async () => {
  let attempt = 0;
  const h = harness({ request: async () => { if (++attempt === 1) throw Error('failed'); return btoa('original'); } });
  h.controller.zoom(0.8);
  await new Promise(resolve => setTimeout(resolve, 210));
  assert.equal(h.img.src, 'thumbnail');
  assert.equal(h.button.disabled, false);
  await h.controller.loadOriginal();
  assert.equal(h.img.src, 'blob:original');
  assert.equal(h.requests, 2);
  h.controller.dispose();
});
