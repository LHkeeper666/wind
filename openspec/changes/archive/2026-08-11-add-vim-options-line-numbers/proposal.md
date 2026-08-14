## Why

当前 vim 命令系统是分散的白名单模式（每加一个命令要在 3 个地方改代码），缺少可扩展的选项基础设施和持久化机制。现在需要支持行号模式切换（`:set nu/rnu/nonu`），正是建立统一 `:set` 框架的时机——一次投入，后续所有 vim 选项都能零代码新增。

## What Changes

- 新增 `VimOptionStore` 单例：管理 vim 选项的注册、读写、变更通知
- 新增 `windrc.json` 配置文件持久化（Rust 后端读写，存放于 `%APPDATA%/wind/`）
- 支持完整的 `:set` 命令语法：`:set nu`、`:set nonu`、`:set nu!`、`:set invnu`、`:set number?`、`:set`（列出已修改项）、`:set all`
- 行号模式切换：绝对行号（`:set nu`）、相对行号（`:set rnu`）、无行号（`:set nonu`），支持混合模式（nu + rnu）
- `basicSetup` → 分解为独立 extension imports，用 compartment 控制 `lineNumbers()`
- 整合当前散落的 ex 命令注册（`:reg`、`:w`/`:q`）到统一入口

## Capabilities

### New Capabilities
- `vim-options`: VimOptionStore 选项系统 + `:set` 命令解析 + `windrc.json` 持久化到 AppData 目录
- `vim-line-numbers`: 行号模式切换，通过 `lineNumbers({formatNumber})` 支持 absolute / relative / hybrid 三种模式

### Modified Capabilities
- `vim-editor`: 命令路由重构——`:set` 命令下沉到 VimOptionStore，overlay 的 `processOverlayCommand` 与 `Vim.defineEx` 通过统一入口注册

## Impact

- `src/lib/utils/vim-options.ts` — **新增**：VimOptionStore 单例、`:set` 命令解析器、windrc.json 读写
- `src/lib/utils/vim-line-numbers.ts` — **新增**：行号模式 compartment + extension 构建
- `src/lib/utils/vim-commands.ts` — **重构**：整合 `:reg` 注册、新增 `setupAllVimCommands()` 统一入口
- `src/lib/utils/clipboard-bridge.ts` — 无改动
- `src-tauri/src/lib.rs` — **新增**：`read_config` / `write_config` 两个 Tauri command
- `src/lib/components/PreviewEditor.svelte` — basicSetup 替换为独立 imports + compartment
- `src/lib/components/FullscreenEditor.svelte` — 同上
