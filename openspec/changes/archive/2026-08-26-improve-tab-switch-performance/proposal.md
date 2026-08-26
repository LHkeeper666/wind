## Why

Tab 切换时不同列宽的 grid 布局过渡动画（0.2s）与内容切换同时发生，导致明显卡顿；预览面板会先闪现旧 tab 的残留内容再更新；终端容器用 display:none 切换触发不必要的 layout recalc。三个问题叠加导致 tab 切换体验生硬不流畅。

## What Changes

**Grid 布局过渡**
- From: `.panel-layout` 有 `transition: grid-template-columns 0.2s ease`，切换不同列宽 tab 时触发 200ms layout-heavy 动画
- To: 移除 transition，列宽瞬间切换
- Reason: grid-template-columns transition 每帧都触发 layout recalc，与内容切换叠加导致掉帧
- Impact: 非破坏性，视觉效果从"卡顿过渡"变为"瞬间切换"

**预览面板内容闪现**
- From: `showTabSlot()` 切换 z-index 后 slot 立即可见，但 `loadFile()` 异步完成前显示旧残留内容
- To: 切换 z-index 前清空 slot 的 innerHTML，确保不显示旧内容
- Reason: 消除"闪现无关内容"的视觉干扰
- Impact: 非破坏性，缓存数据在 `tabEditorCache` 中独立保留不受影响

**终端容器切换策略**
- From: `setContainerVisible()` 使用 `display: none/block` 切换
- To: 使用 `visibility: hidden` + `z-index` 切换
- Reason: 对齐 PreviewEditor 的 slot 策略，避免 layout recalc
- Impact: 非破坏性，容器已使用绝对定位，无需改 CSS 布局

## Capabilities

### New Capabilities

_无新增能力，纯性能优化_

### Modified Capabilities

_无需求变更，纯实现优化_

## Impact

- `src/lib/components/PanelLayout.svelte` — 删除 1 行 CSS transition
- `src/lib/components/PreviewEditor.svelte` — `showTabSlot()` 加清空逻辑
- `src/lib/terminal/terminal-manager.ts` — `setContainerVisible()` 改用 visibility