# File Type Detection

Provides unified file type detection utilities for the Wind frontend.

## Requirements

### Requirement: Unified file type detection module
The system SHALL provide a single `file-types.ts` module that exports all file type detection functions.

#### Scenario: Import file type detectors from utils
- **WHEN** a component needs to check file types
- **THEN** it imports from `$lib/utils/file-types`

### Requirement: Consistent binary extension list
The system SHALL maintain a single authoritative `BINARY_EXTENSIONS` set used by both `isTextFile()` and TextPreviewer.

#### Scenario: Binary extension list is unified
- **WHEN** checking if a file is binary
- **THEN** both `isTextFile()` and TextPreviewer use the same `BINARY_EXTENSIONS` set from `file-types.ts`

### Requirement: Complete archive format support
`isArchiveFile()` SHALL detect `.zip`, `.tar`, `.tar.gz`, `.tgz`, and `.7z` extensions.

#### Scenario: Archive detection is consistent
- **WHEN** a file has extension `.tar.gz` or `.7z`
- **THEN** `isArchiveFile()` returns `true` in all components

### Requirement: TypeScript migration for project-tree-focus
`project-tree-focus.js` SHALL be migrated to `project-tree-focus.ts` with full type annotations.

#### Scenario: Type-safe project tree functions
- **WHEN** using `projectTreePathKey`, `isProjectTreePathWithin`, or `getCollapseSelectionTarget`
- **THEN** TypeScript provides parameter and return type checking