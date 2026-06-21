## ADDED Requirements

### Requirement: Neovim process management
The system SHALL manage a Neovim process in embedded headless mode, communicating via msgpack-rpc over stdin/stdout.

#### Scenario: Start Neovim
- **WHEN** Wind application starts
- **THEN** a Neovim process is spawned with --embed --headless flags

#### Scenario: Neovim process crash recovery
- **WHEN** the Neovim process crashes unexpectedly
- **THEN** the system detects the crash, restarts Neovim, and notifies the user

#### Scenario: Graceful shutdown
- **WHEN** Wind application closes
- **THEN** the Neovim process is gracefully shut down

### Requirement: Canvas rendering engine
The system SHALL render Neovim's UI using HTML Canvas, processing redraw events from the Neovim UI protocol.

#### Scenario: Render text grid
- **WHEN** Neovim sends grid_line redraw events
- **THEN** the Canvas renderer updates the corresponding cells with correct characters and highlight colors

#### Scenario: Render cursor
- **WHEN** Neovim sends grid_cursor_goto events
- **THEN** the cursor is drawn at the specified position with the correct shape (block/vertical/horizontal)

#### Scenario: Render floating windows
- **WHEN** Neovim sends win_float_pos events for completion menus, hover docs, or diagnostics
- **THEN** floating windows are rendered as separate layers above the main grid

#### Scenario: Handle CJK characters
- **WHEN** Neovim sends grid_line events containing double-width CJK characters
- **THEN** the renderer correctly handles the double-width occupation and skips placeholder cells

### Requirement: Keyboard input forwarding
The system SHALL capture keyboard events and translate them to Neovim key sequences.

#### Scenario: Normal key input
- **WHEN** user presses 'a' in normal mode
- **THEN** the key 'a' is sent to Neovim via nvim_input

#### Scenario: Special key input
- **WHEN** user presses Escape
- **THEN** the key sequence '<Esc>' is sent to Neovim

#### Scenario: Ctrl combination
- **WHEN** user presses Ctrl+w
- **THEN** the key sequence '<C-w>' is sent to Neovim

#### Scenario: Prevent browser default
- **WHEN** user presses Tab, Ctrl+n, or other browser-shortcut keys
- **THEN** the browser default behavior is prevented and the key is forwarded to Neovim

### Requirement: Chinese input method support
The system SHALL properly handle Chinese IME input in Neovim insert mode.

#### Scenario: IME composition start
- **WHEN** user activates IME and begins typing pinyin
- **THEN** the system enters composition mode and does not forward intermediate keystrokes to Neovim

#### Scenario: IME composition update
- **WHEN** user is composing characters via IME
- **THEN** a preedit text overlay is displayed at the cursor position showing the current composition

#### Scenario: IME composition end
- **WHEN** user confirms the IME input (e.g., selects a character)
- **THEN** the final composed text is sent to Neovim and the preedit overlay is removed

### Requirement: Neovim UI protocol support
The system SHALL support the following Neovim UI protocol extensions: ext_linegrid, ext_multigrid, ext_cmdline, ext_popupmenu, ext_tabline, ext_messages.

#### Scenario: Multi-grid rendering
- **WHEN** Neovim creates multiple grids (windows, splits, floating windows)
- **THEN** each grid is rendered independently with its own position and size

#### Scenario: External command line
- **WHEN** Neovim sends cmdline_show events
- **THEN** the command line is rendered as a separate input area (not embedded in the grid)

#### Scenario: External popup menu
- **WHEN** Neovim sends popupmenu_show events
- **THEN** the completion menu is rendered as a floating popup with selectable items

### Requirement: Neovim buffer synchronization
The system SHALL synchronize Neovim buffer state with other Wind components.

#### Scenario: Buffer enter event
- **WHEN** user opens a file in Neovim (BufEnter event)
- **THEN** the file tree highlights the corresponding file and the preview panel updates

#### Scenario: Buffer modified event
- **WHEN** user modifies a buffer in Neovim
- **THEN** the preview panel updates if the file type supports live preview (e.g., Markdown)
