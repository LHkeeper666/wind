## ADDED Requirements

### Requirement: Preview editor mode state machine
The PreviewEditor component SHALL maintain a three-state mode: `global-normal`, `editor-normal`, and `editor-insert`.

#### Scenario: Default mode on file open
- **WHEN** a text file is selected in the directory panel
- **THEN** the PreviewEditor enters `global-normal` mode and displays the Shiki syntax-highlighted preview

#### Scenario: Mode indicator display
- **WHEN** the PreviewEditor is in any mode
- **THEN** the panel header SHALL display the current mode as an indicator (e.g., "NORMAL", "INSERT", "PREVIEW")

### Requirement: Enter editor normal mode with e key
The PreviewEditor SHALL transition from `global-normal` to `editor-normal` when the user presses `e` on a text file.

#### Scenario: e key on text file
- **WHEN** the user presses `e` in `global-normal` mode on a text file
- **THEN** the PreviewEditor transitions to `editor-normal` mode
- **AND** a CodeMirror editor with vim keybindings is displayed
- **AND** the editor is in vim normal mode

#### Scenario: e key on non-text file
- **WHEN** the user presses `e` in `global-normal` mode on an image or other non-editable file
- **THEN** a toast message "此文件类型不支持编辑" SHALL be displayed
- **AND** the mode remains `global-normal`

#### Scenario: E key on text file
- **WHEN** the user presses `E` in `global-normal` mode on a text file
- **THEN** the fullscreen editor opens with vim mode enabled

### Requirement: Enter editor insert mode with i key
The PreviewEditor SHALL transition from `editor-normal` to `editor-insert` when the user presses `i`, as handled by CodeMirror vim.

#### Scenario: i key in editor normal
- **WHEN** the user presses `i` in `editor-normal` mode
- **THEN** the CodeMirror vim extension transitions to insert mode
- **AND** the mode indicator changes to "INSERT"

### Requirement: Exit insert mode with Escape
The PreviewEditor SHALL transition from `editor-insert` to `editor-normal` when the user presses Escape, as handled by CodeMirror vim.

#### Scenario: Escape in editor insert
- **WHEN** the user presses Escape in `editor-insert` mode
- **THEN** the CodeMirror vim extension transitions to normal mode
- **AND** the mode indicator changes to "NORMAL"

### Requirement: Save file with :w command
The PreviewEditor SHALL save the current file and remain in `editor-normal` when the user executes `:w`.

#### Scenario: :w saves and stays
- **WHEN** the user types `:w` and presses Enter in `editor-normal` mode
- **THEN** the file is saved to disk
- **AND** the modified indicator is cleared
- **AND** the Shiki preview is refreshed with the latest content
- **AND** the mode remains `editor-normal`

#### Scenario: :w on unmodified file
- **WHEN** the user executes `:w` on a file with no changes
- **THEN** no write operation occurs (vim-like behavior)

### Requirement: Quit editor with :q command
The PreviewEditor SHALL transition from `editor-normal` to `global-normal` when the user executes `:q`.

#### Scenario: :q with no modifications
- **WHEN** the user executes `:q` and the file has not been modified
- **THEN** the PreviewEditor transitions to `global-normal`
- **AND** the Shiki preview is displayed

#### Scenario: :q with unsaved modifications
- **WHEN** the user executes `:q` and the file has unsaved modifications
- **THEN** the vim command line displays an error message (e.g., "No write since last change")
- **AND** the mode remains `editor-normal`

### Requirement: Save and quit with :wq command
The PreviewEditor SHALL save the file and transition to `global-normal` when the user executes `:wq`.

#### Scenario: :wq saves and exits
- **WHEN** the user executes `:wq` in `editor-normal` mode
- **THEN** the file is saved to disk
- **AND** the Shiki preview is refreshed
- **AND** the PreviewEditor transitions to `global-normal`

### Requirement: Force quit with :q! command
The PreviewEditor SHALL discard unsaved changes and transition to `global-normal` when the user executes `:q!`.

#### Scenario: :q! discards changes
- **WHEN** the user executes `:q!` in `editor-normal` mode
- **AND** the file has unsaved modifications
- **THEN** the unsaved changes are discarded
- **AND** the file content reverts to the last saved version
- **AND** the PreviewEditor transitions to `global-normal`

### Requirement: CodeMirror instance persistence
The CodeMirror editor instance SHALL persist across mode transitions between `editor-normal`, `editor-insert`, and back to `global-normal`.

#### Scenario: Undo history preserved after mode cycle
- **WHEN** the user enters `editor-normal`, makes edits, switches to `global-normal` via `:q`, and re-enters `editor-normal` via `e`
- **THEN** the undo history from the previous edit session SHALL be available

#### Scenario: Editor hidden on global-normal
- **WHEN** the PreviewEditor is in `global-normal` mode
- **THEN** the CodeMirror editor container SHALL be hidden via CSS (`display: none`)
- **AND** the Shiki preview container SHALL be visible

### Requirement: File change triggers editor rebuild
When the selected file changes, the CodeMirror editor instance SHALL be destroyed and a new one created for the new file.

#### Scenario: Select different file
- **WHEN** the user selects a different file while in `editor-normal` mode
- **THEN** the current CodeMirror instance is destroyed
- **AND** the PreviewEditor enters `global-normal` for the new file
- **AND** a new CodeMirror instance will be created when the user presses `e` again

### Requirement: Vim keybindings in CodeMirror
The CodeMirror editor SHALL use vim keybindings via the `@codemirror/vim` extension.

#### Scenario: Vim normal mode operations
- **WHEN** the user is in `editor-normal` mode
- **THEN** standard vim operations SHALL work: `dd` (delete line), `yy` (yank line), `p` (paste), `w`/`b` (word motion), `0`/`$` (line start/end), `gg`/`G` (file start/end), `/` (search)

#### Scenario: Vim visual mode
- **WHEN** the user presses `v` in `editor-normal` mode
- **THEN** vim visual mode activates and selections work as expected

### Requirement: Fullscreen editor vim mode
The FullscreenEditor SHALL also use vim keybindings and support :w/:q/:wq/:q! commands.

#### Scenario: Fullscreen :w saves
- **WHEN** the user executes `:w` in the fullscreen editor
- **THEN** the file is saved and the editor remains open

#### Scenario: Fullscreen :q closes
- **WHEN** the user executes `:q` in the fullscreen editor with no unsaved changes
- **THEN** the fullscreen editor closes

#### Scenario: Fullscreen :wq saves and closes
- **WHEN** the user executes `:wq` in the fullscreen editor
- **THEN** the file is saved and the fullscreen editor closes

#### Scenario: Fullscreen :q! force closes
- **WHEN** the user executes `:q!` in the fullscreen editor
- **THEN** unsaved changes are discarded and the fullscreen editor closes

### Requirement: Shared language detection
A shared `getLanguage()` function SHALL be used by both PreviewEditor and FullscreenEditor for CodeMirror language support detection.

#### Scenario: Language detection consistency
- **WHEN** the same file is opened in the inline editor and the fullscreen editor
- **THEN** both editors SHALL apply the same syntax highlighting language

### Requirement: Preview refresh on save
When the file is saved via `:w` or `:wq`, the Shiki preview in `global-normal` mode SHALL be refreshed with the latest content.

#### Scenario: Preview updates after :w
- **WHEN** the user executes `:w` in `editor-normal` mode
- **AND** then executes `:q` to return to `global-normal`
- **THEN** the Shiki preview displays the saved content

#### Scenario: Preview updates after :wq
- **WHEN** the user executes `:wq` in `editor-normal` mode
- **THEN** the Shiki preview in `global-normal` displays the saved content immediately
