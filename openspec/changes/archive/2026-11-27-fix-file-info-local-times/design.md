## Context

`get_file_info` reads three `SystemTime` values, converts them to Unix seconds, and formats those seconds with `chrono_like`, which assumes UTC. `FileInfoPanel` displays the returned strings unchanged. Other frontend surfaces already format Unix timestamps with JavaScript `Date` in the user's local timezone.

## Goals / Non-Goals

**Goals:**
- Show creation, modification, and access times in the local timezone of the running app.
- Preserve the underlying instant across timezone and date boundaries.
- Keep unavailable metadata absent instead of inventing a time.

**Non-Goals:**
- Changing stored file metadata, directory sorting, FTP metadata, or other date displays.
- Adding a timezone preference or a new dependency.

## Decisions

- Change the three `get_file_info` fields from formatted strings to optional Unix timestamps in milliseconds. Milliseconds are directly accepted by JavaScript `Date` and can represent pre-1970 instants if the OS returns them. The frontend TypeScript interface changes in the same implementation step.
- Format timestamps in `FileInfoPanel` using `Date` and the runtime's local timezone. Follow the existing UI locale convention, and avoid appending a misleading fixed UTC offset. This also accounts for daylight saving rules where applicable.
- Return `null` when an individual metadata call fails. Do not use `unwrap_or_default`, which can turn an invalid or pre-epoch time into 1970-01-01.
- Remove the bespoke `chrono_like` and `is_leap` helpers after their only use is replaced. Formatting in Rust with `chrono::Local` was considered, but would couple presentation to the backend process timezone and return strings that cannot be reformatted by the UI.

## Risks / Trade-offs

- [Internal command response shape changes] → Update the Rust response and frontend type atomically; search all `get_file_info` consumers before implementation.
- [A timestamp is unsupported or outside JavaScript `Date` range] → Treat it as unavailable rather than showing `Invalid Date`.
- [Access time can differ from expectations because Windows may defer updates] → Preserve the OS value; only its timezone presentation changes.

## Migration Plan

No persisted data migration is needed. Ship backend and frontend changes together. Reverting both restores the prior command contract.

## Open Questions

None.
