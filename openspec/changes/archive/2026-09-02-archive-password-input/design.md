## Context

The current archive flow already supports browsing and extracting `zip`, `tar`, `tar.gz`, and `7z` archives, but it assumes archives are readable without credentials. Encrypted `zip` and `7z` archives currently fail without a password entry path, and the frontend has no retry loop for wrong credentials.

The change spans the Rust archive backend, the Tauri command surface, and the Svelte archive UI. The existing `InputDialog` can provide the base interaction pattern, but it does not yet model an archive-specific password retry flow.

## Goals / Non-Goals

**Goals:**
- Support password entry for encrypted `zip` and `7z` archives.
- Show a preview-panel hint when an encrypted archive is merely selected.
- Prompt when the user explicitly enters an encrypted archive or starts extraction.
- Keep the password prompt open after a failed attempt and show a clear error.
- Cache successful passwords for the lifetime of the app session.

**Non-Goals:**
- Add password support for `tar` or `tar.gz`.
- Redesign archive browsing or extraction UX beyond the password flow.
- Persist passwords across app restarts.

## Decisions

- Use optional password parameters on archive read/extract commands instead of adding separate password-only commands.
  - Rationale: the current command surface already groups archive operations by action; optional passwords keep the API small and preserve existing call sites for unencrypted archives.
  - Alternatives considered: separate preflight/auth commands or a full archive session object. Both add more plumbing than this change needs.

- Keep the password cache in memory for the backend process lifetime.
  - Rationale: the app session boundary matches the user request, avoids disk persistence, and lets every archive command reuse the same cache.
  - Alternatives considered: frontend-only caching or persistent storage. Frontend-only caching is fragile across entry points, and persistent storage is a security regression.

- Surface archive credential failures with a recognizable backend error code or message.
  - Rationale: the UI needs to distinguish "need password" from generic archive corruption so it can keep the dialog open and retry cleanly.
  - Alternatives considered: infer from generic error text. That is brittle and would leak backend-specific wording into the UI.

- Use a visible text input for the password prompt by default.
  - Rationale: the requirement explicitly asks for visible entry, and the archive flow is closer to a quick credential retry than a secrets vault.
  - Alternatives considered: a masked password field with reveal toggle. That would be more conservative, but it does not match the requested default behavior.

- Retry archive reads after a password is entered, then reuse the cached password for later actions on the same archive.
  - Rationale: entering the password once should unblock both browsing and extraction during the session.
  - Alternatives considered: prompting separately for browse and extract. That would be repetitive and violate the caching goal.

## Risks / Trade-offs

- [ZipCrypto can accept the wrong password in some cases] -> Treat later decryption/read failures as password failure and allow retry instead of assuming the first success is valid.
- [AES-encrypted ZIP support may depend on crate features] -> Verify with real encrypted archives and enable the needed `zip` feature only if required.
- [Archive path caching can go stale if the file changes] -> Key the cache by archive path plus file fingerprint, or invalidate on size/mtime change.
- [Holding passwords in process memory increases exposure] -> Keep the cache in RAM only, never persist it, and avoid logging password values.
- [7z solid archives can make repeated retries slower] -> Keep the existing archive access pattern and only add the password branch, not a broader archive-session refactor.
