## ADDED Requirements

### Requirement: Ratio syntax supports colon and dot separators

The system SHALL accept `:ratio A:B:C` and `:ratio A.B.C` as equivalent forms. The command MUST contain exactly three values, all three MUST use the same separator, each value MUST be a non-negative integer, and the sum of the three values MUST be greater than zero. Decimals, negative values, mixed separators, fewer than three values, more than three values, and all-zero values MUST be rejected.

#### Scenario: Colon syntax

- **WHEN** user types `:ratio 2:1:2`
- **THEN** column ratios are set to 2:1:2

#### Scenario: Dot syntax

- **WHEN** user types `:ratio 2.1.2`
- **THEN** column ratios are set to 2:1:2

#### Scenario: Zero hides a column

- **WHEN** user types `:ratio 0:1:4`
- **THEN** the parent column has zero width while current and preview remain visible

#### Scenario: Invalid ratio is reported

- **WHEN** user enters a ratio with one or two values, four values, a negative value, a decimal such as `1:2.2`, or all zero values
- **THEN** the layout does not change and a usage error toast is displayed

### Requirement: Preview-only ratio preset

The system SHALL provide a preview-only layout with ratios `[0,0,1]`. It MUST be reachable through `:ratio 0:0:C` where C is greater than zero, through `:ratio preview`, and through the layout cycle shortcut.

#### Scenario: Zero-only preview request

- **WHEN** user types `:ratio 0:0:1` or `:ratio 0.0.5`
- **THEN** column ratios are normalized to `[0,0,1]`

#### Scenario: Preview alias

- **WHEN** user types `:ratio preview`
- **THEN** column ratios are set to `[0,0,1]`

### Requirement: Layout cycle shortcut

The system SHALL cycle column layouts when the user presses `Ctrl+W m`, in the order three-column → two-column → one-column → three-column. Two-column and one-column layouts MUST use the fixed ratios `[0,1,4]` and `[0,0,1]`. Returning to three columns MUST restore the most recently applied three-column ratios, defaulting to `[1,1,3]`. The current mode MUST be derived from the number of non-zero columns, so non-standard zero positions participate in the cycle.

#### Scenario: Cycle from three columns

- **WHEN** the current layout has three non-zero columns and user presses `Ctrl+W m`
- **THEN** column ratios become `[0,1,4]`

#### Scenario: Cycle from two columns

- **WHEN** the current layout has two non-zero columns and user presses `Ctrl+W m`
- **THEN** column ratios become `[0,0,1]`

#### Scenario: Cycle from one column

- **WHEN** the current layout has one non-zero column and user presses `Ctrl+W m`
- **THEN** column ratios return to the most recently applied three-column ratios

#### Scenario: Custom three-column ratio is restored

- **WHEN** user applies `:ratio 2:1:2`, then cycles to two columns, one column, and back to three columns
- **THEN** column ratios are `[2,1,2]`

#### Scenario: Custom two-column ratio is not retained

- **WHEN** user applies `:ratio 0:2:1`, then cycles to one column and back to two columns
- **THEN** column ratios are `[0,1,4]`

#### Scenario: Explicit preview preset participates in the cycle

- **WHEN** user applies `:ratio 0:0:1` or `:ratio preview` and then presses `Ctrl+W m`
- **THEN** column ratios return to the most recently applied three-column ratios

#### Scenario: Non-standard zero positions

- **WHEN** user applies `:ratio 1:0:4` and then presses `Ctrl+W m`
- **THEN** the two-visible-column layout advances to `[0,0,1]`

### Requirement: Zero-width columns are not focusable

The system SHALL exclude columns whose ratio is zero from keyboard focus navigation and SHALL move focus out of a column that becomes zero-width.

#### Scenario: Skip hidden parent

- **WHEN** the parent ratio is zero, current is focused, and user presses `Ctrl+W h`
- **THEN** focus remains on the nearest visible column in that direction

#### Scenario: Active column hidden by a ratio command

- **WHEN** current is focused and user applies `:ratio 0:0:1`
- **THEN** focus moves to the preview column

#### Scenario: Restored tab has a hidden active column

- **WHEN** a tab is restored with `activeColumn` pointing at a zero-ratio column
- **THEN** focus moves to the nearest visible column

## MODIFIED Requirements

### Requirement: Default ratio preset

The system SHALL provide `:ratio` and `:ratio default` commands that restore the default three-column layout. Applying either command SHALL also update the three-column ratios used by the layout cycle.

#### Scenario: Restore default ratio

- **WHEN** user types `:ratio` or `:ratio default`
- **THEN** column ratios are set to 1:1:3

#### Scenario: Default resets the cycle snapshot

- **WHEN** the saved three-column ratio differs from 1:1:3 and user types `:ratio default`
- **THEN** cycling back to three columns restores 1:1:3
