// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';
import { parseRst } from '../src/lib/previewers/rst-parser.ts';
import { highlightRstCode } from '../src/lib/previewers/rst-highlighter.ts';

test('renders common blocks and preserves heading source lines', () => {
  const source = `Guide
=====

Intro with **bold**, *emphasis*, and \`\`code\`\`.

Details
-------

- one
- two

Example::

   print(1)

.. code-block:: python

   print(2)

.. note::

   Remember this.

.. image:: images/logo.png
`;
  const doc = parseRst(source);
  assert.equal(doc.headings[0].line, 0);
  assert.equal(doc.headings[0].children[0].line, 5);
  for (const fragment of ['<strong>bold</strong>', '<em>emphasis</em>', '<code>code</code>', '<ul>', 'print(1)', 'language-python', 'rst-note', 'data-rst-src="images/logo.png"']) {
    assert.ok(doc.html.includes(fragment), fragment);
  }
});

test('renders simple and grid tables', () => {
  const simple = parseRst('====   =====\nName   Value\n====   =====\nA      1\n====   =====');
  const grid = parseRst('+----+-----+\n| A  | B   |\n+----+-----+');
  assert.ok(simple.html.includes('<table>'));
  assert.ok(grid.html.includes('<table>'));
});

test('keeps unsafe or unsupported content inert and readable', () => {
  const doc = parseRst('`bad <javascript:alert(1)>`_\n\n.. raw:: html\n\n   <script>alert(1)</script>\n\nAfter.');
  assert.ok(!doc.html.includes('href="javascript:'));
  assert.ok(doc.html.includes('&lt;script&gt;'));
  assert.ok(doc.html.includes('After.'));
});

test('resolves ordinary reference links and rejects unsafe references', () => {
  const doc = parseRst('See docs_ and bad_.\n\n.. _docs: https://example.com\n.. _bad: javascript:alert(1)');
  assert.ok(doc.html.includes('href="https://example.com"'));
  assert.ok(!doc.html.includes('href="javascript:'));
});

test('dedents code content and hides RST comments', () => {
  const doc = parseRst('.. code-block:: python\n\n   if True:\n       print(1)\n\n.. internal comment\n   hidden detail\n\nVisible.');
  assert.ok(doc.html.includes('class="language-python"'));
  assert.ok(doc.html.includes('if True:\n    print(1)'));
  assert.ok(!doc.html.includes('internal comment'));
  assert.ok(!doc.html.includes('hidden detail'));
  assert.ok(doc.html.includes('Visible.'));
});

test('Shiki highlights a supported language and falls back for unknown languages', async () => {
  const highlighted = await highlightRstCode('const answer = 42', 'javascript');
  const fallback = await highlightRstCode('plain text', 'unknown-rst-language');
  assert.ok(highlighted.includes('shiki'));
  assert.ok(highlighted.includes('<span'));
  assert.ok(fallback.includes('plain text'));
});

test('preserves first-line indentation in code directives and literal blocks', () => {
  const diagram = '    +---------------------+\n| Infrastructure      |\n+---------------------+';
  const directive = parseRst('.. code-block:: text\n\n        +---------------------+\n    | Infrastructure      |\n    +---------------------+');
  const literal = parseRst('Diagram::\n\n        +---------------------+\n    | Infrastructure      |\n    +---------------------+');
  assert.ok(directive.html.includes(diagram), directive.html);
  assert.ok(literal.html.includes(diagram), literal.html);
});

test('code directive options do not strip indentation from the first code line', () => {
  const doc = parseRst('.. code-block:: text\n   :linenos:\n\n       first\n   second');
  assert.ok(doc.html.includes('    first\nsecond'), doc.html);
});
