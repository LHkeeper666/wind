# Markdown Local SVG Images

## Purpose

Render local SVG images referenced from Markdown previews.

## Requirements

### Requirement: Markdown renders local SVG images
The system SHALL render local SVG files referenced by Markdown image syntax using the SVG image MIME type during initial rendering and incremental updates.

#### Scenario: Relative Markdown image
- **WHEN** Markdown contains `![diagram](./diagram.svg)` and that file is valid
- **THEN** the preview displays the diagram

#### Scenario: Obsidian image reference
- **WHEN** Markdown contains `![[diagram.svg]]` and that file is valid
- **THEN** the preview displays the diagram

#### Scenario: Incremental update
- **WHEN** Markdown is updated while retaining or adding a local SVG image reference
- **THEN** the rendered image remains visible or is loaded without a broken cached URL

### Requirement: Local SVG image failures are contained
The system SHALL expose a failed local SVG image as an image-level failure and SHALL keep the rest of the Markdown preview rendered.

#### Scenario: Missing or malformed image
- **WHEN** a referenced SVG file is missing or cannot be decoded
- **THEN** that image shows a failure indication
- **AND** the remaining Markdown stays rendered
