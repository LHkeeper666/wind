## Why

Wind currently opens `.rst` files directly in the text editor, so users cannot read reStructuredText documents as formatted pages. Add a preview for common RST documents while keeping source editing available.

## What Changes

- Open `.rst` files in the preview pane by default, with the existing editor action available for source changes.
- Render common RST structures: section headings, paragraphs, inline emphasis and code, lists, links, literal and code blocks, simple and grid tables, images, and common admonitions.
- Show RST section headings in the existing outline sidebar with navigation to the corresponding source line.
- Handle unsupported directives and malformed input without executing document content or breaking the preview.
- Leave Sphinx-specific directives, roles, and cross-document references outside the initial scope.

## Capabilities

### New Capabilities

- `rst-preview`: Rendering and safe fallback behavior for common reStructuredText documents.

### Modified Capabilities

- `file-type-detection`: Classify `.rst` as a text document that opens in preview by default.
- `outline-sidebar`: Make the existing document outline available for RST sections.

## Impact

Frontend file classification, preview routing, preview styling, document outline state, and associated tests. The initial scope should work without requiring Python or a separately installed system renderer.
