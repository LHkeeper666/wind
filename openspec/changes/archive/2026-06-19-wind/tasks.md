## 1. Project Setup

- [x] 1.1 Initialize Tauri 2.0 project with Svelte frontend template
- [x] 1.2 Configure project structure (src-tauri for Rust, src for Svelte)
- [x] 1.3 Add core dependencies: xterm.js, @xterm/addon-fit, @xterm/addon-web-links
- [x] 1.4 Add Shiki, markdown-it, KaTeX, Mermaid, pdf.js dependencies
- [x] 1.5 Set up basic Tauri IPC communication (invoke/emit) between frontend and Rust
- [x] 1.6 Create main window with default panel layout structure

## 2. File Tree Component

- [x] 2.1 Implement FileTree Svelte component with recursive directory rendering
- [x] 2.2 Implement Rust backend: read directory entries (files, folders) via Tauri command
- [x] 2.3 Add vim key navigation (j/k/gg/G) for file tree selection
- [x] 2.4 Add Enter key to open file in editor or expand/collapse folder
- [x] 2.5 Add h/l keys for collapse/expand folders
- [x] 2.6 Add mouse support: click to select, double-click to open, scroll wheel
- [x] 2.7 Implement file search/filter with / key activation
- [x] 2.8 Implement file operations: delete (dd), rename (r), create (o), copy (yy), paste (p)
- [x] 2.9 Add file tree to editor/terminal path synchronization

## 3. Terminal Component

- [x] 3.1 Implement Terminal Svelte component with xterm.js
- [x] 3.2 Implement Rust backend: ConPTY process management (spawn, resize, input/output)
- [x] 3.3 Connect xterm.js frontend to ConPTY backend via Tauri IPC
- [x] 3.4 Implement terminal resize handling (sync xterm.js size to ConPTY)
- [x] 3.5 Add shell type selection (cmd, PowerShell, Git Bash, WSL)
- [x] 3.6 Add mouse support in terminal (click, scroll)
- [x] 3.7 Implement terminal panel toggle (show/hide with state preservation)

## 4. Neovim Editor - Core

- [x] 4.1 Implement Rust backend: Neovim process spawning (--embed --headless)
- [x] 4.2 Implement msgpack-rpc client in Rust (encode/decode, request/response/notification)
- [x] 4.3 Send nvim_ui_attach with ext_linegrid, ext_multigrid, ext_cmdline, ext_popupmenu options
- [x] 4.4 Implement redraw event parser: parse grid_resize, grid_line, grid_cursor_goto, grid_scroll, flush events
- [x] 4.5 Implement highlight attribute parser: parse hl_attr_define, default_colors_set events
- [x] 4.6 Create NeovimCanvas Svelte component with Canvas rendering engine
- [x] 4.7 Implement grid cell rendering: background colors, text with highlight, CJK double-width handling
- [x] 4.8 Implement cursor rendering: block/vertical/horizontal shapes, blink animation
- [x] 4.9 Implement multi-grid rendering: separate Canvas layers for floating windows

## 5. Neovim Editor - Input

- [x] 5.1 Implement keyboard input handler: DOM KeyboardEvent to Neovim key sequence mapping
- [x] 5.2 Handle special keys (Escape, Enter, Tab, Backspace, arrows, function keys)
- [x] 5.3 Handle modifier combinations (Ctrl+, Alt+, Shift+)
- [x] 5.4 Prevent browser default behavior for intercepted keys
- [x] 5.5 Implement IME composition handling: compositionstart/update/end events
- [x] 5.6 Implement preedit text overlay display at cursor position during IME composition
- [x] 5.7 Implement mouse input forwarding to Neovim (click, scroll)

## 6. Neovim Editor - Advanced UI

- [x] 6.1 Implement ext_cmdline: render command line as separate input area
- [x] 6.2 Implement ext_popupmenu: render completion menu as floating popup
- [x] 6.3 Implement ext_tabline: display tab line in Wind's tab bar
- [x] 6.4 Implement ext_messages: display Neovim messages in Wind's message area
- [x] 6.5 Implement mode_info_set handling: update cursor shape per mode (normal/insert/visual)
- [x] 6.6 Implement Neovim crash detection and auto-restart

## 7. File Preview - Text/Code

- [x] 7.1 Define Previewer interface (match, render, dispose) in frontend
- [x] 7.2 Implement PreviewRouter: file type detection and previewer dispatch
- [x] 7.3 Implement TextPreviewer with Shiki syntax highlighting
- [x] 7.4 Add support for 50+ common languages (JS, TS, Python, Java, Go, Rust, C, C++, etc.)
- [x] 7.5 Implement virtual scrolling for large text files (>1000 lines)
- [x] 7.6 Implement fallback: plain text or hex dump for unrecognized file types

## 8. File Preview - Markdown

- [x] 8.1 Implement MarkdownPreviewer with markdown-it parser
- [x] 8.2 Add KaTeX plugin for math formula rendering
- [x] 8.3 Add Mermaid plugin for diagram rendering
- [x] 8.4 Add Shiki integration for code block syntax highlighting
- [x] 8.5 Implement Markdown live preview: update preview on Neovim buffer change
- [x] 8.6 Implement scroll sync between editor and preview

## 9. File Preview - PDF, Image, Data

- [x] 9.1 Implement PdfPreviewer with pdf.js: render pages as canvas elements
- [x] 9.2 Implement PDF page navigation (scroll, page number jump)
- [x] 9.3 Implement PDF zoom controls
- [x] 9.4 Implement ImagePreviewer: display PNG/JPG/GIF/SVG/WebP with native img element
- [x] 9.5 Add image zoom (mouse wheel) and pan (mouse drag) controls
- [x] 9.6 Display image info (dimensions, format, file size)
- [x] 9.7 Implement JsonPreviewer: collapsible tree view and raw syntax-highlighted view
- [x] 9.8 Implement CsvPreviewer: table view with header row and pagination

## 10. Panel Layout System

- [x] 10.1 Implement default layout: file tree (left), editor/preview (center), terminal (bottom)
- [x] 10.2 Implement panel focus management: Ctrl+w h/j/k/l to switch focus
- [x] 10.3 Add visual focus indicator (highlighted border for active panel)
- [x] 10.4 Implement panel resize: keyboard (Ctrl+w +/-) and mouse drag on borders
- [x] 10.5 Implement file tree ↔ editor path sync (BufEnter event)
- [x] 10.6 Implement editor ↔ preview live sync for Markdown files

## 11. Global Vim Mode & Command Palette

- [x] 11.1 Implement global keybinding layer: intercept keys before dispatching to panels
- [x] 11.2 Implement command palette: : key activation, input field, fuzzy search
- [x] 11.3 Add built-in commands: :open, :split, :terminal, :theme, :quit
- [x] 11.4 Implement fuzzy file search overlay: Ctrl+p activation, file listing, Enter to open
- [x] 11.5 Implement status bar: vim mode display, file path, encoding info
- [x] 11.6 Implement theme system: dark/light theme switching, theme persistence

## 12. Integration & Polish

- [x] 12.1 Wire all components together in the main App layout
- [x] 12.2 Implement Neovim buffer synchronization with file tree and preview
- [x] 12.3 Add keyboard shortcut configuration (keybindings.json)
- [x] 12.4 Test Chinese input method compatibility (Sogou, Microsoft Pinyin, Baidu)
- [x] 12.5 Test with common Neovim plugin configurations
- [x] 12.6 Performance optimization: startup time, memory usage, rendering performance
- [x] 12.7 Add error handling and user-friendly error messages
- [x] 12.8 Create README.md with installation instructions and usage guide
- [x] 12.9 Configure GitHub Actions CI/CD for automated builds and releases
- [x] 12.10 Package application for distribution (Windows installer)
