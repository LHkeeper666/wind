// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const source = readFileSync(new URL('../src/lib/previewers/ImagePreviewer.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText;

test('cleanup does not revoke an image URL while its decode is pending', async () => {
  let image;
  const revoked = [];
  const host = { dataset: {}, innerHTML: '', appendChild() {} };
  const exports = {};
  new Function('require', 'exports', 'Image', 'URL', 'document', compiled)(
    id => id === './ImageViewport' ? { ImageViewport: class {} } : { logError() {} },
    exports,
    class { constructor() { image = this; } set src(value) { this._src = value; } },
    { createObjectURL: () => 'blob:pending', revokeObjectURL: url => revoked.push(url) },
    { querySelectorAll: () => [], createElement: () => ({ className: '', appendChild() {} }) },
  );
  const previewer = new exports.ImagePreviewer();
  const rendering = previewer.render(new ArrayBuffer(2), host);
  previewer.cleanupUnused();
  assert.deepEqual(revoked, []);
  image.onload();
  await rendering;
  previewer.cleanupUnused();
  assert.deepEqual(revoked, ['blob:pending']);
});

test('repeated afterSwap for the same image keeps its viewport', () => {
  let created = 0;
  let disposed = 0;
  const img = {};
  const wrapper = { querySelector: () => img };
  const container = {
    dataset: { filePath: 'photo.jpg' },
    classList: { add() {} },
    querySelector: selector => selector === '.preview-image' ? wrapper : null,
  };
  const exports = {};
  new Function('require', 'exports', 'document', compiled)(
    id => id === './ImageViewport'
      ? { ImageViewport: class {
        constructor() { created++; }
        owns(w, i) { return w === wrapper && i === img; }
        dispose() { disposed++; }
      } }
      : { logError() {} },
    exports,
    { querySelectorAll: () => [] },
  );
  const previewer = new exports.ImagePreviewer();
  previewer.afterSwap(container);
  previewer.afterSwap(container);
  assert.equal(created, 1);
  assert.equal(disposed, 0);
  previewer.dispose();
});
