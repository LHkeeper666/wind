## Context

Wind 的 vim 编辑器基于 CodeMirror 6 + `@replit/codemirror-vim` 构建。当前键盘输入分两条路径：

- **editor-normal 模式**：`<div>` overlay 捕获所有键盘事件，通过 `codeToVimKey()` 转换为 vim key 字符串，再调用 `Vim.multiSelectHandleKey()` 分发给 vim 引擎
- **editor-insert 模式**：overlay 隐藏，CodeMirror 直接接收原生键盘事件，通过 keymap 系统处理

`@codemirror/commands` 已安装（v6.10.3），提供 `toggleComment` 命令和 `commentKeymap`（绑定 `Ctrl+/` 和 `Ctrl+Shift+/`）。

## Goals / Non-Goals

**Goals:**
- `Ctrl+/` 在 vim 编辑器的所有模式（normal/insert/visual）下均可切换行注释
- 选中多行时，`Ctrl+/` 对每行同时添加/移除注释符
- 两个编辑器（PreviewEditor 和 FullscreenEditor）行为一致

**Non-Goals:**
- 不支持 vim 原生注释快捷键（如 `gcc`、`gc`），仅实现 VSCode 风格的 `Ctrl+/`
- 不新增 `Ctrl+Shift+/` 块注释切换（`commentKeymap` 自带，由 CodeMirror 自动处理）
- 不修改 `global-normal` 预览模式的行为（预览模式为只读，不需要注释功能）

## Decisions

### Decision 1: 双重拦截策略

在 `handleOverlayKeydown` 中拦截 `Ctrl+/` 事件（editor-normal 模式），同时在 CodeMirror extensions 中添加 `commentKeymap`（editor-insert 模式）。

**理由**：两条路径相互独立。overlay 路径在 editor-normal 下生效，CodeMirror keymap 在 editor-insert 下生效。不存在一个方案同时覆盖两条路径的情况。

**替代方案**：在 vim 引擎中注册自定义映射（如 `:nmap <C-/> :ToggleComment`）。但 `@replit/codemirror-vim` 的自定义 Ex 命令机制较复杂，且需要处理 visual 模式下的选区映射，不如直接调用 `toggleComment` 简单。

### Decision 2: 在 overlay handler 中直接调用 `toggleComment(editorView)`

而非通过 `Vim.multiSelectHandleKey` 转发。

**理由**：`Ctrl+/` 不是 vim 原生键位，`Vim.multiSelectHandleKey` 不认识它也无法正确处理。直接调用 `toggleComment` 绕过 vim 引擎，行为更可预测。

### Decision 3: 使用 `commentKeymap` 而非手动绑定

在 CodeMirror extensions 中直接 spread `commentKeymap`。

**理由**：`commentKeymap` 是 CodeMirror 官方提供的 keymap，已包含 `Ctrl+/` → `toggleComment` 和 `Ctrl+Shift+/` → `toggleBlockComment` 两组绑定。手动绑定会增加维护成本且无额外收益。

## Risks / Trade-offs

- **`commentKeymap` 与 vim keymap 的优先级冲突**：在 editor-insert 模式下，如果 vim 引擎也拦截 `Ctrl+/`，`commentKeymap` 可能不生效。但 `@replit/codemirror-vim` 的 insert 模式对 `Ctrl+/` 没有特殊处理，所以不存在冲突。如发现问题，可通过 `Prec.highest` 提升 `commentKeymap` 优先级。
- **部分语言不支持注释**：`toggleComment` 在无法识别注释语法的语言中是 no-op，不会报错或产生副作用。