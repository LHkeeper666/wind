# Binary selection preview browser check

Run `npm run tauri dev` and open a directory containing a `.torrent` file larger than 1 MiB. A copy named `.TORRENT` checks extension case. Select either file in the current panel.

1. In DevTools, instrument `window.__TAURI_INTERNALS__.invoke` before selecting the file. Record command names, then verify one `get_file_metadata` and one `read_binary_file_partial` call, with no `read_file` or `read_file_partial`. A file at most 1 MiB should use `read_binary_file` instead.
2. Verify the notice reports the disk size and “仅显示前 64.0 KB”. In the console, run `document.querySelectorAll('.selection-preview .hex-pre [data-line]').length`; it should stay near the viewport row count plus 16, rather than 4096.
3. Scroll the selection body to the bottom. Check that the last row has `data-line="4095"`, starts at offset `0000fff0`, and its ASCII column matches bytes 65520–65535. Repeat the row count check.
4. Hold `j` or `k` to switch selections quickly. The displayed file and notice should match the latest selection; scrolling the new preview should still update its hex rows.

If no torrent file is available, a temporary binary file with a `.torrent` suffix and more than 1 MiB of data exercises the same path.
