## MODIFIED Requirements

### Requirement: Layout cycle shortcut

The system SHALL cycle column layouts when the user presses `Ctrl+W m`, in the order three-column → two-column → one-column → three-column. Two-column and one-column layouts MUST use the fixed ratios `[0,1,4]` and `[0,0,1]`. Returning to three columns MUST restore the most recently applied three-column ratios, defaulting to `[1,1,3]`. The current mode MUST be derived from the number of non-zero columns, so non-standard zero positions participate in the cycle. While `Ctrl` and `W` remain held, each distinct press of `m` SHALL advance the cycle once. Repeated `keydown` events from holding `m` MUST NOT advance it again. Releasing `Ctrl` or `W` SHALL end this continuous shortcut state. The existing sequential `Ctrl+W`, then `m` input SHALL remain supported.

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

#### Scenario: Repeated m presses while Ctrl and W remain held

- **WHEN** the user holds `Ctrl` and `W`, presses and releases `m` three times, starting from a three-column layout
- **THEN** each press advances exactly one step and the final layout restores the saved three-column ratios

#### Scenario: Holding m does not skip layouts

- **WHEN** the user holds `Ctrl` and `W` and keeps `m` pressed long enough to generate repeated keydown events
- **THEN** the layout advances exactly once until `m` is released and pressed again

#### Scenario: Releasing a modifier ends continuous cycling

- **WHEN** the user cycles once while holding `Ctrl` and `W`, then releases either `Ctrl` or `W` and presses `m` again
- **THEN** the later `m` press does not cycle the layout without a new `Ctrl+W` prefix

#### Scenario: Sequential shortcut remains available

- **WHEN** the user presses and releases `Ctrl+W`, then presses `m` within the existing prefix timeout
- **THEN** the layout advances once
