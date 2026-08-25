## Context

Markdown files are loaded as preview content first. Pressing `e` switches `PreviewEditor` from `global-normal` to `editor-normal`, hides the preview DOM, shows `.editor-area`, then schedules CodeMirror creation on `requestAnimationFrame`. `initEditor()` immediately creates the `EditorView`, requests one measure, and may call `scrollIntoView` to center the Markdown line inferred from the preview.

In a packaged Tauri WebView, frame timing differs enough that the editor container may still have stale or zero layout when CodeMirror performs its first measurement. The editor state contains the document, but the virtual viewport is invalid, producing a blank editor or a split view where later input only repaints part of the content.

## Goals / Non-Goals

Goals:

- Ensure Markdown `e` transitions create or activate CodeMirror only after the editor host is visible and has non-zero dimensions.
- Request CodeMirror measurement after the browser has committed visibility/layout changes.
- Apply initial cursor/scroll restoration after the stable measurement point.
- Keep direct-edit code files, tab session reuse, Vim overlay behavior, and Markdown preview rendering intact.

Non-Goals:

- No Rust file loading or encoding changes.
- No new editor engine or dependency.
- No change to Markdown preview syntax highlighting, TOC, KaTeX, Mermaid, or image handling.

## Decisions

### Defer Initial Editor Measurement Until Layout Is Stable

Introduce a small helper that waits for Svelte DOM updates and animation frames, then verifies that the editor container and active session host have non-zero bounding boxes. CodeMirror creation still happens only when the existing `readyTextContent` snapshot matches the active tab/path/generation, but scroll restoration and focus are deferred until layout is stable.

If layout is not ready on the first frame, schedule a bounded retry rather than using an arbitrary long timeout. This keeps the fix deterministic while covering grid transitions, preview expansion, terminal resize, and tab switch restore.

### Centralize Active Editor Refresh

Add an `refreshActiveEditorLayout()` helper that calls `view.requestMeasure()` for the active session after mode changes, session activation, and resize observer callbacks. It should also clamp scroll after measurement, preserving the existing scroll safety behavior.

### Preserve Preview Line Targeting

`getVisibleLine()` remains the source for the Markdown target line. The change is only when the resulting editor scroll is applied: after the editor has a stable size, not immediately after `new EditorView`.

## Risks / Trade-offs

- A bounded retry can delay first editor focus by one or two frames, but avoids a broken viewport.
- Running extra `requestMeasure()` calls has small cost, but only happens around editor activation/layout changes.
- The fix depends on DOM size checks rather than build-mode assumptions, so it should cover both dev and packaged runs.
