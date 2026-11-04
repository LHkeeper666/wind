## Context

The current panel opens `SelectionPreviewOverlay` on selection. `.torrent` is absent from `BINARY_EXTENSIONS`, so `loadTextOrBinary` reads and decodes the whole file as text, detects NUL, then reads it again as binary. `TextPreviewer` renders up to 64 KiB of binary data as 4,096 hex rows in one synchronous `innerHTML` update. The overlay uses the same router and renderer as ordinary preview slots.

## Goals / Non-Goals

**Goals:**
- Route `.torrent` straight to the binary read path in every preview context.
- Bound synchronous hex rendering and live DOM rows while preserving hex offsets, ASCII column, scroll navigation, and truncation notice.
- Preserve latest-selection wins behavior and correct total file size in the overlay.

**Non-Goals:**
- Decode torrent metadata or add a torrent-specific viewer.
- Change text preview behavior or the general 1 MiB file-loading cap.

## Decisions

1. Add `torrent` to the authoritative `BINARY_EXTENSIONS` set. The existing `isTextFile` early exit and `TextPreviewer` binary check will then agree. Alternative: add an overlay-only exception; that would leave ordinary slots and editor routing inconsistent.
2. Virtualize hex rows in `TextPreviewer` with a fixed row height, scroll spacer, and a small overscanned window. Generate row text only for visible offsets and update the window on scroll. Preserve the existing 64 KiB display cap. Alternative: render rows in chunks; it reduces first-frame blocking but eventually retains thousands of nodes.
3. Pass `originalFileSize` from `loadTextOrBinary` through the overlay/router to the hex renderer so the notice reports disk size even when preview input is partial. Keep generation checks around async selection loading and detach scroll listeners on disposal. Alternative: infer size from the received buffer; that gives an incorrect total for partial reads.

## Risks / Trade-offs

- Fixed-height assumptions can cause scroll drift with font or zoom changes → Measure or recompute row height when the view changes; keep row layout non-wrapping.
- Reused `TextPreviewer` instances may retain an old scroll listener → Remove listeners before rerender and on dispose; test rapid selection changes.
- Virtual rows change DOM structure used by navigation → Retain `data-line` on rendered rows and validate scroll/offset behavior.

## Migration Plan

No data migration. Ship as a frontend change; revert the change if preview regressions occur.

## Open Questions

None required for implementation.
