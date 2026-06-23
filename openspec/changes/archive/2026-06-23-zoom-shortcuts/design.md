## Context

当前所有组件使用硬编码的 `px` 字号（10px~16px 不等），分散在 10+ 个文件中，没有统一的缩放机制。用户只能通过操作系统级别的显示缩放来调整，无法在应用内独立控制。

Tauri 2 在 Windows 上使用 WebView2 (Chromium)，完整支持 CSS `zoom` 属性。

## Goals / Non-Goals

**Goals:**
- 通过 `Ctrl+=`/`Ctrl+-`/`Ctrl+0` 缩放所有面板的内容（文字、图标等）
- 列宽布局、拖动手柄、状态栏不参与缩放
- 缩放范围 50%~200%，步长 10%
- 缩放时显示 toast 提示当前百分比

**Non-Goals:**
- 不缩放整个窗口（不改 window.innerWidth）
- 不持久化缩放值（后续可加）
- 不改变 grid 列宽比例

## Decisions

### 缩放方案：CSS `zoom` 作用于 `.panel` 元素

**选择**：在 `.panel` 元素上应用 `zoom: var(--zoom-level)`。

**为什么不用整页 zoom**：`document.documentElement.style.zoom` 会缩放整个窗口（包括状态栏和拖动手柄），不符合需求。

**为什么不用 `transform: scale()`**：`scale()` 只缩放视觉，不改变元素占用的布局空间，会导致内容溢出或出现滚动条。

**为什么不用 rem 重构**：需要改动 50+ 处硬编码字号，工作量大且容易遗漏。

`.panel` 元素已有 `overflow: hidden`，zoom 后的内容不会溢出。

### 存储方式：CSS 变量 + 内存状态

将缩放级别存储在：
1. CSS 变量 `--zoom-level` 在 `:root` 上（驱动视觉缩放）
2. PanelLayout.svelte 中的 `$state()` 变量（驱动 toast 显示和快捷键逻辑）

不写入 layout store，因为缩放是纯前端视觉设置，与后端状态无关。

### 快捷键：Ctrl+= / Ctrl+- / Ctrl+0

遵循行业标准（VS Code、Chrome、Windows Terminal）。在 `handleGlobalKeydown` 中拦截。

## Risks / Trade-offs

- **CSS zoom 在极端比例下可能轻微模糊** → 范围限制在 0.5~2.0，步长 0.1，实践中极少超出 0.8~1.3
- **`.panel` 内如有绝对定位元素可能偏移** → 当前面板内无固定/绝对定位元素，风险低
- **与 fullscreen overlay 的交互** → fullscreen overlay 是 `.app-layout` 的直接子元素，不在 `.panel` 内，不受影响
