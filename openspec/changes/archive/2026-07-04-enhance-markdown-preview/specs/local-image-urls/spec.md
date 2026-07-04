## ADDED Requirements

### Requirement: Convert local image paths
The system SHALL convert all relative and absolute local image paths in rendered markdown to Tauri asset URLs using `convertFileSrc`.

#### Scenario: Relative path image
- **WHEN** the preview renders `![alt](./images/photo.png)` from a file at `/docs/readme.md`
- **THEN** the img src is converted to a Tauri asset URL pointing to `/docs/images/photo.png`

#### Scenario: Absolute path image
- **WHEN** the preview renders `![alt](/home/user/photo.png)`
- **THEN** the img src is converted to a Tauri asset URL

### Requirement: Skip non-local URLs
The system SHALL NOT modify image URLs that are already remote (http/https) or data URIs.

#### Scenario: Remote URL
- **WHEN** the preview renders `![alt](https://example.com/image.png)`
- **THEN** the img src remains unchanged as `https://example.com/image.png`

#### Scenario: Data URI
- **WHEN** the preview renders `![alt](data:image/png;base64,...)`
- **THEN** the img src remains unchanged

### Requirement: Asset protocol support
The system SHALL use Tauri's `convertFileSrc` API to generate asset protocol URLs for local files.

#### Scenario: Tauri asset conversion
- **WHEN** a local file path is detected in an img src
- **THEN** the path is passed through `convertFileSrc()` to produce a valid `asset://` URL
