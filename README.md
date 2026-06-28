# Wind

A vim-driven file workstation for Windows, built with Tauri 2.0 (Rust backend + Svelte 5 frontend).

Inspired by [Yazi](https://github.com/sxyazi/yazi), Wind combines a three-column file browser, embedded terminal, code editor, and multi-format preview into a keyboard-first workflow.

## Features

- **Three-column layout** — parent directory | current directory | preview/editor, adjustable with `:ratio X:Y:Z`
- **Vim keybindings** — `j/k` navigation, `gg/G`, `h/l` panel switching, Normal/Insert modes in the terminal
- **File preview** — text (Shiki syntax highlight), Markdown (KaTeX + Mermaid), images (turbojpeg), PDF, video (ffmpeg thumbnails + HTTP streaming), JSON, archives (zip), directories
- **Fullscreen viewers** — image viewer (zoom/pan/rotate), PDF viewer (text search), video player
- **Floating terminal** — ConPTY-powered, supports PowerShell, CMD, Git Bash with shell integration (OSC 133/OSC 7)
- **Multi-tab system** — Yazi-style `t` prefix keybindings for tab management
- **Code editor** — CodeMirror 6 with Vim mode, 15+ language syntax highlighting, fullscreen editing
- **Command palette** — `:` to open, `:cd <path>`, `:e <path>`, `:ratio X:Y:Z`, Tab path completion
- **File search** — `Ctrl+P` fuzzy search across the current directory
- **Content zoom** — `Ctrl+=`/`Ctrl+-`/`Ctrl+0` and `Ctrl+wheel` for panel content scaling
- **Theme system** — dark/light mode with Gruvbox-inspired design
- **Chinese IME** — full keyboard navigation support with IME input

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/) stable
- [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows)

## Quick Start

```bash
# Install dependencies
npm install

# Start development
npm run tauri dev

# Production build
npm run tauri build
```

## Keyboard Shortcuts

### Global
| Key | Action |
|---|---|
| `Ctrl+\`` | Toggle floating terminal |
| `:` | Open command palette |
| `Ctrl+P` | File search |

### Directory Panels
| Key | Action |
|---|---|
| `j` / `k` | Navigate down / up |
| `gg` / `G` | Jump to top / bottom |
| `Enter` | Open file or enter directory |
| `R` | Refresh directory listing |
| `h` | Switch to left panel (parent dir) |
| `l` | Switch to right panel (preview) |
| `Ctrl+W h/j/k/l` | Panel navigation |

### Floating Terminal
| Key | Action |
|---|---|
| `Ctrl+\`` | Toggle terminal |
| `Escape` | Switch to Normal mode |
| `i` | Switch to Insert mode |

### Preview / Editor Panel
| Key | Action |
|---|---|
| `Enter` | Enter edit mode |
| `e` | Open fullscreen editor |
| `Escape` | Exit edit mode |
| `Ctrl+S` | Save file |

### Fullscreen Editor
| Key | Action |
|---|---|
| `Escape` ×2 | Close editor |
| `Ctrl+S` | Save and close |
| `Ctrl+Q` | Close without saving |

### Multi-Tab (Yazi-style)
| Key | Action |
|---|---|
| `t n` | New tab |
| `t x` | Close tab |
| `t 1`–`9` | Switch to tab by number |
| `t [` / `t ]` | Previous / next tab |

### Content Zoom
| Key | Action |
|---|---|
| `Ctrl+=` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Reset zoom |
| `Ctrl+wheel` | Zoom in/out |

## Command Palette

| Command | Description |
|---|---|
| `:cd <path>` | Change directory (`.` for current, `..` for parent, no args for home) |
| `:e <path>` | Open file or directory |
| `:ratio X:Y:Z` | Set column width ratio |
| `Tab` | Auto-complete path (`cd`: dirs only, `e`: files + dirs) |

## Shell Integration

The terminal supports OSC 133 (command tracking) and OSC 7 (directory sync). To enable visual indicators (exit codes, current directory), install a prompt framework:

**PowerShell** (Oh My Posh):
```powershell
winget install JanDeDobbeleer.OhMyPosh
# Add to $PROFILE:
oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH\paradox.omp.json" | Invoke-Expression
```

**Bash / Git Bash** (Starship):
```bash
curl -sS https://starship.rs/install.sh | sh
# Add to ~/.bashrc:
eval "$(starship init bash)"
```

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | Tauri 2.0 |
| Frontend | SvelteKit + Svelte 5 (runes) + TypeScript |
| Terminal | ConPTY + xterm.js |
| Code editor | CodeMirror 6 + @replit/codemirror-vim |
| PDF | pdfjs-dist + pdfium-render |
| Image processing | turbojpeg + fast_image_resize |
| Markdown | markdown-it + KaTeX + Mermaid |
| Syntax highlight | Shiki |

## Project Structure

```
src/                         # Svelte frontend
  lib/components/            # UI components
  lib/previewers/            # File preview system (text, image, PDF, video, etc.)
  lib/stores/                # State management
  routes/                    # SvelteKit routes
src-tauri/                   # Rust backend
  src/lib.rs                 # Tauri commands (file ops, terminal, neovim)
  src/terminal/mod.rs        # ConPTY terminal implementation
  src/neovim/mod.rs          # Embedded Neovim (msgpack-rpc)
```

## License

MIT
