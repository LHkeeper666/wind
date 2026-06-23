## ADDED Requirements

### Requirement: Open fullscreen player via E key
The system SHALL open the FullscreenVideoPlayer when the user presses `E` with a video file selected in the preview panel.

#### Scenario: Video file selected, E pressed
- **WHEN** the preview panel has a video file loaded and the user presses `E`
- **THEN** the FullscreenVideoPlayer SHALL open as a fullscreen overlay showing the video

#### Scenario: Non-video file selected, E pressed
- **WHEN** the preview panel has a non-video file loaded and the user presses `E`
- **THEN** the existing fullscreen handler (editor or image viewer) SHALL be invoked instead

### Requirement: Video loading via Blob URL
The FullscreenVideoPlayer SHALL load the video file into memory, create a Blob URL, and render it in a `<video>` element.

#### Scenario: Video file under 200MB
- **WHEN** a video file of size <= 200MB is opened
- **THEN** the system SHALL call `read_binary_file`, create a Blob with inferred MIME type from extension, create a blob URL, and set it as the `<video>` element's `src`, then begin autoplay

#### Scenario: Video between 200MB and 1GB
- **WHEN** a video file of size > 200MB and <= 1GB is opened
- **THEN** the system SHALL display a confirmation prompt warning about potential slowness before loading

#### Scenario: Video over 1GB
- **WHEN** a video file of size > 1GB is opened
- **THEN** the system SHALL refuse to load and display a message suggesting the user open the file with the system media player

### Requirement: Playback keyboard controls
The FullscreenVideoPlayer SHALL respond to the following keyboard shortcuts during playback.

#### Scenario: Space to play/pause
- **WHEN** the user presses Space
- **THEN** the video SHALL toggle between playing and paused

#### Scenario: h/l to seek
- **WHEN** the user presses `h` or `ArrowLeft`
- **THEN** the video SHALL seek backward 5 seconds (minimum 0)
- **WHEN** the user presses `l` or `ArrowRight`
- **THEN** the video SHALL seek forward 5 seconds (maximum duration)

#### Scenario: j/k to adjust volume
- **WHEN** the user presses `j` or `ArrowDown`
- **THEN** the video volume SHALL decrease by 10% (minimum 0)
- **WHEN** the user presses `k` or `ArrowUp`
- **THEN** the video volume SHALL increase by 10% (maximum 100%)

#### Scenario: 0/$ to jump to start/end
- **WHEN** the user presses `0` or `Home`
- **THEN** the video SHALL seek to the beginning (0:00)
- **WHEN** the user presses `$` or `End`
- **THEN** the video SHALL seek to the end (duration)

### Requirement: Fullscreen toggle
The FullscreenVideoPlayer SHALL support toggling native browser fullscreen mode.

#### Scenario: f key toggles browser fullscreen
- **WHEN** the user presses `f`
- **THEN** the browser SHALL enter fullscreen mode if not already in fullscreen, or exit fullscreen if already in fullscreen

### Requirement: Close behavior
The FullscreenVideoPlayer SHALL close on `Esc` and restore the previous focus state.

#### Scenario: Esc closes player
- **WHEN** the user presses `Esc`
- **THEN** the fullscreen overlay SHALL be removed, the video SHALL be paused, the blob URL SHALL be revoked, and focus SHALL return to the preview panel

### Requirement: Video overlay layout
The FullscreenVideoPlayer SHALL render a fullscreen overlay with a header bar, centered video area, and control hints bar.

#### Scenario: Player renders
- **WHEN** the FullscreenVideoPlayer is opened
- **THEN** the overlay SHALL be fixed-position, 100% viewport, with z-index 1000, semi-transparent background, containing a header bar (file name, resolution, time progress, mode indicator), a centered `<video>` element (max 90vw × 80vh), and a bottom control hints bar showing keyboard shortcuts

### Requirement: Blob URL cleanup
The FullscreenVideoPlayer SHALL revoke Blob URLs when closed to prevent memory leaks.

#### Scenario: Player closes
- **WHEN** the FullscreenVideoPlayer is closed (via Esc or any exit path)
- **THEN** all blob URLs created during playback SHALL be revoked via `URL.revokeObjectURL()`

#### Scenario: Video load error
- **WHEN** the `<video>` element fires an error event
- **THEN** an error message SHALL be displayed and the blob URL SHALL be revoked
