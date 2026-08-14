## 1. Rust 后端：配置文件读写

- [x] 1.1 lib.rs: 新增 `read_config()` Tauri command，读取 `%APPDATA%/wind/windrc.json`，文件不存在返回空 `{}`
- [x] 1.2 lib.rs: 新增 `write_config(options: Value)` Tauri command，写入 `%APPDATA%/wind/windrc.json`，自动创建目录

## 2. VimOptionStore 核心模块

- [x] 2.1 新增 `src/lib/utils/vim-options.ts`：实现 VimOptionStore 单例（register / get / set / toggle / onChange / list）
- [x] 2.2 `:set` 命令解析器：支持 nu/nonu/nu!/invnu、查询（nu?）、列出（空/all）
- [x] 2.3 windrc.json 集成：save() 调用 invoke write_config，load() 调用 invoke read_config，只在 persist=true 的选项变更时保存

## 3. basicSetup 分解

- [x] 3.1 PreviewEditor.svelte: `basicSetup` 替换为独立 imports（highlightActiveLineGutter, history, foldGutter 等），初始化为 `lineNumberCompartment.of(lineNumbers())`
- [x] 3.2 FullscreenEditor.svelte: 同上

## 4. Vim 行号实现

- [x] 4.1 新增 `src/lib/utils/vim-line-numbers.ts`：register number/relativenumber 选项到 VimOptionStore，通过 `lineNumbers({formatNumber})` 实现 relative/hybrid 模式
- [x] 4.2 compartment reconfigure：onChange 回调中 `view.dispatch({ effects: comp.reconfigure(buildLineNumbers(nu, rnu)) })`

## 5. 命令注册整合

- [x] 5.1 vim-commands.ts: 新增 `setupAllVimCommands()` 统一入口，聚合 `:set` (`setupVimLineNumbers` + VimOptionStore 的 set 命令解析器) + `:reg` (`setupVimRegCommand`)，确保 Vim.defineEx 之间不冲突

## 6. 编辑器集成

- [x] 6.1 PreviewEditor.svelte: initEditor 中调用 `setupAllVimCommands()` 替换分散的 setupVimRegCommand 调用
- [x] 6.2 FullscreenEditor.svelte: 同上

## 7. 验证

- [x] 7.1 cargo check: Rust 编译无错误
- [x] 7.2 svelte-check: 前端无类型错误
- [ ] 7.3 手动测试: `:set rnu` → 行号切换为相对模式；`:set nu` → 恢复绝对行号；`:set nonu` → 隐藏行号
- [ ] 7.4 手动测试: 设置 `:set rnu` 后重启应用 → 行号仍为相对模式
