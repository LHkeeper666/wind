## 1. Parsing and routing

- [x] 1.1 Evaluate an in-process RST parser against the specified constructs, source-line mapping, bundle size, and safe rendering requirements; record the chosen approach in design.md.
- [x] 1.2 Add RST block and inline parsing with fixtures for headings, paragraphs, lists, links, literal and code blocks, tables, images, and common admonitions.
- [x] 1.3 Add safe rendering for parsed RST, including escaping, URL allowlisting, and readable fallback for malformed or unsupported constructs.
- [x] 1.4 Register `RstPreviewer` in the preview router and classify `.rst` as preview-first text while preserving source editing.

## 2. Preview integration

- [x] 2.1 Resolve local RST image references through the existing local asset path behavior and handle missing images.
- [x] 2.2 Apply document styling for supported RST structures in both dark and light themes.
- [x] 2.3 Emit RST section headings with source lines and connect them to the shared outline, including tab switching and cached state.

## 3. Verification

- [x] 3.1 Add tests for supported syntax, mixed heading styles, extension case, source editing, unsafe links, unsupported directives, and malformed input.
- [x] 3.2 Verify rapid file switches, relative images, and outline navigation in the preview UI. (User verified.)
- [x] 3.3 Run `npm test` and `npm run check`; resolve failures caused by this change.
- [x] 3.4 Highlight code and code-block directives, and verify the distinction between comments and directives.
- [x] 3.5 Preserve relative indentation on the first line of literal and code directive blocks, including blocks with options.
