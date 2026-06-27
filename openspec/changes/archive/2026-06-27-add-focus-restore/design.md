## Context

Wind 是一个 vim 驱动的文件工作站，使用三栏布局（parent / current / preview）+ 浮动终端。`PanelLayout.svelte` 中的 `focusPanel()` 函数是焦点管理的核心，`layout.activeColumn` 跟踪当前活跃面板。

问题：从其他应用 Alt+Tab 切回时，Tauri 窗口获得 OS 焦点，但 DOM 焦点丢失，键盘事件无法被任何面板捕获。

当前 `handleGlobalKeydown` 已注册在 `window` 的 capturing phase，所以键盘事件理论上仍能被处理——但某些面板（如 DirectoryPanel 的 vim 导航）依赖 `document.activeElement` 在特定元素上才能正常工作。

## Goals / Non-Goals

**Goals:**
- 窗口重获焦点时自动恢复 DOM 焦点到上次活跃面板
- 提供 Ctrl+L 手动恢复焦点的快捷键
- 两种方式均显示 toast 提示

**Non-Goals:**
- 全屏模式下的焦点管理（overlay 已独占）
- 终端 insert 模式下的 Ctrl+L 拦截（透传给 shell）
- 跨面板焦点记忆（不跟踪"上上次"活跃面板）

## Decisions

### 1. 使用 Tauri `getCurrentWindow().onFocusChanged` 监听窗口焦点

**选择**: Tauri 2 的窗口 API `onFocusChanged`

**替代方案**:
- `document.addEventListener('visibilitychange')` — 只检测页面可见性，不精确
- `window.addEventListener('focus')` — 浏览器级事件，在 Tauri 中行为不一致

**理由**: Tauri 原生事件最可靠，直接对应窗口级焦点变化。

### 2. 焦点检测策略：`document.activeElement` + `.panel-layout` 容器检查

**选择**: 检查 `document.activeElement` 是否在 `.panel-layout` 容器内

```
activeElement 在 .panel-layout 内？ → 不干预
activeElement 是 body/null？ → 自动恢复
activeElement 在 overlay/modal 内？ → 不干预
```

**替代方案**:
- 为每个面板维护 `hasFocus` 状态 — 需要大量 $effect 和事件监听
- 只检查 `document.hasFocus()` — 无法区分"焦点在面板"和"焦点在 overlay"

**理由**: 利用 DOM 原生 API，无需额外状态管理，覆盖所有场景。

### 3. Ctrl+L 排除条件

在 `handleGlobalKeydown` 中拦截，排除以下场景：
- `showCommandPalette` 或 `showFileSearch` 为 true
- `$layout.fullscreenEditorOpen` / `fullscreenImageViewerOpen` / `fullscreenPdfViewerOpen` / `fullscreenVideoPlayerOpen`
- `$layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert'`

### 4. 防止窗口首次加载误触发

**选择**: 使用 `windowReady` 标志位，`onMount` 中延迟设置

```ts
let windowReady = false;
onMount(() => {
  setTimeout(() => { windowReady = true; }, 500);
});
```

`onFocusChanged` 回调中检查 `windowReady`，跳过首次触发。

### 5. Toast 内容

格式: `Focus: CURRENT` / `Focus: PARENT` / `Focus: PREVIEW` / `Focus: TERMINAL`

复用现有 `showToast()` 函数。

## Risks / Trade-offs

- **[风险] Tauri `onFocusChanged` 在某些平台上可能触发时机不精确** → 使用 `windowReady` 标志位延迟注册，避免初始化干扰
- **[风险] 焦点恢复可能打断用户正在进行的操作** → 通过 `activeElement` 检测避免覆盖已有焦点
- **[权衡] Ctrl+L 与终端 clear 快捷键冲突** → 仅在非 insert 模式拦截，insert 模式透传给 shell
