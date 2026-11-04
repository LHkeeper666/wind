// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const source = readFileSync(new URL('../src/lib/utils/pdf-zoom.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText;
const zoom = {};
new Function('exports', compiled)(zoom);

test('wheel zoom follows mouse and touchpad displacement with bounded events', () => {
  assert.equal(zoom.wheelZoomDelta(-100, 0), 0.1);
  assert.equal(zoom.wheelZoomDelta(-10, 0), 0.01);
  assert.equal(zoom.wheelZoomDelta(-3, 1), 0.048);
  assert.equal(zoom.wheelZoomDelta(-10000, 0), 0.16);
});

test('zoom targets clamp and consecutive key steps accumulate', () => {
  assert.equal(zoom.nextZoomScale(0.3, -zoom.PDF_ZOOM_KEY_STEP, 0.3, 5), 0.3);
  assert.equal(zoom.nextZoomScale(1.05, zoom.PDF_ZOOM_KEY_STEP, 0.3, 5), 1.1);
  assert.equal(zoom.nextZoomScale(5, zoom.PDF_ZOOM_KEY_STEP, 0.3, 5), 5);
});

test('animation advances toward the latest target and converges', () => {
  const first = zoom.advanceZoomScale(1, 2, 16);
  assert.ok(first > 1 && first < 2);
  assert.ok(zoom.advanceZoomScale(first, 0.5, 16) < first);
  let scale = first;
  for (let i = 0; i < 100; i++) scale = zoom.advanceZoomScale(scale, 2, 16);
  assert.equal(scale, 2);
});
