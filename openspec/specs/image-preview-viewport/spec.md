# Image Preview Viewport

## Purpose

Provide responsive image viewing in preview slots while preserving fast thumbnail-first selection.

## Requirements

### Requirement: Image opens fitted in preview viewport
The system SHALL display an image opened from the current panel in the preview slot with its aspect ratio preserved, fully visible and centered within the available image viewport.

#### Scenario: Open image from current panel
- **WHEN** the user selects an image in the current panel and presses `l`
- **THEN** the image opens in the preview slot and fits entirely within its viewport without cropping or distortion

#### Scenario: Resize preview before manual zoom
- **WHEN** the preview viewport changes size before the user manually zooms
- **THEN** the image remains fully visible, centered and proportionally scaled

### Requirement: Focused image preview supports directional movement
The system SHALL use `j/k/h/l` to move the image down, up, left and right respectively while its preview slot has focus, and SHALL constrain movement to the image bounds.

#### Scenario: Pan a zoomed image
- **WHEN** a zoomed image extends beyond the focused preview viewport and the user presses `j/k/h/l`
- **THEN** the image moves in the corresponding direction without moving outside its available bounds

#### Scenario: Image fits entirely
- **WHEN** the image fits entirely in the focused viewport and the user presses a movement key
- **THEN** the image remains centered and no other preview or panel is scrolled

### Requirement: Focused image preview supports smooth zoom
The system SHALL zoom only the focused preview image with `Ctrl+=`, `Ctrl+-` and `Ctrl+wheel`, SHALL preserve the zoom anchor, and SHALL keep zoom within defined bounds.

#### Scenario: Keyboard zoom
- **WHEN** the image preview slot has focus and the user presses `Ctrl+=` or `Ctrl+-`
- **THEN** only that image changes scale smoothly around the viewport center

#### Scenario: Pointer zoom
- **WHEN** the image preview slot has focus and the user uses `Ctrl+wheel` over the image viewport
- **THEN** only that image changes scale smoothly while the image point under the pointer remains stable

#### Scenario: Other preview has focus
- **WHEN** the image preview slot does not have focus
- **THEN** its movement and zoom shortcuts do not alter that image

### Requirement: Original image loads only after intentional enlargement
The system SHALL preserve the existing thumbnail-first selection path and SHALL request the original image in the background only after the user intentionally zooms beyond the loaded thumbnail's effective pixel resolution, or explicitly chooses the existing original-image action.

#### Scenario: Initial selection of large image
- **WHEN** a large image is selected and shown at fit scale
- **THEN** the existing thumbnail preview appears without waiting for an original-image request

#### Scenario: Zoom exceeds effective resolution
- **WHEN** the user enlarges a thumbnail so that its displayed CSS dimensions multiplied by the device pixel ratio exceed its actual pixel dimensions on either axis
- **THEN** the system starts at most one original-image request for that preview while the thumbnail remains interactive

#### Scenario: Original image replaces thumbnail
- **WHEN** the requested original image has decoded successfully and the same file remains in the same preview slot
- **THEN** the system replaces the thumbnail without a blank frame or a change in zoom, pan position or focus

#### Scenario: Image changes during request
- **WHEN** the user switches images or closes the preview before the original request completes
- **THEN** the stale result does not replace the current preview and its temporary resources are released

#### Scenario: Original image fails to load
- **WHEN** the original-image request or decode fails
- **THEN** the thumbnail remains visible and interactive, and the user can retry loading the original

### Requirement: Image view survives preview slot switching
The system SHALL retain each preview slot's image zoom and position when switching between slots and SHALL display an already rendered image without an intermediate blank frame or unnecessary reload.

#### Scenario: Return to an unchanged image slot
- **WHEN** the user switches away from and back to a slot containing an unchanged image
- **THEN** the image appears immediately at its previous zoom and position

#### Scenario: Image node must be rebuilt
- **WHEN** an image preview is rebuilt for the same file
- **THEN** the previous image remains visible until its replacement is ready and the zoom and position are restored

### Requirement: Image preview does not represent an edited draft
The system SHALL NOT mark an unedited image preview slot as modified when restoring it from cache.

#### Scenario: Restore cached image
- **WHEN** the user returns to a cached image preview slot
- **THEN** the slot has no modified indicator solely because its binary placeholder differs from saved text content
