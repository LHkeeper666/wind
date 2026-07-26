# panel-detach Specification

## Purpose
TBD - created by archiving change add-ftp-client. Update Purpose after archive.
## Requirements
### Requirement: Left panel auto mode (default)
The system SHALL default left panel to auto mode, where the left panel path is automatically derived as the parent directory of the center panel's current path.

#### Scenario: Auto mode follows center panel
- **WHEN** center panel navigates to `C:\Users\foo\Projects`
- **THEN** left panel automatically displays `C:\Users\foo`

#### Scenario: Auto mode for drive root
- **WHEN** center panel navigates to `C:\`
- **THEN** left panel displays virtual root `\` showing all drives and FTP connections

### Requirement: Detach left panel
The system SHALL allow users to detach the left panel via explicit command or keybinding, making its path independent of the center panel.

#### Scenario: Detach via command
- **WHEN** user types `:detach`
- **THEN** left panel path freezes at its current value and enters manual mode
- **AND** left panel header shows a manual mode indicator (e.g., `[●]`)

#### Scenario: Detach via keybinding
- **WHEN** user presses `t` then `d` (tab prefix + d)
- **THEN** left panel toggles between auto and manual mode

#### Scenario: Detach from auto mode
- **WHEN** left panel is in auto mode and user triggers detach
- **THEN** left panel path is set to its current displayed path and stays fixed

#### Scenario: No automatic detach
- **WHEN** center panel navigates to a path with a different scheme (e.g., FTP vs local)
- **OR** left panel navigates independently via `Enter` or `:cd`
- **THEN** left panel mode remains unchanged. Detach MUST only occur via explicit user action (`:detach` or `t d`).

### Requirement: Left panel manual mode navigation
In manual mode, the left panel SHALL function as an independent navigation panel with its own path state.

#### Scenario: Navigate into subdirectory in manual mode
- **WHEN** left panel is in manual mode and user presses `Enter` on a directory
- **THEN** left panel navigates into that directory without affecting center panel

#### Scenario: Navigate up in manual mode
- **WHEN** left panel is in manual mode and user presses `h`
- **THEN** left panel navigates to its own parent directory

#### Scenario: Command palette cd in manual mode
- **WHEN** left panel is focused and in manual mode, and user types `:cd C:\Downloads`
- **THEN** left panel navigates to `C:\Downloads` without affecting center panel

#### Scenario: All DirectoryPanel shortcuts work in manual mode
- **WHEN** left panel is in manual mode
- **THEN** all standard DirectoryPanel keyboard shortcuts (j, k, gg, G, y, x, d, D, space, v, R, etc.) operate on the left panel's own file list

### Requirement: Attach left panel
The system SHALL allow users to re-attach the left panel, restoring auto mode where it follows the center panel's parent directory.

#### Scenario: Attach via command
- **WHEN** left panel is in manual mode and user types `:attach`
- **THEN** left panel path immediately updates to center panel's parent directory
- **AND** left panel enters auto mode
- **AND** manual mode indicator is removed

#### Scenario: Attach via keybinding
- **WHEN** left panel is in manual mode and user presses `t` then `d`
- **THEN** left panel toggles back to auto mode (same keybinding toggles both ways)

### Requirement: Manual mode visual indicator
The system SHALL display a visual indicator in the left panel header when it is in manual mode.

#### Scenario: Auto mode header
- **WHEN** left panel is in auto mode
- **THEN** left panel header displays the path name without any special marker

#### Scenario: Manual mode header
- **WHEN** left panel is in manual mode
- **THEN** left panel header displays a distinct marker (e.g., background color change or icon) so the user can distinguish it from auto mode

### Requirement: Cross-panel paste routing
The system SHALL route paste operations to the correct backend based on the source and destination path schemes.

#### Scenario: Local to local paste
- **WHEN** clipboard contains local files and the active paste target is a local directory
- **THEN** system uses existing local copy/move operations (unchanged behavior)

#### Scenario: Local to FTP paste (upload)
- **WHEN** clipboard contains local files and the active paste target is an FTP directory
- **THEN** system invokes ftp_upload for each file

#### Scenario: FTP to local paste (download)
- **WHEN** clipboard contains FTP files and the active paste target is a local directory
- **THEN** system invokes ftp_download for each file

#### Scenario: FTP to FTP paste (same server)
- **WHEN** clipboard contains FTP files from server "A" and the active paste target is another directory on server "A"
- **THEN** system uses server-side rename/move (RNFR/RNTO) for efficiency

#### Scenario: FTP to FTP paste (different servers)
- **WHEN** clipboard contains FTP files from server "A" and the active paste target is a directory on server "B"
- **THEN** system downloads from A then uploads to B (relay transfer)

### Requirement: Panel focus switching unchanged
The system SHALL retain the existing Ctrl+W h/l focus switching logic regardless of left panel mode.

#### Scenario: Switch focus from center to left
- **WHEN** center panel is focused and user presses Ctrl+W h
- **THEN** focus moves to left panel regardless of whether left is in auto or manual mode

#### Scenario: Switch focus from left to center
- **WHEN** left panel is focused and user presses Ctrl+W l
- **THEN** focus moves to center panel

### Requirement: Detach state persistence across tabs
The system SHALL persist left panel detach state (auto/manual mode, leftPath, cursorIndex, scrollOffset) when switching tabs.

#### Scenario: Save detach state on tab switch
- **WHEN** left panel is in manual mode at `ftp://srv/var` and user switches to another tab
- **THEN** the current tab saves leftMode='manual', leftPath='ftp://srv/var', leftCursorIndex, and leftScrollOffset

#### Scenario: Restore detach state on tab switch
- **WHEN** user switches back to a tab that was previously in manual mode
- **THEN** left panel restores in manual mode at the saved path with the saved cursor position and scroll offset

#### Scenario: Restore auto mode tab
- **WHEN** user switches back to a tab that was in auto mode
- **THEN** left panel starts in auto mode, deriving its path from the restored center panel path

