# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wind is a vim-driven file workstation for Windows, built with Tauri 2.0 (Rust backend + Svelte 5 frontend).

## Commands

```bash
# Development
npm run tauri dev          # Start dev server + Rust backend
npx svelte-check           # TypeScript/Svelte type checking

# Rust only (from src-tauri/)
cargo check                # Type check Rust code
cargo build                # Build Rust backend

# Production
npm run tauri build        # Build release binary
```

## Architecture

**Frontend (SvelteKit + Svelte 5 Runes)**
- `src/lib/components/` - UI components using Svelte 5 runes syntax (`$state`, `$derived`, `$props()`, `$effect`)
- `src/lib/previewers/` - File preview system (TextPreviewer with Shiki, MarkdownPreviewer, ImagePreviewer, JsonPreviewer)
- `src/lib/stores/` - Svelte stores for theme management
- `src/routes/+layout.svelte` - Root layout imports global CSS variables

**Backend (Rust + Tauri 2)**
- `src-tauri/src/lib.rs` - Main Tauri commands (file ops, terminal, neovim)
- `src-tauri/src/terminal/mod.rs` - ConPTY terminal with stdout event emission
- `src-tauri/src/neovim/mod.rs` - Embedded Neovim via msgpack-rpc

**Communication Pattern**
- Frontend → Backend: `invoke('command_name', { params })` from `@tauri-apps/api/core`
- Backend → Frontend: `app_handle.emit('event-name', data)` + `listen('event-name', callback)` from `@tauri-apps/api/event`

## Key Design Decisions

- **Svelte 5 Runes**: All components use `$state()` for reactive state, `$derived()` for computed values, `$props()` for component props. Never use `export let` (Svelte 4 syntax).
- **Three-Column Layout**: Default layout shows parent directory (1fr), current directory (1fr), and preview/editor (3fr). Inspired by Yazi file manager.
- **Terminal Mode System**: Terminal has Insert mode (keys go to shell) and Normal mode (keys for navigation). Escape switches to Normal, 'i' switches to Insert. Implemented via overlay div that blocks xterm.js input.
- **Panel Navigation**: `h/l` to switch between panels when in directory panels. `j/k` to navigate within panel.
- **Theme System**: CSS variables in `src/lib/styles/themes.css` with dark/light theme support.
- **Preview Router**: Pluggable previewer architecture - each previewer implements `match(filePath)` and `render(content, container)`.
- **CodeMirror 6**: Full code editor with syntax highlighting, line numbers, and language support.
- **Floating Terminal**: Default hidden, toggle with `Ctrl+``. Supports PowerShell, CMD, Git Bash.

## Keyboard Shortcuts

### Global
- `Ctrl+`` - Toggle floating terminal (works from any focus state)
- `:` - Open command palette
- `Ctrl+P` - File search

### Floating Terminal
- `Ctrl+`` - Toggle terminal visibility
- `Escape` - Switch from Insert to Normal mode
- `i` - Switch from Normal to Insert mode

### Directory Panels (Parent/Current)
- `j/k` - Navigate up/down
- `gg/G` - Jump to top/bottom
- `Enter` - Open file or enter directory
- `R` - Refresh directory
- `h` - Switch to left column (parent)
- `l` - Switch to right column (preview)

### Preview/Editor Panel
- `Enter` - Enter edit mode
- `e` - Open fullscreen editor
- `Escape` - Exit edit mode
- `Ctrl+S` - Save file

### Fullscreen Editor
- `Escape` twice - Close editor
- `Ctrl+S` - Save and close
- `Ctrl+Q` - Close without saving

### Command Palette
- `ratio X:Y:Z` - Set column ratios (e.g., `ratio 1:2:2`)
- `cd path` - Change current directory (supports absolute/relative paths, `..`, no args = home)
- `e path` - Open file in preview or navigate to directory; no args = refresh current directory
- `Tab` - Auto-complete paths (cd: directories only, e: files + directories)

## Shell Integration

Terminal supports shell integration via OSC 133 (command tracking) and OSC 7 (directory sync).

### Enable Shell Integration

**PowerShell** (requires Oh My Posh or similar):
```powershell
# Install oh-my-posh
winget install JanDeDobbeleer.OhMyPosh

# Add to $PROFILE
oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH\paradox.omp.json" | Invoke-Expression
```

**Bash/Zsh** (requires Starship or similar):
```bash
# Install starship
curl -sS https://starship.rs/install.sh | sh

# Add to ~/.bashrc or ~/.zshrc
eval "$(starship init bash)"
```

**Git Bash**:
Add starship to `~/.bashrc`:
```bash
eval "$(starship init bash)"
```

### Features
- Command exit code display (non-zero shown in red)
- Current directory sync in terminal header
- Running command indicator

## Rust Dependencies

- `tauri 2` - Desktop framework
- `windows` - ConPTY terminal support
- `rmpv` / `rmp-serde` - Neovim msgpack communication
- `dirs` - Home directory detection

## Environment Notes

- Rust toolchain configured with USTC mirror (`mirrors.ustc.edu.cn`) for China network
- Cargo config at `~/.cargo/config.toml` sets crates.io registry mirror
- Windows-specific: uses PowerShell, CMD, Git Bash as terminal shells
