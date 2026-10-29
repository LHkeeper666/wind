## Context

`TransferManager.svelte` renders a full-width bottom panel outside the three `.panel` elements. Its row uses Flexbox with source/destination capped at 300px, a 120px progress bar, and `margin-left: auto` on metadata. This leaves unused space at wide widths. The panel uses fixed 10–12px text. `PanelLayout.svelte` stores content zoom in `--zoom-level` but applies CSS `zoom` to `.panel` only. Existing transfer specifications require single-line rows, including at narrow widths.

## Goals / Non-Goals

**Goals:**
- Give file names and progress the available horizontal space, with readable behavior at narrow widths.
- Use the existing content zoom value for Transfer Manager text and the dimensions needed to fit that text.
- Keep panel height, workspace column widths, resize handle, and status bar unaffected by transfer text scaling.

**Non-Goals:**
- Change transfer scheduling, progress calculations, history, or keyboard controls.
- Introduce a separate Transfer Manager zoom preference.

## Decisions

1. Replace the row's fixed-width composition with flexible source/destination regions and a progress region that can grow. Keep status-specific content in the progress region and reserve a small action region. At narrow widths, let transfer details form a second line. This uses horizontal space without requiring horizontal scrolling. Alternative: enlarge the fixed width limits; that still creates gaps at wider window sizes.
2. Use `--zoom-level` for Transfer Manager typography and supporting row dimensions, with a larger readable base size. Do not apply CSS `zoom` to the whole bottom panel, since that could change its measured height and drag behavior. Alternative: pass `zoomLevel` as a component prop; the shared CSS variable already exists and avoids duplicate state.
3. Preserve the current short file name display unless a path is needed to distinguish items; retain full source/destination in tooltips. Layout work should not change transfer identity or data formatting. Alternative: always show full paths; long common prefixes would consume the space intended for names and progress.
4. Keep the title bar and single-line rows close to the current directory panel's vertical density. Scale their compact padding with content zoom, while reserving extra lines only when the available width requires reflow.
5. Give progress percentage, bar, transferred size, speed, and time fixed semantic columns in each row. Queue and completed sizes occupy the same size column as running transfers, so values line up vertically across statuses.

## Risks / Trade-offs

- [Very long names or FTP locations] → Allow truncation within flexible columns and show full values on hover.
- [Large zoom at narrow width] → Switch to a two-line row and keep cancel/dismiss actions reachable.
- [Mixed transfer states have different metadata] → Check running, queued, completed, failed, cancelled, and indeterminate extraction states separately.

## Migration Plan

Frontend-only change; no persisted data migration. Reverting the component layout and zoom styles restores the previous presentation.

## Open Questions

None required before implementation.
