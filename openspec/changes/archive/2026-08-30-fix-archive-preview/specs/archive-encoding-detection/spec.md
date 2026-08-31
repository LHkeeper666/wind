## ADDED Requirements

### Requirement: ZIP entry name encoding auto-detection
The system SHALL automatically detect the character encoding of ZIP archive entry names and decode them correctly, without relying solely on the ZIP format's language encoding flag (bit 11).

#### Scenario: UTF-8 flag set in ZIP
- **WHEN** a ZIP entry has bit 11 of the general purpose bit flag set to 1
- **THEN** the system SHALL decode the entry name as UTF-8

#### Scenario: UTF-8 flag not set but name is valid UTF-8
- **WHEN** a ZIP entry has bit 11 not set, but the raw name bytes are valid UTF-8
- **THEN** the system SHALL decode the entry name as UTF-8

#### Scenario: Non-UTF-8 name detected (GBK)
- **WHEN** a ZIP entry has bit 11 not set and the raw name bytes are not valid UTF-8
- **THEN** the system SHALL use chardetng to detect the encoding from the raw bytes
- **AND** SHALL decode the entry name using the detected encoding (e.g., GBK)

#### Scenario: Non-UTF-8 name detected (Shift-JIS)
- **WHEN** a ZIP entry has bit 11 not set and raw bytes are valid Shift-JIS but not UTF-8
- **THEN** the system SHALL detect Shift-JIS encoding via chardetng
- **AND** SHALL decode the entry name correctly

#### Scenario: Encoding detection fallback for short names
- **WHEN** chardetng cannot reliably detect encoding for a short entry name (fewer than 4 bytes)
- **THEN** the system SHALL fall back to CP437 (IBM Code Page 437) as the ZIP specification default

#### Scenario: Raw bytes extraction failure
- **WHEN** the system cannot parse the ZIP Central Directory to extract raw filename bytes
- **THEN** the system SHALL fall back to the zip crate's `entry.name()` result
- **AND** the archive listing SHALL still succeed

### Requirement: TAR entry name encoding auto-detection
The system SHALL automatically detect the character encoding of TAR archive entry names and decode them correctly.

#### Scenario: TAR entry name is valid UTF-8
- **WHEN** a TAR entry's raw name bytes (from `path_bytes()`) are valid UTF-8
- **THEN** the system SHALL decode the entry name as UTF-8

#### Scenario: TAR entry name is non-UTF-8
- **WHEN** a TAR entry's raw name bytes are not valid UTF-8
- **THEN** the system SHALL use chardetng to detect the encoding
- **AND** SHALL decode the entry name using the detected encoding

#### Scenario: TAR raw bytes unavailable
- **WHEN** `entry.path_bytes()` returns an error or is unavailable
- **THEN** the system SHALL fall back to `entry.path()?.to_string_lossy()`
- **AND** the archive listing SHALL still succeed

### Requirement: Encoding detection cache
The system SHALL cache the detected encoding per archive file to avoid redundant detection for every entry.

#### Scenario: Cache hit for same archive
- **WHEN** the system has already detected the encoding for a ZIP/TAR archive
- **THEN** subsequent entry name decoding for the same archive SHALL use the cached encoding
- **AND** SHALL NOT re-run chardetng detection

#### Scenario: Cache cleared on archive close
- **WHEN** the archive listing operation completes
- **THEN** the encoding cache for that archive MAY be retained for the session lifetime
- **AND** SHALL NOT persist across application restarts

#### Scenario: Different archives have independent caches
- **WHEN** two different archive files are opened
- **THEN** each archive SHALL have its own independently detected and cached encoding