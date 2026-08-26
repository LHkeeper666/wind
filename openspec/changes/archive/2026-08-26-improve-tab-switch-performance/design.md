## Context

Tab 切换时存在三个性能问题：
1. 不同列宽布局的 tab 之间切换时，`grid-template-columns` 的 CSS transition（0.2s）与内容切换同时发生，导致掉帧卡顿
2. 预览面板的 slot 在 z-index 切换后立即可见，但 `loadFile()` 是异步的，中间窗口显示旧 tab 的残留内容
3. 终端容器用 `display: none/block` 切换，触发 layout recalc

当前架构：PreviewEditor 的 slot 已经用 `z-index` 策略（不触发 layout），但终端容器未对齐。

## Goals / Non-Goals

**Goals:**
- 消除 tab 切换时 grid 布局过渡导致的卡顿
- 消除预览面板的"闪现旧内容"
- 终端容器切换对齐 PreviewEditor 的 z-index 策略

**Non-Goals:**
- 不添加任何过渡动画（用户偏好无动画）
- 不改变 tab 切换的架构流程
- 不修改缓存策略（`tabEditorCache`、editor session 保持不变）

## Decisions

### 1. 移除 grid-template-columns transition

直接删除 `.panel-layout` 的 `transition: grid-template-columns 0.2s ease`。用户确认 resize 拖拽也不需要过渡效果，无需保留。

### 2. showTabSlot 时清空 slot innerHTML

在 `showTabSlot()` 中，切换 z-index 之前清空目标 slot 的 `innerHTML` 和 `dataset`。缓存数据在 `tabEditorCache` 中独立保留，`renderPreview()` 会重新生成 DOM。

### 3. 终端容器用 visibility + z-index

`setContainerVisible()` 改用 `visibility: hidden` + `z-index` 替代 `display: none`。`.terminal-container` 已有 `position: absolute; inset: 0`，所有容器叠加在同一位置，z-index 控制可见性。

## Risks / Trade-offs

- **xterm.js 在 visibility:hidden 下的行为**: Canvas 元素在 `visibility: hidden` 时仍会渲染（只是不可见），预期不影响终端输出。→ 实际验证
- **slot 清空后缓存未命中时短暂空白**: 比显示错误内容好，且缓存命中时同步渲染，1 帧内完成不可见
- **所有终端容器常驻 DOM**: 已有此行为（`display: none` 不移除 DOM），改为 `visibility: hidden` 不改变内存占用