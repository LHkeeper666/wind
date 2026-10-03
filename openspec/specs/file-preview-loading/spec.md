# File Preview Loading

Handles loading file content for the preview system, including text/binary detection and routing.

## Requirements

### Requirement: Binary file early exit in loadTextOrBinary

`loadTextOrBinary()` SHALL check `isTextFile()` before attempting to read file content. For files where `isTextFile()` returns `false`, the function SHALL directly invoke `read_binary_file()` or `read_binary_file_partial()` without first attempting a text read.

#### Scenario: Known binary extension triggers binary-only read
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is in `BINARY_EXTENSIONS` (e.g., `.exe`, `.dll`, `.mp3`)
- **THEN** the function SHALL call `read_binary_file()` or `read_binary_file_partial()` directly, without calling `read_file()` or `read_file_partial()`

#### Scenario: Unknown extension preserves existing text-first behavior
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is NOT in `BINARY_EXTENSIONS` (e.g., `.xyz`, `.dat`)
- **THEN** the function SHALL attempt text read first, then fall back to binary detection via null byte check (existing behavior)

#### Scenario: Text file extension preserves existing text read behavior
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is a known text type (e.g., `.txt`, `.js`, `.py`)
- **THEN** the function SHALL read the file as text via `read_file()` or `read_file_partial()` (existing behavior unchanged)