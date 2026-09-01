## 1. Backend password support

- [x] 1.1 Add optional password parameters to archive read and extract command paths for `zip` and `7z`
- [x] 1.2 Add an in-memory archive password cache keyed by archive identity and reuse it across backend calls
- [x] 1.3 Wire `zip` archive reads/extracts to use the provided or cached password and return a distinct password error on failure
- [x] 1.4 Wire `7z` archive reads/extracts to use the provided or cached password and return a distinct password error on failure

## 2. Frontend password prompt flow

- [x] 2.1 Extend the archive password dialog to support visible text input and inline error messaging
- [x] 2.2 Show a password-needed hint for selected encrypted archives, and prompt when archive open or extraction needs credentials
- [x] 2.3 Keep the dialog open after an incorrect password and retry the archive action after a successful resubmission
- [x] 2.4 Reuse cached passwords for later archive actions in the same application session without reprompting

## 3. Archive entry point integration

- [x] 3.1 Update archive browsing entry points to pass passwords through `read_archive_directory` and `read_archive_file`
- [x] 3.2 Update extraction entry points to pass passwords through `extract_archive` and `extract_archive_files`
- [x] 3.3 Ensure the `E` mark flow seeds the password cache for later `p` extraction in the same session
- [x] 3.4 Keep `tar` and `tar.gz` flows unchanged and explicitly excluded from password prompting

## 4. Verification

- [ ] 4.1 Test encrypted `zip` archive open, preview, `e`, and `E` flows with a correct password
- [ ] 4.2 Test encrypted `7z` archive open, preview, `e`, and `E` flows with a correct password
- [ ] 4.3 Verify wrong passwords leave the dialog open and display an error instead of closing it
- [x] 4.4 Run `cargo check` and `npx svelte-check` after the implementation
