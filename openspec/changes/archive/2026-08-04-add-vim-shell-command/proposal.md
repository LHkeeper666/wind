## Why

在 vim 编辑模式下，用户经常需要快速执行 shell 命令（如 `git diff`、`ls`、`grep`），而不必切换到浮动终端。`:!` 是 vim 的核心工作流命令，缺失它会导致频繁的窗口切换，打断编辑流。

## What Changes

- 在 vim ex 命令行中新增 `:!` 前缀支持，后续文本作为 shell 命令执行
- 使用 Git Bash (`bash -c`) 执行命令，带路径检测 fallback
- 命令工作目录自动设为当前编辑文件的所在目录
- 执行结果通过编辑器底部弹出的可滚动面板显示
- 输出文本支持鼠标选中和 Ctrl+C 原生复制
- Enter 或 Esc 关闭输出面板，返回编辑模式

## Capabilities

### New Capabilities
- `vim-shell-command`: vim 编辑模式下通过 `:!` 执行 shell 命令并显示输出

### Modified Capabilities
- `vim-editor`: 新增 `:!` ex 命令处理需求，扩展命令行模式的行为

## Impact

- `src-tauri/src/lib.rs` — 新增 `exec_shell_command` Tauri 命令
- `src/lib/components/PreviewEditor.svelte` — `processOverlayCommand()` 中加 `!` 处理 + 输出面板 UI
- `src/lib/components/FullscreenEditor.svelte` — 同上
- `src/lib/utils/vim-commands.ts` — `processCommand()` 中加 `!` 处理作为兜底
