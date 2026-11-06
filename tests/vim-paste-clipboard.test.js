// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { build } from 'esbuild';

const result = await build({
  entryPoints: ['src/lib/utils/clipboard-bridge.ts', 'src/lib/utils/vim-paste-keys.ts'],
  bundle: true, write: false, outdir: 'out', platform: 'node', format: 'esm', splitting: false,
  plugins: [{
    name: 'vim-stub', setup(plugin) {
      plugin.onResolve({ filter: /^@replit\/codemirror-vim$/ }, () => ({ path: 'vim', namespace: 'stub' }));
      plugin.onLoad({ filter: /.*/, namespace: 'stub' }, () => ({ contents: `
        const register = { value: '', setText(value) { this.value = value }, toString() { return this.value } };
        const rc = { unnamedRegister: register, pushText(_name, _op, value) { register.setText(value) } };
        globalThis.__vimRegisterController = rc;
        export const Vim = {
          getRegisterController: () => rc,
          multiSelectHandleKey(cm, key) { cm.handled.push([key, register.value]) },
        };
        export const getCM = view => view.cm;
      ` }));
    },
  }],
});
const bundled = name => result.outputFiles.find(file => file.path.endsWith(`${name}.js`)).text;
const bridgeModule = await import(`data:text/javascript;base64,${Buffer.from(bundled('clipboard-bridge')).toString('base64')}`);
const bridgeRegisterController = globalThis.__vimRegisterController;
const queueModule = await import(`data:text/javascript;base64,${Buffer.from(bundled('vim-paste-keys')).toString('base64')}`);

function view() {
  return { cm: { state: { vim: { inputState: { keyBuffer: [], registerName: undefined, operator: null } } }, handled: [] } };
}

test('default put reads fresh text, clears empty text, and falls back on failure', async () => {
  let value = 'PDF copy';
  navigator.clipboard = { readText: async () => value, writeText: async () => {} };
  const editor = view();
  const bridge = bridgeModule.initClipboardBridge(editor);
  assert.equal(await bridge.readForPaste(), 'text');
  value = '';
  assert.equal(await bridge.readForPaste(), 'empty');
  navigator.clipboard.readText = async () => { throw Error('denied'); };
  assert.equal(await bridge.readForPaste(), 'fallback');
  assert.equal(bridgeModule.isDefaultPut(editor.cm, 'p'), true);
  editor.cm.state.vim.inputState.registerName = 'a';
  assert.equal(bridgeModule.isDefaultPut(editor.cm, 'p'), false);
  editor.cm.state.vim.inputState.registerName = '+';
  assert.equal(bridgeModule.isDefaultPut(editor.cm, 'p'), false);
  editor.cm.state.vim.inputState.registerName = undefined;
  editor.cm.state.vim.inputState.keyBuffer = ['2'];
  assert.equal(bridgeModule.isDefaultPut(editor.cm, 'P'), true);
  editor.cm.state.vim.inputState.keyBuffer = ['g'];
  assert.equal(bridgeModule.isDefaultPut(editor.cm, 'p'), false);
});

test('queued keys retain order and discard paste after session change', async () => {
  let resolveRead;
  navigator.clipboard = { readText: () => new Promise(resolve => { resolveRead = resolve; }), writeText: async () => {} };
  const editor = view();
  const bridge = bridgeModule.initClipboardBridge(editor);
  const queue = queueModule.createVimKeyQueue();
  let active = true;
  const press = key => ({ key, view: editor, bridge, valid: () => active, after: () => {} });
  queue(press('p'));
  queue(press('j'));
  assert.deepEqual(editor.cm.handled, []);
  resolveRead('external copy');
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.deepEqual(editor.cm.handled.map(([key]) => key), ['p', 'j']);
  queue(press('p'));
  active = false;
  resolveRead('other tab');
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.deepEqual(editor.cm.handled.map(([key]) => key), ['p', 'j']);
});

test('local yank/delete keeps its register while clipboard write is pending', async () => {
  let finishWrite;
  navigator.clipboard = {
    readText: async () => 'older system text',
    writeText: () => new Promise(resolve => { finishWrite = resolve; }),
  };
  const bridge = bridgeModule.initClipboardBridge(view());
  for (const operator of ['yank', 'delete', 'change']) {
    bridgeRegisterController.pushText(undefined, operator, `local ${operator}`);
    assert.equal(await bridge.readForPaste(), 'fallback');
    assert.equal(bridgeRegisterController.unnamedRegister.value, `local ${operator}`);
    finishWrite();
    await new Promise(resolve => setTimeout(resolve, 0));
  }
});
