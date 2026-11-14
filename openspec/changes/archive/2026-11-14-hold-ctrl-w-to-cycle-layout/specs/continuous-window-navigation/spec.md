## ADDED Requirements

### Requirement: Continuous Ctrl+W panel navigation

The system SHALL allow the user to hold `Ctrl` and `W` and press `h`, `j`, `k`, or `l` repeatedly to invoke the existing panel navigation for each distinct key press. The prefix SHALL remain active while both keys remain held, including after an `m` layout cycle, so these commands MAY be mixed. Repeated `keydown` events caused by holding a navigation key MUST NOT invoke navigation again. Releasing `Ctrl` or `W` SHALL end continuous navigation. Sequential `Ctrl+W`, then one navigation key within the existing prefix timeout SHALL remain supported.

#### Scenario: Navigate horizontally with repeated presses

- **WHEN** the user holds `Ctrl+W` and separately presses `l` twice from the parent panel in a three-column layout
- **THEN** focus advances to current and then preview using the existing focus routing

#### Scenario: Preserve existing edge and hidden-column behavior

- **WHEN** the user holds `Ctrl+W` and presses `h` or `l` where no further visible panel exists
- **THEN** focus follows the existing edge and hidden-column behavior

#### Scenario: Navigate between workspace and terminal

- **WHEN** the terminal is visible and the user holds `Ctrl+W`, presses `j`, releases `j`, then presses `k`
- **THEN** focus moves to the terminal and then returns according to the existing pre-terminal panel rule

#### Scenario: Preserve outline focus routing

- **WHEN** a visible Markdown or PDF outline can receive focus and the user repeatedly uses `Ctrl+W l` or `Ctrl+W h` while holding the prefix
- **THEN** each distinct press uses the existing preview-to-outline and outline-to-content routing

#### Scenario: Mix navigation with layout cycling

- **WHEN** the user holds `Ctrl+W`, presses and releases `m`, then presses a navigation key
- **THEN** the layout cycles once and the navigation key acts on the resulting visible panels

#### Scenario: Ignore navigation key repeat

- **WHEN** the user holds `Ctrl+W` and holds one of `h/j/k/l` long enough to generate repeated keydown events
- **THEN** the navigation action executes once until that navigation key is released and pressed again

#### Scenario: Release ends continuous navigation

- **WHEN** the user navigates while holding `Ctrl+W`, releases `Ctrl` or `W`, and then presses a navigation key
- **THEN** the later key does not invoke a `Ctrl+W` navigation action without a new prefix

#### Scenario: Sequential navigation remains available

- **WHEN** the user presses and releases `Ctrl+W`, then presses one of `h/j/k/l` within the existing prefix timeout
- **THEN** the existing navigation action executes once
