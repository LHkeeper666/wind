## Context

Wind's preview system uses pluggable `Previewer` implementations registered in `PreviewRouter`. Each previewer implements `match(filePath)` and `render(content, container)`. The `PreviewEditor.loadFile()` routes files to either preview mode or direct editor mode based on file type.

.ipynb is JSON with structure:
```json
{
  "cells": [
    { "cell_type": "markdown", "source": ["# Title\n", "text..."] },
    { "cell_type": "code", "source": ["import numpy\n"], "outputs": [...] },
    { "cell_type": "raw", "source": ["raw content"] }
  ],
  "metadata": { "kernelspec": { "language": "python" }, ... },
  "nbformat": 4, "nbformat_minor": 2
}
```

Current behavior: .ipynb goes to `codeFileDirectEdit` → CodeMirror shows raw JSON directly. No preview.

## Goals / Non-Goals

**Goals:**
- Parse .ipynb JSON and render cells as readable HTML
- Markdown cells: render with markdown-it (headings, lists, code blocks, images)
- Code cells: syntax highlight with Shiki, show outputs (text/plain, image/png)
- Raw cells: show as plain preformatted text
- Cell type badges for visual distinction
- Press `e` to edit raw JSON in CodeMirror
- j/k scroll navigation works in preview

**Non-Goals:**
- Execute code cells (no Python kernel integration)
- Full MarkdownPreviewer parity (no KaTeX, Mermaid, TOC for notebook cells)
- Streaming output display
- In-place cell editing in preview mode
- .ipynb output images as separate files (inline base64 only)
- Plotly/Bokeh/HTML output rendering beyond plain text/images

## Decisions

### 1. New IpynbPreviewer implements Previewer interface

**Decision**: Create a dedicated `IpynbPreviewer` class rather than extending `JsonPreviewer` or `MarkdownPreviewer`.

**Rationale**: Notebook rendering is fundamentally different from both JSON tree view and standalone markdown. The cell structure (markdown/code/output interleaving) requires a custom HTML layout. Composition (using standalone markdown-it + Shiki) is cleaner than inheritance.

### 2. Reuse Shiki for code highlighting, standalone markdown-it for markdown

**Decision**: Create a new `MarkdownIt` instance inside `IpynbPreviewer` for markdown cell rendering. Use the existing Shiki `createHighlighter` for code highlighting.

**Rationale**: The `MarkdownPreviewer`'s markdown-it instance is heavily customized (KaTeX, Mermaid, Obsidian wikilinks, image caching). Notebook markdown cells are typically simple — no math, no diagrams. A fresh markdown-it without plugins is lighter and avoids caching complexity. For code, Shiki is already a dependency and provides the best syntax highlighting.

**Alternative considered**: Reuse `MarkdownPreviewer`'s `md` instance. Rejected because it would couple the two classes and notebook markdown doesn't need the extra plugins.

### 3. CSS styling matches existing preview aesthetic

**Decision**: Style notebook cells using the project's CSS variables (`--bg-primary`, `--bg-secondary`, `--border`, `--accent`). Code cell input area gets a subtle left border; output area gets a different background shade.

**Rationale**: Consistent with the overall Wind theme system. Dark/light theme support comes for free via CSS variables.

### 4. Cell output: text/plain and image/png only

**Decision**: Support only `text/plain` (multi-line text) and `image/png` (base64 → `<img>`). Ignore `text/html`, `application/javascript`, `application/vnd.plotly`, etc.

**Rationale**: Text and images cover 95%+ of real-world notebook outputs. HTML/JS outputs are security risks (XSS). Complex outputs (Plotly, widgets) require libraries we don't have and aren't worth bundling.

### 5. PreviewEditor routing: treat .ipynb like .md/.json

**Decision**: Add `ext !== 'ipynb'` to the condition in `loadFile()` that sets `codeFileDirectEdit = true`. Also add `E` (fullscreen editor) support for .ipynb.

**Rationale**: Minimal code change. The existing routing logic already handles this pattern for markdown and json. .ipynb is a preview-first file type.

### 6. Keyboard shortcut: `Ctrl+Enter` in preview to toggle code cell output

**Decision**: Add `Ctrl+Enter` handler in `PreviewEditor.svelte` that toggles output visibility for the code cell nearest to current scroll position.

**Rationale**: Notebook outputs can be long — users need a way to collapse them. Using the nearest-visible-cell heuristic avoids complex cell indexing.

## Visual Design

```
┌─────────────────────────────────────────────────────┐
│ PREVIEW                        [ipynb]         ☰    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─ Markdown ────────────────────────────────────┐  │
│  │  # Data Analysis                              │  │
│  │  This notebook explores the dataset...        │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌─ Code [python] ───────────────────────────────┐  │
│  │  In [1]:                                      │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │ import pandas as pd                     │  │  │
│  │  │ df = pd.read_csv('data.csv')            │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  │                                               │  │
│  │  Out [1]:                                     │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │    col_a  col_b  col_c                  │  │  │
│  │  │ 0   1.2    3.4    5.6                   │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌─ Code [python] ───────────────────────────────┐  │
│  │  In [2]:                                      │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │ plt.plot(df.col_a)                      │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  │                                               │  │
│  │  Out [2]:  [image/png]                        │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │            [chart image]                 │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Risks / Trade-offs

**[Risk] Large notebooks (>100 cells) may be slow to render** → Mitigation: Parse and build HTML in one pass. Shiki highlighting runs async per code cell, non-blocking. If perf is an issue, add a cell count limit or virtual scrolling later.

**[Risk] Malformed .ipynb JSON** → Mitigation: Wrap parse in try/catch, fall back to plain text display with error message.

**[Risk] `source` field is array of strings, not single string** → Mitigation: Join with `''` (no separator). This is the standard representation; each string in the array is typically one line.
