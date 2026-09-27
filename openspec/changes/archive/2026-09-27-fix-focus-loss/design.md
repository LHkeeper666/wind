## Context

Wind 使用 Tauri 2 + Svelte 5 构建，采用三栏布局（parent / current / preview）。焦点管理是 vim-driven 工作流的核心：用户通过 h/l 在面板间切换，j/k 在面板内导航，所有操作依赖 DOM 焦点正确落在目标面板上。

最近的重构（提取 composables、拆分组件）改变了 DOM 结构和组件挂载顺序，暴露了焦点管理中的两个脆弱点：

1. `handleWindowFocusChanged` 直接调用 `focusPanelNow`，没有重试机制
2. ProjectTreePanel 的 toggle 点击后，Svelte 异步重渲染可能冲掉焦点

## Goals / Non-Goals

**Goals:**
- Alt-Tab 切回窗口后，焦点可靠恢复到 `activeColumn` 对应的面板
- Ctrl+L 第一次按下就能恢复焦点（不需要按两次）
- 项目树 toggle 点击后焦点保持在目录面板上
- preview 面板的 `.preview-editor` 元素不可用时，有 fallback 焦点目标

**Non-Goals:**
- 不改变焦点管理的整体架构（activeColumn + focusPanel 模式）
- 不修改 VimOverlay 或 CodeMirror 的焦点逻辑
- 不改变 Tab 切换的焦点恢复流程（已经使用 focusPanel，工作正常）

## Decisions

### Decision 1: `handleWindowFocusChanged` 改用 `focusPanel`

**选择：** 将 `handleWindowFocusChanged` 中的 `focusPanelNow($layout.activeColumn)` 替换为 `focusPanel($layout.activeColumn)`。

**理由：**
- `focusPanel` 已经包含 `focusPanelNow` + `tick().then(() => focusPanelNow())` 的重试逻辑
- `focusPanel` 还会调用 `setActiveColumn`，但列未变时是 no-op，无副作用
- 避免重复实现重试逻辑

**替代方案：** 在 `handleWindowFocusChanged` 中手动添加 tick 重试 — 重复代码，维护成本高。

### Decision 2: `previewPanel` 的 tabindex 改为 0

**选择：** 将 PanelLayout 中 `previewPanel` div 的 `tabindex` 从 `-1` 改为 `0`。

**理由：**
- `focusPanelNow` 的 preview 分支在 `.preview-editor` 不可用时需要一个 fallback 焦点目标
- `tabindex="-1"` 的元素无法通过 `focus()` 获得焦点（编程式 focus 也不行，取决于浏览器实现）
- `tabindex="0"` 使 `previewPanel` 成为可聚焦元素，可作为最终 fallback

**替代方案：** 不改 tabindex，只依赖 `.preview-editor` — 但如果 PreviewEditor 组件未挂载，焦点无处可去。

### Decision 3: toggle 点击后延迟恢复焦点

**选择：** 在 ProjectTreePanel 的 `handleToggleClick` 中，将 `panelElement?.focus()` 改为 `setTimeout(() => panelElement?.focus(), 0)`。

**理由：**
- `toggleTreeNode` 触发异步子节点加载 → Svelte 重渲染 → DOM 变化可能导致焦点丢失
- `setTimeout(fn, 0)` 将焦点恢复推迟到当前事件循环结束后，确保 Svelte 的 DOM 更新已完成
- 这是 Svelte 中处理 post-render 焦点的标准模式

**替代方案：** 使用 `tick().then()` — 语义更清晰，但 `tick()` 只等待 Svelte 的微任务，不一定覆盖浏览器的 DOM 更新。`setTimeout` 更可靠。

## Risks / Trade-offs

- **Risk：** `setTimeout(fn, 0)` 可能导致可见的焦点闪烁（先丢焦点再恢复）
  - **Mitigation：** 0ms 延迟几乎不可感知，且只在 toggle 点击时触发
- **Risk：** `previewPanel` 的 `tabindex="0"` 可能导致 Tab 键导航时焦点落在 preview panel 容器上
  - **Mitigation：** Wind 的 Tab 键已被 `handleGlobalKeydown` 拦截并 `preventDefault()`，不会触发原生 Tab 导航
