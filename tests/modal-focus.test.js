// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';

class FakeNode {
  constructor(parent = null) {
    this.parent = parent;
    this.isConnected = true;
    this.disabled = false;
    this.children = [];
    if (parent) parent.children.push(this);
  }
  contains(node) {
    for (let current = node; current; current = current.parent) {
      if (current === this) return true;
    }
    return false;
  }
  hasAttribute(name) { return name === 'disabled' && this.disabled; }
  getClientRects() { return this.isConnected ? [{}] : []; }
  focus() {
    document.activeElement = this;
    document.dispatch('focusin', { target: this });
  }
  querySelectorAll() { return this.children.filter(child => child.tabbable); }
}
globalThis.Node = FakeNode;
globalThis.HTMLElement = FakeNode;

class FakeDocument {
  activeElement = null;
  listeners = new Map();
  panels = [];
  addEventListener(name, fn) {
    const items = this.listeners.get(name) ?? new Set();
    items.add(fn);
    this.listeners.set(name, items);
  }
  removeEventListener(name, fn) { this.listeners.get(name)?.delete(fn); }
  dispatch(name, event) { for (const fn of this.listeners.get(name) ?? []) fn(event); }
  querySelectorAll() { return this.panels; }
}
globalThis.document = new FakeDocument();

const { registerModal, hasActiveModal } = await import('../src/lib/utils/modal-focus.ts');

test('ordinary modal traps external focus, cycles Tab, and restores focus', async () => {
  const panel = new FakeNode();
  document.panels = [panel];
  panel.focus();
  const root = new FakeNode();
  const first = new FakeNode(root);
  const last = new FakeNode(root);
  first.tabbable = last.tabbable = true;
  const close = registerModal(root, first);
  assert.equal(document.activeElement, first);
  panel.focus();
  assert.equal(document.activeElement, first);
  last.focus();
  const tab = { key: 'Tab', shiftKey: false, preventDefault() { this.prevented = true; } };
  document.dispatch('keydown', tab);
  assert.equal(tab.prevented, true);
  assert.equal(document.activeElement, first);
  const reverse = { ...tab, shiftKey: true };
  document.dispatch('keydown', reverse);
  assert.equal(document.activeElement, last);
  close();
  await Promise.resolve();
  assert.equal(document.activeElement, panel);
  assert.equal(hasActiveModal(), false);
});

test('stacked and custom-Tab modals restore the remaining dialog', async () => {
  const panel = new FakeNode();
  panel.focus();
  const lower = new FakeNode();
  const lowerInput = new FakeNode(lower);
  const closeLower = registerModal(lower, lowerInput);
  const upper = new FakeNode();
  const upperInput = new FakeNode(upper);
  const closeUpper = registerModal(upper, upperInput, 'custom');
  const tab = { key: 'Tab', preventDefault() { this.prevented = true; } };
  document.dispatch('keydown', tab);
  assert.equal(tab.prevented, undefined);
  lowerInput.focus();
  assert.equal(document.activeElement, upperInput);
  closeUpper();
  await Promise.resolve();
  assert.equal(document.activeElement, lowerInput);
  closeLower();
  await Promise.resolve();
  assert.equal(document.activeElement, panel);
});

test('removed opener falls back to a visible panel', async () => {
  const panel = new FakeNode();
  document.panels = [panel];
  const opener = new FakeNode();
  opener.focus();
  const root = new FakeNode();
  const close = registerModal(root);
  opener.isConnected = false;
  close();
  await Promise.resolve();
  assert.equal(document.activeElement, panel);
});
