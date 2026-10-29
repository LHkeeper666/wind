## MODIFIED Requirements

### Requirement: Zoom does not affect layout
The system SHALL apply zoom to `.panel` content areas and Transfer Manager text and its supporting row dimensions. It SHALL NOT change column widths, the Transfer Manager's outer height or resize handle behavior, or the status bar.

#### Scenario: Column widths unchanged after zoom
- **WHEN** the user changes the zoom level
- **THEN** the three-column grid layout maintains its original column width ratios

#### Scenario: Transfer Manager outer layout unchanged after zoom
- **WHEN** the user changes the zoom level while the Transfer Manager is visible
- **THEN** the panel text scales with the zoom level
- **AND** its outer height and the surrounding workspace layout remain unchanged
