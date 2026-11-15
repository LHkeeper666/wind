## Context

`PreviewEditor.svelte` sends most text files directly to the editor through `isDirectEditorFile()`. Markdown is exempted, routed through `PreviewRouter`, and provides `TocHeading` data to the existing outline. RST currently has no parser or previewer. The change spans classification, parsing, rendering, styling, and outline state.

## Goals / Non-Goals

**Goals:**
- Preview ordinary `.rst` documents without an external Python or system installation.
- Preserve the current source editing action and per-tab preview behavior.
- Provide a safe, useful fallback when a document contains unsupported RST constructs.

**Non-Goals:**
- Full Docutils or Sphinx compatibility, including Sphinx roles, directives, cross-document references, and build configuration.
- Executing RST directives or fetching remote resources during parsing.

## Decisions

1. Add an `RstPreviewer` before the text fallback in `PreviewRouter`, and exempt `.rst` from direct-editor classification. Reuse the router's staging and stale-render checks. Alternative: convert RST to Markdown first; that loses RST-specific structure and source positions.
2. Parse RST into a bounded intermediate document structure, then render escaped text and allowlisted elements. Keep block and inline parsing separate so section levels, source lines, tables, and directives can be handled explicitly. Evaluate an in-process parser dependency during implementation; accept it only if it covers the specified subset, preserves needed source positions, and keeps output sanitizable. Otherwise implement the bounded subset in the frontend. Alternative: invoke Python Docutils; this would add a runtime dependency to the packaged Windows app.
3. Treat heading adornment styles in order of first appearance as section levels and produce `TocHeading` entries with zero-based source lines. Make the existing outline state available to RST documents by generalizing the current Markdown-only gate. Alternative: build a separate RST outline; that duplicates keyboard navigation and tab state handling.
4. Render local image references through the same path-resolution and Tauri asset-loading pattern used by Markdown. Do not interpret raw HTML, executable directives, or unsafe URI schemes. Show unsupported directives as escaped source or a readable placeholder, with the rest of the document still visible.
5. Reuse shared document typography where appropriate, with RST-specific selectors for structures that differ. Keep parsing independent of Markdown's parser and caches.

## Risks / Trade-offs

- [Risk] RST grammar has context-sensitive indentation and several table forms. → Mitigation: define supported constructs with representative fixtures, and preserve readable source for unsupported forms.
- [Risk] Parsed links or images could inject active content. → Mitigation: escape text and attributes, allowlist URL schemes, and use existing local asset resolution.
- [Risk] Large or malformed files could stall the UI. → Mitigation: bound parsing work and follow existing preview size behavior; test malformed nesting and rapid tab switches.
- [Risk] Markdown-named outline state may obscure its new shared role. → Mitigation: rename or wrap only the necessary gates during implementation, preserving existing Markdown behavior.

## Migration Plan

Ship as a frontend change. Existing `.rst` files open in preview after upgrade, and source remains reachable via the editor action. Rollback consists of reverting the classification and router registration alongside the new previewer.

## Open Questions

- Resolved: implement the specified subset in TypeScript. A generic converter would obscure source lines and produce HTML requiring a second sanitization layer; Python Docutils would add a packaged runtime dependency. The bounded parser keeps source lines and HTML policy explicit.
