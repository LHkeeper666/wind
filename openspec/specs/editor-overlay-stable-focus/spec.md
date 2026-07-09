# editor-overlay-stable-focus Specification

## Purpose
TBD - created by archiving change fix-editor-overlay-focus-race. Update Purpose after archive.
## Requirements
### Requirement: Overlay DOM element is stable across mode switches

编辑器 overlay div SHALL 在组件的整个生命周期内保持存在于 DOM 中，不因 vim 模式切换而被创建或销毁。显隐通过 CSS `display: none` 控制，可达性通过动态 `tabindex`（normal 模式为 0，其他模式为 -1）控制。

#### Scenario: Switching from normal to insert mode
- **WHEN** 用户在 editor-normal 模式下按 i/a/o 进入 insert 模式
- **THEN** overlay 变为 `display: none` 且 `tabindex=-1`，但 DOM 元素不被移除
- **AND** `bind:this` 的 overlayElement 引用保持有效

#### Scenario: Switching from insert back to normal mode
- **WHEN** 用户在 editor-insert 模式下按 Escape 回到 normal 模式
- **THEN** overlay 变为 `display: block` 且 `tabindex=0`
- **AND** overlay 获得焦点，IME 不激活

#### Scenario: Tab switch away and back
- **WHEN** 用户在 editor-normal 模式下切换到另一个 Tab 再切回
- **THEN** overlay DOM 元素仍然存在且 `bind:this` 引用有效
- **AND** overlay 正确获得焦点

### Requirement: Focus is managed by a single owner

编辑器焦点 SHALL 由 mode `$effect` 统一管理。`initEditor()` 函数 SHALL NOT 调用 `editorView.focus()`，避免焦点在 CodeMirror contentEditable 和 overlay 之间不必要的翻转。

#### Scenario: First time entering editor mode
- **WHEN** 用户在预览模式按 e 首次进入编辑模式
- **THEN** `initEditor()` 创建 CodeMirror 实例但不调用 `editorView.focus()`
- **AND** mode `$effect` 将焦点置于 overlay（normal 模式）或 editorView（insert 模式）

#### Scenario: Editor already initialized, re-entering normal mode
- **WHEN** 编辑器已初始化且用户从 insert 切回 normal
- **THEN** `initEditor()` 不被调用
- **AND** mode `$effect` 将焦点置于 overlay

### Requirement: IME composition is blocked on overlay

overlay div SHALL 以 capture 阶段注册 `compositionstart` 事件监听器，调用 `preventDefault()` 和 `stopPropagation()` 阻止 IME 在 normal 模式下激活。

#### Scenario: IME attempts to compose on overlay
- **WHEN** overlay 持有焦点且系统 IME 尝试发起 composition
- **THEN** `compositionstart` 事件被拦截，IME 不激活
- **AND** 后续的 `compositionupdate` 和 `compositionend` 事件也不会触发

