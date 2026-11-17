## 1. Local conflict decisions

- [x] 1.1 Add a local-only Keep Copy action to top-level and streamed conflict dialogs without changing FTP or archive choices.
- [x] 1.2 Implement first-available ` (n)` naming for ordinary, extensionless, and leading-dot files, reserving names within one paste batch.

## 2. Transfer destinations

- [x] 2.1 Carry explicit destinations for top-level local file transfers through the paste task builder, including copy and cut.
- [x] 2.2 Carry per-file renamed destinations from local directory conflict scan into local directory transfer execution while preserving skip and merge behavior.
- [x] 2.3 Protect Keep Copy destinations against late collisions and ensure cancellation cleanup and cut source removal use the actual transferred paths.
- [x] 2.4 Ensure top-level transfer feedback uses the chosen destination.

## 3. Verification

- [x] 3.1 Add focused tests for suffix selection, repeated paste, batched names, top-level and nested task routing, and cut behavior; verify late collision protection through exclusive destination creation.
- [x] 3.2 Run `npx svelte-check` and `cargo check` plus relevant focused tests; verify existing Overwrite, Skip, Abort, `P`, FTP, and archive behavior remains intact.

## 4. Duplicate all conflicts

- [x] 4.1 Add Duplicate All to local conflict dialogs and carry its policy across directory scanning and top-level files for one paste.
- [x] 4.2 Verify repeated conflicts use unique destinations and that FTP, archive, `P`, and a later paste keep their existing behavior.
