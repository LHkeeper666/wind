## ADDED Requirements

### Requirement: Backend image thumbnail generation
The system SHALL provide a Tauri command `read_image_thumbnail` that reads an image file, optionally downscales it, encodes as JPEG, and returns the result with metadata.

#### Scenario: Large image is downscaled
- **WHEN** user previews an image where the long edge exceeds 2000px
- **THEN** the system SHALL scale the image proportionally so the long edge is 2000px, encode as JPEG quality 80, and return a JSON object with `data` (base64), `width`, `height`, `original_size`, and `is_thumbnail: true`

#### Scenario: Small image passes through
- **WHEN** user previews an image where both dimensions are <= 2000px
- **THEN** the system SHALL encode the image as JPEG quality 80 without scaling, and return `is_thumbnail: false`

#### Scenario: GIF files are excluded
- **WHEN** user previews a GIF file
- **THEN** the system SHALL NOT use the thumbnail command; GIF files SHALL continue to use `read_binary_file`

#### Scenario: File does not exist
- **WHEN** the specified file path does not exist
- **THEN** the system SHALL return an error

### Requirement: Image metadata display
The ImagePreviewer SHALL display image dimension and scaling information when showing a thumbnail.

#### Scenario: Thumbnail with scaling info
- **WHEN** a thumbnail is displayed with `is_thumbnail: true`
- **THEN** the system SHALL show a status bar below the image with original dimensions (e.g., "原图 4000×3000 · 已缩放预览")

#### Scenario: Thumbnail without scaling
- **WHEN** a thumbnail is displayed with `is_thumbnail: false`
- **THEN** the system SHALL show original dimensions without scaling notice

### Requirement: View original image
The system SHALL allow users to load the full original image on demand.

#### Scenario: User requests original image
- **WHEN** user clicks "查看原图" button in the image info bar
- **THEN** the system SHALL load the original file via `read_binary_file` and display the full-resolution image
