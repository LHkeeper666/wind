## ADDED Requirements

### Requirement: ffmpeg availability detection
The system SHALL detect whether ffmpeg is installed and available on the system PATH before attempting video thumbnail generation.

#### Scenario: ffmpeg is available
- **WHEN** the backend checks for ffmpeg availability
- **THEN** the system SHALL run `ffmpeg -version` and return `true` if the process exits with code 0

#### Scenario: ffmpeg is not available
- **WHEN** the backend checks for ffmpeg availability and the process fails
- **THEN** the system SHALL return `false` and video thumbnail commands SHALL return a descriptive error

### Requirement: Video thumbnail frame extraction
The system SHALL extract a single frame from a video file as a JPEG thumbnail via the `get_video_thumbnail` Tauri command.

#### Scenario: Successful thumbnail generation
- **WHEN** `get_video_thumbnail` is called with a valid video file path
- **THEN** the system SHALL spawn ffmpeg to extract a frame at the 10-second mark (or second 1 if duration < 10s), encode as JPEG, scale to max_side 400px, and return JSON with `data` (base64 JPEG), `width`, `height`, `duration_seconds`, and `file_size`

#### Scenario: ffmpeg not available
- **WHEN** `get_video_thumbnail` is called but ffmpeg is not installed
- **THEN** the system SHALL return an error with message "ffmpeg not found"

#### Scenario: Video file does not exist
- **WHEN** `get_video_thumbnail` is called with a path that does not exist
- **THEN** the system SHALL return an error with message indicating the file does not exist

#### Scenario: Short video (duration < 10 seconds)
- **WHEN** the video duration is less than 10 seconds
- **THEN** the system SHALL extract the frame at second 1 instead of second 10

### Requirement: Thumbnail execution timeout
The system SHALL enforce a 30-second timeout on ffmpeg thumbnail generation to prevent hanging.

#### Scenario: ffmpeg process exceeds timeout
- **WHEN** ffmpeg runs for more than 30 seconds during thumbnail generation
- **THEN** the system SHALL kill the ffmpeg process and return a timeout error

### Requirement: Supported video formats
The `get_video_thumbnail` command SHALL support the following container formats: mp4, mkv, avi, mov, wmv, flv, webm, m4v, mpg, mpeg.

#### Scenario: Supported format
- **WHEN** `get_video_thumbnail` is called with any of mp4, mkv, avi, mov, wmv, flv, webm, m4v, mpg, mpeg files
- **THEN** the system SHALL attempt to extract a thumbnail via ffmpeg

#### Scenario: Unsupported format
- **WHEN** `get_video_thumbnail` is called with a non-video file extension
- **THEN** the backend behavior is undefined (callers are expected to pre-filter by extension); the ffmpeg process may fail gracefully
