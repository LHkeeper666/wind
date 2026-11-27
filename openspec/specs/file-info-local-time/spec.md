## Purpose

Define how the file information panel presents file timestamps in the user's local timezone.

## Requirements

### Requirement: File information shows local times
The file information panel SHALL display available creation, modification, and access timestamps in the running user's local timezone, preserving the instant returned by the operating system.

#### Scenario: Local time differs from UTC
- **WHEN** a user opens file information for a file whose modification instant is 2026-11-26 00:00:00 UTC while the local timezone is UTC+08:00
- **THEN** the modified time is displayed as 2026-11-26 08:00:00 in local time

#### Scenario: Local date boundary
- **WHEN** a timestamp crosses a calendar day after conversion to the local timezone
- **THEN** the panel displays the correct local date and time

#### Scenario: Each timestamp field
- **WHEN** creation, modification, and access timestamps are all available
- **THEN** each field is converted independently and shown in local time

### Requirement: Unavailable timestamps remain absent
The file information panel MUST NOT show a fabricated epoch date or invalid date for an unavailable or unrepresentable timestamp.

#### Scenario: Metadata read fails
- **WHEN** the operating system does not provide one of the timestamps
- **THEN** the corresponding time row is omitted and the other available rows remain visible

#### Scenario: Timestamp cannot be displayed
- **WHEN** a returned timestamp cannot be represented by the UI date formatter
- **THEN** the corresponding time row is omitted rather than displaying an invalid date
