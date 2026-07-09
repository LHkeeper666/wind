## Context

PreviewEditor 和 FullscreenEditor 使用 CodeMirror 6 + `@replit/codemirror-vim` 实现 vim 风格编辑。为解决 IME 在 vim normal 模式下的干扰，组件在 CodeMirror（contentEditable）上覆盖了一个透明 overlay `<div tabindex="0">`，通过拦截 keydown 事件转发给 vim 引擎。

当前实现用 Svelte 条件渲染 `{#if mode === 'editor-normal'}` 控制 overlay 的创建/销毁。每次 vim 在 normal 和 insert 之间切换时，overlay DOM 元素被创建或从 DOM 中移除，导致 `bind:this` 引用在两个渲染周期间失效。

同时 `initEditor()` 函数末尾无条件调用 `editorView.focus()`，将焦点短暂交给 CodeMirror 的 contentEditable，然后外层 `$effect` 再尝试把焦点移回 overlay。这个 focus → blur → refocus 的序列在 Windows 下是危险的——中文输入法检测到 contentEditable 获得焦点后会激活 composition 管道，可能在 overlay 尝试 focus 之前就已抢走焦点。

## Goals / Non-Goals

**Goals:**
- 消除 mode 切换时的 overlay DOM 创建/销毁周期
- 确保 overlay 的 DOM 引用始终有效，焦点管理不依赖渲染时序
- 阻止 Windows IME 在 vim normal 模式下激活
- 对 FullscreenEditor 做相同修复

**Non-Goals:**
- 改变 vim 模式切换的逻辑或行为
- 改变 CodeMirror 或 vim 扩展的配置
- 修改焦点恢复、Tab 切换等其他子系统

## Decisions

### Decision 1: overlay 用 CSS class 控制显隐，不用条件渲染

**选择**: `display: none` + 动态 `tabindex`（normal 模式 tabindex=0，其他模式 tabindex=-1）

**替代方案考虑**:
- `visibility: hidden` — 元素仍占位，不可接受
- `pointer-events: none` — 无法阻止 IME，因为焦点仍可能落在 contentEditable
- 保持条件渲染 + 延迟聚焦 — 不解决根本的 DOM 引用失效问题

**理由**: `display: none` 使 overlay 脱离渲染流但保留在 DOM 中，`bind:this` 引用永不失效。`tabindex=-1` 确保非 normal 模式时 Tab 不会意外聚焦 overlay。

### Decision 2: 删除 initEditor() 中的 editorView.focus()

**选择**: 删除 `PreviewEditor.svelte:1280` 和 `FullscreenEditor.svelte:284` 的 `editorView.focus()` 调用

**理由**: 焦点管理应统一由 mode `$effect` 负责，不在初始化函数中抢占。`initEditor()` 只应负责创建编辑器，不应管理焦点。

### Decision 3: overlay 上注册 composition 事件拦截

**选择**: 在 overlay 上以 capture 模式监听 `compositionstart` 和 `compositionend`，`preventDefault()` + `stopPropagation()`

**理由**: 这是一个 defense-in-depth 措施。即使 overlay 正确获得焦点，某些 Windows IME 版本仍可能在检测到 DOM 树中有 contentEditable 时尝试发起 composition。主动拦截可以消除这个隐患。

## Risks / Trade-offs

- **overlay 常驻占用一点点内存** — 一个空 div，可忽略
- **CSS class 切换仍然有渲染开销** — 但远小于 DOM 创建/销毁，且 Svelte 不会重新创建元素
- **composition 拦截可能在某些极端的 IME 场景下导致输入法无法使用** — 但仅在 overlay 有焦点时（normal 模式），此时本就不应使用 IME。insert 模式下 overlay 的 tabindex=-1 不会拦截事件
