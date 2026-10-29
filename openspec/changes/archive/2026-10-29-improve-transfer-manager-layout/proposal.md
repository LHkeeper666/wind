## Why

The Transfer Manager leaves unused space between tightly constrained file names and a fixed-width progress indicator. Its small text also stays the same size when the user changes the application's content zoom, making transfer details harder to read.

## What Changes

- Distribute transfer row space between source, destination, and progress information so wider panels reveal more useful detail and a wider progress bar.
- Adapt row layout at narrow widths while retaining the information and actions for each transfer state.
- Increase the default readability of Transfer Manager text and make its text follow the existing content zoom controls.
- Preserve the panel's outer height and the surrounding workspace layout when zoom changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `transfer-manager`: Define responsive row layout and readable transfer information at varying panel widths and zoom levels.
- `content-zoom`: Include Transfer Manager text in the shared zoom behavior without changing workspace layout dimensions.

## Impact

- Frontend: `src/lib/components/TransferManager.svelte`, and possibly the zoom value handoff in `src/lib/components/PanelLayout.svelte`.
- OpenSpec: delta specifications for `transfer-manager` and `content-zoom`.
- No backend commands, transfer scheduling, persisted data, or dependencies change.
