## Context

当前项目有 3 个分散的 vim 命令注册点：`processOverlayCommand()` 白名单（在 PreviewEditor/FullscreenEditor 中）、`createVimCommandHandler()` ViewPlugin（vim-commands.ts）、以及 `Vim.defineEx` ad-hoc 调用（`:reg` 在 setupVimRegCommand）。新增一个 `:set` 命令需要改动 overlay handler + ViewPlugin + 可能多个文件，不可持续。

需要一个统一的选项注册框架，让后续扩展（`:set tabstop`、`:set shiftwidth` 等）只需声明注册即可。

## Goals / Non-Goals

**Goals:**
- `VimOptionStore` 单例作为所有 vim 选项的唯一注册入口
- `:set` 命令支持完整语法：设置、取消、toggle、查询、列出
- `windrc.json` 存储在 `%APPDATA%/wind/`，持久化被修改过的选项
- 行号模式切换（`:set nu` / `:set rnu` / `:set nonu`）作为 vim-options 的第一个消费者
- basicSetup → 分解 imports + compartment 控制 lineNumbers
- ex 命令注册整合到统一入口（在 initEditor 中调用一次）

**Non-Goals:**
- 不实现 `:setlocal`（无 buffer/window 概念）
- 不支持 `:set option+=value` / `-=` 语法
- 不迁移已有的 `:w` `:q` 处理逻辑（这些是 editor 生命周期命令，不属于选项系统）
- 不实现自动补全（`:set <Tab>` 列出可用选项）

## Decisions

### Decision 1: VimOptionStore 作为单例模块

在 `vim-options.ts` 顶层创建 store 实例并 export。Svelte 无法跨组件共享 `$state`，使用发布-订阅模式做变更通知。

```typescript
export const vimOptions = {
  register(meta: OptionMeta): void,
  get<T>(name: string): T,
  set(name: string, value: unknown): void,
  toggle(name: string): void,
  onChange(name: string, fn: (value: unknown) => void): () => void,
  list(): OptionSnapshot[],
  save(): Promise<void>,    // → invoke write_config
  load(): Promise<void>,    // → invoke read_config
}
```

**为什么不使用 Svelte store / $state**：`$state` 在 Svelte 5 中限定在组件作用域，无法从纯 TS 模块共享。发布-订阅模式通用性更好，也方便非 Svelte 消费方使用。

### Decision 2: windrc.json 在 Rust 端读写

位置：`dirs::config_dir().join("wind").join("windrc.json")`，Windows 上即 `%APPDATA%/wind/windrc.json`。

两个新 Tauri command：
- `read_config()` → 返回 `Value` (serde_json)，文件不存在返回空对象
- `write_config(options: Value)` → 写入文件，自动创建目录

**为什么不从前端直读文件**：项目已有 `invoke` 命令体系，前后端风格统一。后端还可以做校验、merge 逻辑。

### Decision 3: basicSetup 分解

`basicSetup` 只 export 3 个函数（codemirror v6.03），无法局部拆解。改为从 `@codemirror/view`、`@codemirror/commands`、`@codemirror/language` 等子包导入独立 extensions。

两个 editor 的 initEditor() 中，`basicSetup` 替换为：

```typescript
[
  ...nonLineNumberExtensions, // highlightActiveLineGutter, history, foldGutter, ...
  search({ top: true }),
  lineNumberCompartment.of(lineNumbers()),
  ...
]
```

### Decision 4: :set 命令注册到 Vim.defineEx

```
:set nu        → vimOptions.set('number', true)
:set nonu      → vimOptions.set('number', false)  
:set nu!       → vimOptions.toggle('number')
:set invnu     → 同 toggle
:set nu?       → vimOptions.get('number') → 显示当前值
:set           → 列出所有 modified=true 的选项
:set all       → 列出所有注册的选项
```

通过在 `Vim.defineEx('set', 'se', handler)` 中解析 `params.argString`。解析逻辑与 Neovim 一致：`no` 前缀 = false，`inv` 前缀 = toggle，`!` 后缀 = toggle，`?` 后缀 = query。

### Decision 5: 行号模式作为独立模块

`vim-line-numbers.ts` 通过 `vimOptions.register()` 注册 `number` 和 `relativenumber` 两个布尔选项，并注册 onChange 回调来 reconfigure compartment。

```typescript
export function setupVimLineNumbers(comp: Compartment, view: EditorView) {
  vimOptions.register({ name: 'number', shortName: 'nu', type: 'boolean',
    defaultValue: true, persist: true, onChange: (v) => rebuild(view, comp) });
  vimOptions.register({ name: 'relativenumber', shortName: 'rnu', type: 'boolean',
    defaultValue: false, persist: true, onChange: (v) => rebuild(view, comp) });
}

function rebuild(view: EditorView, comp: Compartment) {
  const nu = vimOptions.get<boolean>('number');
  const rnu = vimOptions.get<boolean>('relativenumber');
  view.dispatch({ effects: comp.reconfigure(buildLineNumbers(nu, rnu)) });
}
```

## Risks / Trade-offs

- **basicSetup 分解可能遗漏扩展** → 对照 `@codemirror/basic-setup` 源码逐项列出，两边比对确认覆盖
- **windrc.json 写失败**（权限/磁盘满） → Rust 端 catch io error，前端静默忽略，不阻塞用户操作
- **Vim.defineEx('set') 可能和未来其他 'set' 注册冲突** → 在 initEditor 中只调用一次，`regCommandSetup` guard 防止重复
- **行号 compartment reconfigure 性能** → `formatNumber` 在 viewport 内每行调用，函数体极轻量（整数减法），无性能问题
