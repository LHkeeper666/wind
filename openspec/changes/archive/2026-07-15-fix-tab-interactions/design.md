## Context

TabBar 当前是"哑"组件——只通过 `tabs.switchTab()` 操作 store 的 `activeTabId`，不知道 PanelLayout 有完整的状态保存/恢复流程（`saveCurrentTabState()` + `restoreTabAndFocus()`）。PanelLayout 通过键盘快捷键调用这些函数，但 TabBar 的鼠标点击绕过了它们。

## Goals / Non-Goals

**Goals:**
- 鼠标点击 tab 触发与键盘快捷键相同的完整切换流程
- `t r` 快捷键触发当前激活 tab 的内联重命名输入框
- Tab 标签根据窗口宽度动态等分可用空间

**Non-Goals:**
- 不改变 tab 状态保存/恢复的核心逻辑
- 不增加新的 store 字段
- 不引入组件库或第三方依赖

## Decisions

### Decision 1: TabBar 通过 callback props 与 PanelLayout 通信

给 TabBar 添加两个 props：
- `onSwitchTab: (tabId: number) => void` — 替代直接调用 `tabs.switchTab()`
- `onRenameTab: (tabId: number) => void` — 触发重命名

**为什么不用事件/custom event？** Svelte 的 component events 在 Svelte 5 中已废弃，callback props 是推荐的父子通信方式。而且 PanelLayout 已经用 `bind:this` 获取子组件引用，callback 模式与现有代码风格一致。

**为什么不用 store 驱动的 reactive 方式？** 如果在 store 里加一个 `pendingRenameTabId` 字段，TabBar 通过 `$effect` 响应，会增加不必要的响应式复杂度。TabBar 的重命名状态已经是本地 `$state`，没必要提到 store 层。

### Decision 2: `t r` 触发当前激活 tab 的重命名

当用户按 `t r` 时，PanelLayout 调用 `onRenameTab(activeTabId)`，TabBar 设置 `renamingId = activeTabId` 并聚焦输入框。这样与双击重命名的用户体验一致——都是用同一个内联输入框。

**为什么不是弹出一个 modal？** 双击已经是内联编辑，`t r` 应该保持一致。而且内联编辑更轻量、更快。

### Decision 3: Tab 宽度使用 CSS flex 等分

```css
.tab-item {
    flex: 1 1 0;       /* 等分剩余空间，可收缩 */
    min-width: 80px;    /* 太窄无意义 */
    max-width: 250px;   /* 太宽浪费空间 */
}
.tab-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
```

**为什么用纯 CSS 而不是 JS 计算？** tab 数量少（通常 < 20），CSS flex 天然支持等分 + 溢出处理，无需引入 ResizeObserver 或手动计算。

**为什么是最小/最大宽度而非固定比例？** `min-width: 80px` 保证 tab 在任何窗口下至少能看到序号，`max-width: 250px` 防止全屏时单个 tab 占用过多空间。

## Risks / Trade-offs

- [新增 callback props 不传递时 TabBar 回退到旧行为] → TabBar 内部检查 props 是否存在，`onSwitchTab` 不存在时仍调用 `tabs.switchTab()` 保证向后兼容
- [Tab 重命名输入框在 tab 很窄时显示不佳] → 输入框使用 `position: absolute` 或 `min-width` 确保可用性
- [Tab 很多时 flex 等分导致名称截断] → 已有 `text-overflow: ellipsis`，配合 title 属性显示完整名称（可后续添加）
