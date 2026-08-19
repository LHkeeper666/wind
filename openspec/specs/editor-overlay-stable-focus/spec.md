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

### Requirement: 鼠标选中在编辑器外释放后 overlay 保持焦点

在 normal 模式下，系统 SHALL 注册文档级 mouseup 监听器，当用户在编辑器面板外释放鼠标时，自动将焦点恢复到 overlay div，防止 CodeMirror 获得焦点导致键盘输入绕过 overlay handler。

#### Scenario: 鼠标拖拽选中后在面板外释放

- **WHEN** 用户在 editor-normal 模式下用鼠标拖拽选择文本
- **AND** 鼠标在编辑器面板区域外释放（mouseup 事件不命中 .editor-content）
- **THEN** 下一帧 overlay 重新获得焦点
- **AND** 键盘输入继续由 overlay 的 keydown handler 处理

#### Scenario: 鼠标拖拽选中后在面板内释放

- **WHEN** 用户在 editor-normal 模式下用鼠标拖拽选择文本
- **AND** 鼠标在编辑器面板区域内释放
- **THEN** 现有的 `onmouseup` handler 正常工作（重新聚焦 overlay）

#### Scenario: 切换到其他面板后不抢回焦点

- **WHEN** 用户通过 Ctrl+W h/l 切换到其他面板
- **AND** `activeColumn` 不再是 'preview'
- **THEN** mouseup listener 不重新聚焦 overlay

#### Scenario: overlay 隐藏时清理 listener

- **WHEN** 编辑器从 normal 模式切换到 insert 模式（overlay 隐藏）
- **THEN** 文档级 mouseup listener 被移除

### Requirement: 仅活动 Vim 会话可接收编辑器焦点

当多个 Tab 保留 Vim 编辑器会话时，系统 SHALL 只允许活动 Tab 的会话接收焦点和编辑器级键盘事件。后台会话 MUST 隐藏且不得通过 overlay、CodeMirror 内容区或会话观察器抢占焦点。

#### Scenario: 切换到另一个 Vim normal 模式 Tab
- **WHEN** Tab A 和 Tab B 都保留 Vim normal 模式会话
- **AND** 用户从 Tab A 切换到 Tab B
- **THEN** Tab B 的 overlay 成为唯一可聚焦的 Vim normal 输入目标
- **AND** Tab A 的 overlay 不再接收键盘输入或抢回焦点

#### Scenario: 后台会话触发延迟测量
- **WHEN** 非活动 Tab 的隐藏 CodeMirror 会话产生延迟测量或观察器回调
- **THEN** 该回调不得改变 `document.activeElement`
- **AND** 不得改变活动 Tab 的 Vim mode 或光标状态
