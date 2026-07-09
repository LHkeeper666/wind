## Why

条件渲染 `{#if mode === 'editor-normal'}` 导致 overlay div 随模式切换被反复创建/销毁，`bind:this` 引用在焦点切换窗口期失效。同时 `initEditor()` 无条件调用 `editorView.focus()` 把焦点短暂交给 CodeMirror 的 contentEditable，Windows IME 检测到可编辑元素后抢走焦点，导致 vim normal 模式下按键行为异常（j/k/o 无效或退出编辑模式）和中文输入法意外弹窗。

## What Changes

- PreviewEditor 的 overlay div 改为常驻 DOM，通过 CSS class 控制显隐、`tabindex` 动态切换可达性
- FullscreenEditor 的 overlay div 做同样改动（相同架构，相同风险）
- 删除 `initEditor()` 中多余的 `editorView.focus()` 调用，焦点统一由 mode `$effect` 管理
- overlay 上注册 `compositionstart`/`compositionend` 事件拦截作为 IME 防御层

## Capabilities

### New Capabilities
- `editor-overlay-stable-focus`: 编辑器 overlay 常驻 DOM，焦点管理可靠，IME 不会在 normal 模式下激活

### Modified Capabilities
<!-- No existing spec requirements changed; this is a bug fix that doesn't alter spec-level behavior -->

## Impact

- `src/lib/components/PreviewEditor.svelte` — 模板、$effect、initEditor()
- `src/lib/components/FullscreenEditor.svelte` — 模板、$effect、initEditor()
