## 1. FTP Rust Backend

- [x] 1.1 Add suppaftp dependency to Cargo.toml (async-native-tls feature for FTPS via schannel)
- [x] 1.2 Create `src-tauri/src/ftp.rs` module with `FtpSession` struct (wraps suppaftp FtpStream + metadata)
- [x] 1.3 Implement `FtpManager` with methods: connect, disconnect, get, list_connections
- [x] 1.4 Implement connection persistence: save/load `ftp-connections.json` in app data dir
- [x] 1.5 Implement auto-reconnect logic when accessing a stored but disconnected session
- [x] 1.6 Register `FtpManager` in `AppState` (wrapped in `Mutex`)

## 2. FTP Tauri Commands

- [x] 2.1 Implement `ftp_connect` command (name, host, port, user, pass)
- [x] 2.2 Implement `ftp_disconnect` command (name)
- [x] 2.3 Implement `ftp_read_directory` — MLSD parsing to `Vec<FileEntry>`, LIST fallback
- [x] 2.4 Implement `ftp_download` async command with progress events (emit to frontend)
- [x] 2.5 Implement `ftp_upload` async command with progress events
- [x] 2.6 Implement `ftp_delete` command (file and recursive directory)
- [x] 2.7 Implement `ftp_rename` command (remote file/directory rename)
- [x] 2.8 Implement `list_ftp_connections` command (returns stored connection metadata without passwords)

## 3. Path Scheme & Virtual Root

- [x] 3.1 Add path scheme detection: `ftp://` prefix vs local path parsing utility
- [x] 3.2 Modify `read_directory` to route to FTP vs local based on path prefix
- [x] 3.3 Modify `list_drives` / add `list_virtual_root` to include FTP connections in `\` root
- [x] 3.4 Mark FTP connection entries in root with `[FTP]` prefix in name

## 4. FTP Frontend Commands

- [x] 4.1 Add `:ftp connect <name> <host> [--port] [--user] [--pass]` command to PanelLayout command palette
- [x] 4.2 Add `:ftp disconnect <name>` command
- [x] 4.3 Add `:ftp list` command (shows toast with connection summary)
- [x] 4.4 Add `:cd ftp://name/` path resolution in resolvePath / handleCommandKeydown

## 5. Panel Detach Mode

- [x] 5.1 Add `leftMode: 'auto' | 'manual'` and `leftPath: string` fields to `LayoutState`
- [x] 5.2 Add `setLeftPath`, `detach`, `attach`, `toggleDetach` methods to layout store
- [x] 5.3 Derive left panel path: when auto → parentPath, when manual → leftPath
- [x] 5.4 Add `:detach` and `:attach` commands to command palette
- [x] 5.5 Register `t d` keybinding (tab prefix + d) in PanelLayout handleTabCommand for toggle detach
- [x] 5.6 Update PanelLayout.svelte: pass correct path to left DirectoryPanel based on mode
- [x] 5.7 Add visual indicator in DirectoryPanel header for manual mode (CSS-only marker)
- [x] 5.8 Ensure left panel's `Enter`/`:cd` in manual mode updates leftPath, not centerPath

## 6. Cross-Backend Paste Routing

- [x] 6.1 Add scheme information to clipboard entries (source scheme stored alongside path)
- [x] 6.2 Modify `handlePaste` in PanelLayout.svelte: detect src/dst schemes and route
- [x] 6.3 Implement local→FTP path: invoke `ftp_upload` with progress
- [x] 6.4 Implement FTP→local path: invoke `ftp_download` with progress
- [x] 6.5 Implement FTP→FTP same-server path: invoke `ftp_rename` for server-side move
- [x] 6.6 Implement FTP→FTP different-server path: download + upload relay
- [x] 6.7 Update paste conflict resolution to work with cross-backend targets

## 7. Tab State Persistence for Detach

- [x] 7.1 Add leftMode, leftPath, leftCursorIndex, leftScrollOffset fields to TabState interface
- [x] 7.2 Update `saveActiveTabState` in tabs store to persist left panel detach state
- [x] 7.3 Update `restoreTabAndFocus` in PanelLayout to restore left panel detach state
- [x] 7.4 Update DirectoryPanel to accept/restore cursor and scroll position from tab state
- [x] 7.5 Ensure `createTab` initializes left panel fields with auto mode defaults

## 8. Ratio Presets

- [x] 8.1 Add `:ratio dual` — sets ratios to [1, 1, 1]
- [x] 8.2 Add `:ratio` (no args) — restores default [1, 1, 3]
- [x] 8.3 Register both in command palette and handleCommandKeydown

## 9. Integration Testing & Polish

- [ ] 9.1 Test FTP connect/browse/navigate/disconnect flow end-to-end
- [ ] 9.2 Test local↔FTP upload/download via yank/paste
- [ ] 9.3 Test FTP file delete and rename
- [ ] 9.4 Test detach/attach toggle and visual indicator
- [ ] 9.5 Test tab switch preserves left panel detach state
- [ ] 9.6 Test virtual root shows both drives and FTP connections
- [ ] 9.7 Test connection persistence (save on connect, restore on startup)
- [x] 9.8 Run `cargo check` and `npx svelte-check` to verify no type errors
