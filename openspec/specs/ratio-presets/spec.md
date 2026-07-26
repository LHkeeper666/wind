# ratio-presets Specification

## Purpose
TBD - created by archiving change add-ftp-client. Update Purpose after archive.
## Requirements
### Requirement: Dual panel ratio preset
The system SHALL provide a `:ratio dual` command that sets all three columns to equal width.

#### Scenario: Switch to dual mode
- **WHEN** user types `:ratio dual`
- **THEN** column ratios are set to 1:1:1 (parent, current, and preview columns equal width)

### Requirement: Default ratio preset
The system SHALL provide a `:ratio` command (no arguments) that restores the default column layout.

#### Scenario: Restore default ratio
- **WHEN** user types `:ratio` with no arguments
- **THEN** column ratios are set to 1:1:3 (default layout)

### Requirement: Existing custom ratio preserved
The system SHALL preserve the existing `:ratio X:Y:Z` syntax for arbitrary ratios.

#### Scenario: Custom ratio still works
- **WHEN** user types `:ratio 2:1:2`
- **THEN** column ratios are set to 2:1:2

