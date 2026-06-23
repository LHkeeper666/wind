## ADDED Requirements

### Requirement: Video file format matching
The VideoPreviewer SHALL match video files by extension, supporting mp4, mkv, avi, mov, wmv, flv, webm, m4v, mpg, mpeg.

#### Scenario: Video file selected
- **WHEN** a file with any of the supported video extensions is selected in the directory panel
- **THEN** the PreviewRouter SHALL route the file to the VideoPreviewer

#### Scenario: Non-video file selected
- **WHEN** a file with a non-video extension is selected
- **THEN** the VideoPreviewer SHALL return false from `match()` and the file SHALL be routed to the next previewer

#### Scenario: GIF file is NOT video
- **WHEN** a .gif file is selected
- **THEN** the VideoPreviewer SHALL NOT match it; ImagePreviewer SHALL handle it

### Requirement: Thumbnail display in preview panel
The VideoPreviewer SHALL display the video thumbnail image in the preview panel when thumbnail data is available.

#### Scenario: Thumbnail available
- **WHEN** `get_video_thumbnail` returns valid JPEG data
- **THEN** the VideoPreviewer SHALL create a blob URL from the base64 data and render it via an `<img>` element with `max-width: 100%` and `object-fit: contain`

#### Scenario: Thumbnail marks a playable video
- **WHEN** a video thumbnail is displayed
- **THEN** a play button overlay or indicator SHALL be shown to distinguish it from a static image preview

### Requirement: Video metadata info bar
The VideoPreviewer SHALL display a metadata info bar below the thumbnail showing duration, resolution, and file size.

#### Scenario: Metadata available
- **WHEN** thumbnail data includes `width`, `height`, `duration_seconds`, and `file_size`
- **THEN** the info bar SHALL display formatted duration (MM:SS), resolution (W×H), and human-readable file size (e.g., "850.0 MB")

#### Scenario: Metadata partially available
- **WHEN** some metadata fields are missing or zero
- **THEN** the info bar SHALL display only the available fields without error

### Requirement: ffmpeg unavailable fallback
The VideoPreviewer SHALL show a descriptive message when ffmpeg is not installed.

#### Scenario: ffmpeg not installed
- **WHEN** `get_video_thumbnail` returns an error indicating ffmpeg is not found
- **THEN** the preview panel SHALL display a message: "ffmpeg 未安装，无法预览视频。运行 `winget install ffmpeg` 安装。" along with the file name and size

#### Scenario: Timeout or other error
- **WHEN** `get_video_thumbnail` returns a non-ffmpeg error (timeout, corrupt file)
- **THEN** the preview panel SHALL display the error message with the file name

### Requirement: Action hints in preview
The VideoPreviewer SHALL display keyboard action hints in the info bar.

#### Scenario: 200MB or smaller video
- **WHEN** video file size is <= 200MB and no larger than 1GB
- **THEN** the info bar SHALL show "E:全屏播放"

#### Scenario: Video between 200MB and 1GB
- **WHEN** video file size is > 200MB and <= 1GB
- **THEN** the info bar SHALL show "E:全屏播放 (文件较大)" with a warning indicator

#### Scenario: Video over 1GB
- **WHEN** video file size is > 1GB
- **THEN** the info bar SHALL show "文件过大，请用系统播放器打开"

### Requirement: Preview cleanup on dispose
The VideoPreviewer SHALL clean up blob URLs and DOM content when `dispose()` is called.

#### Scenario: Preview changes to another file
- **WHEN** the user navigates to a different file and VideoPreviewer.dispose() is called
- **THEN** all blob URLs created by the VideoPreviewer SHALL be revoked via `URL.revokeObjectURL()`
