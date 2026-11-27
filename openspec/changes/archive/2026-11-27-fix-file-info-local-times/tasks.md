## 1. Timestamp Contract

- [x] 1.1 Check every `get_file_info` caller, then change its three time fields to optional Unix milliseconds and remove the UTC-only formatter.
- [x] 1.2 Cover timestamp conversion in Rust, including a normal timestamp, a pre-epoch timestamp, and unavailable metadata where practical.

## 2. Local Display

- [x] 2.1 Update `FileInfoPanel` types and render each available timestamp using the runtime's local timezone; omit invalid values.
- [x] 2.2 Verify all three fields against Windows file properties, including a case where local conversion changes the date.

## 3. Validation

- [x] 3.1 Run `npx svelte-check` and `cargo check` from `src-tauri/`, fixing any issues introduced by the change.
